use itertools::Itertools;
use rand::{thread_rng, Rng};
use shared::{BotJson, GameError, WhichBot};
use std::borrow::BorrowMut;
use std::{
    env,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::{
    fs,
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{ChildStdout, Command},
    time::Instant,
    try_join,
};

use crate::communication::{parse_action, EngineCommunication};
use crate::poker::game::{EndReason, GameState, PlayerPosition, Round};

pub async fn download_and_run<T: Into<String>, U: Into<String>, V: Into<PathBuf>>(
    bot: U,
    bot_path: V,
    bot_bucket: T,
    s3_client: &aws_sdk_s3::Client,
) -> Result<tokio::process::Child, anyhow::Error> {
    let bot_path: PathBuf = bot_path.into();
    Command::new("mount")
        .arg("-t")
        .arg("tmpfs")
        .arg("-o")
        .arg("rw,size=2G")
        .arg(format!("{}", bot_path.display()))
        .arg(&bot_path)
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .await?;
    shared::s3::download_file(
        &bot.into(),
        &bot_path.join("bot.zip"),
        &bot_bucket.into(),
        &s3_client,
    )
    .await?;

    log::debug!("Bot downloaded");
    Command::new("unzip")
        .arg(&bot_path.join("bot.zip"))
        .current_dir(&bot_path)
        .spawn()?
        .wait()
        .await?;
    log::debug!("Bot unzipped to {:?}", bot_path);

    let bot_json: BotJson = async {
        let json = fs::read_to_string(&bot_path.join("bot/bot.json")).await?;
        if let Ok(bot) = serde_json::from_str::<BotJson>(&json) {
            return Ok(bot);
        }
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Unable to parse bot.json",
        ))
    }
    .await?;
    log::debug!("Read json");

    let log_file = Stdio::from(std::fs::File::create(bot_path.join("logs")).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to create log file: {}", e),
        )
    })?);
    std::fs::write(bot_path.join("bot/run.sh"), bot_json.run).expect("write to build.sh failed");
    Command::new("chown")
        .arg("-R")
        .arg("runner:runner")
        .arg(".")
        .current_dir(&bot_path.join("bot"))
        .status()
        .await?;
    Command::new("chmod")
        .arg("+x")
        .arg("run.sh")
        .current_dir(&bot_path.join("bot"))
        .status()
        .await?;

    Ok(Command::new("bwrap")
        .args([
            "--unshare-all",
            "--die-with-parent",
            "--dir",
            "/tmp",
            "--ro-bind",
            "/usr",
            "/usr",
            "--proc",
            "/proc",
            "--dev",
            "/dev",
            "--ro-bind",
            "/lib",
            "/lib",
            "--ro-bind",
            "/usr/bin",
            "/usr/bin",
            "--ro-bind",
            "/bin",
            "/bin",
            "--bind",
            ".",
            "/home/runner",
            "--chdir",
            "/home/runner",
            "./run.sh",
        ])
        .current_dir(&bot_path.join("bot"))
        .uid(1000)
        .gid(1000)
        .process_group(0)
        .stderr(log_file)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?)
}
extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
}

pub struct GameResult {
    pub status: Result<shared::GameStatus, GameError>,
    pub defender_log: Vec<u8>,
    pub challenger_log: Vec<u8>,
    pub public_log: Vec<u8>,
}

pub async fn run_game(
    defender: i32,
    challenger: i32,
    s3_client: &aws_sdk_s3::Client,
    task_id: &String,
    rounds: usize,
) -> Result<GameResult, anyhow::Error> {
    // create tmp directory
    // doesn't have the same id as the task
    let game_id = format!("{:x}", rand::thread_rng().gen::<u32>());
    let tmp_dir = Path::new("/tmp").join(&game_id);
    log::debug!("Playing {} against {}", defender, challenger);
    log::info!("Running game {} with local id {}", task_id, game_id);
    let bot_bucket = std::env::var("COMPILED_BOT_S3_BUCKET")?;
    log::debug!("Bot bucket: {}", bot_bucket);

    // download bots from s3
    log::debug!("Making bot directories");
    let defender_path = tmp_dir.join("defender");
    fs::create_dir_all(&defender_path.clone()).await?;
    let challenger_path = tmp_dir.join("challenger");
    fs::create_dir_all(&challenger_path.clone()).await?;
    log::debug!("Downloading bots from aws");
    let (defender, challenger) = try_join!(
        download_and_run(
            defender.to_string(),
            defender_path.clone(),
            &bot_bucket,
            s3_client
        ),
        download_and_run(
            challenger.to_string(),
            challenger_path.clone(),
            &bot_bucket,
            s3_client
        )
    )?;

    // run game
    let mut game = Game::new(
        defender,
        challenger,
        game_id,
        Duration::from_secs(1),
        tokio::fs::File::create(tmp_dir.join("logs")).await?,
    );

    let status = game.play(rounds).await;
    game.drop().await?;
    // TODO: issues reading the logs probably shouldn't cause an internal error
    let defender_log = tokio::fs::read(tmp_dir.join("defender/logs")).await?;
    let challenger_log = tokio::fs::read(tmp_dir.join("challenger/logs")).await?;
    let public_log = tokio::fs::read(tmp_dir.join("logs")).await?;
    Command::new("umount")
        .arg(format!("{}", challenger_path.display()))
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .await?;

    Command::new("umount")
        .arg(format!("{}", defender_path.display()))
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .await?;

    fs::remove_dir_all(tmp_dir).await?;
    Ok(GameResult {
        status,
        defender_log,
        challenger_log,
        public_log,
    })
}

pub struct Game {
    defender: tokio::process::Child,
    challenger: tokio::process::Child,
    stacks: [u32; 2],
    initial_stacks: [u32; 2],
    sb: WhichBot,
    id: String,
    timeout: Duration,
    logs: tokio::fs::File,
    start_time: Instant,
    // I suck at this :'(
    cleaned_up: bool,
}
impl Game {
    pub fn new(
        defender: tokio::process::Child,
        challenger: tokio::process::Child,
        id: String,
        timeout: Duration,
        logs: tokio::fs::File,
    ) -> Self {
        Self {
            defender,
            challenger,
            stacks: [50, 50],
            initial_stacks: [50, 50],
            sb: WhichBot::Defender,
            timeout,
            id,
            logs,
            start_time: Instant::now(),
            cleaned_up: false,
        }
    }

    async fn write_bot(
        &mut self,
        which_bot: WhichBot,
        message: &EngineCommunication,
    ) -> Result<(), GameError> {
        let message: String =
            message.render_for_bot(which_bot, self.get_position_from_bot(which_bot));
        self.write_log(format!("{} < {}", which_bot, message.clone()))
            .await?;
        let bot = match which_bot {
            WhichBot::Defender => &mut self.defender,
            WhichBot::Challenger => &mut self.challenger,
        };
        if let Some(ref mut stdin) = bot.stdin {
            stdin
                .write_all(format!("{}\n", message).as_bytes())
                .await
                .map_err(|_| {
                    log::error!("Error writing to bot");
                    GameError::RunTimeError(which_bot)
                })?;

            stdin.flush().await.map_err(|_| {
                log::error!("Error writing to bot");
                GameError::RunTimeError(which_bot)
            })?;
            Ok(())
        } else {
            // TODO: determine cause close
            self.logs
                .write_all(
                    format!(
                        "{}ms System >>> Ending because {} lost stdin\n",
                        tokio::time::Instant::now()
                            .duration_since(self.start_time)
                            .as_millis(),
                        which_bot
                    )
                    .as_bytes(),
                )
                .await
                .map_err(|_| GameError::RunTimeError(which_bot))?;
            // TODO: determine cause of close
            self.write_log(format!("System > Ending because {} lost stdin", which_bot))
                .await?;
            Err(GameError::RunTimeError(which_bot))
        }
    }

    async fn write_log<S: Into<String>>(&mut self, msg: S) -> Result<(), shared::GameError> {
        self.logs
            .write_all(
                format!(
                    "{}ms {}\n",
                    tokio::time::Instant::now()
                        .duration_since(self.start_time)
                        .as_millis(),
                    msg.into()
                )
                .as_bytes(),
            )
            .await
            .map_err(|e| {
                log::error!("Error writing to log: {}", e);
                shared::GameError::InternalError
            })?;
        Ok(())
    }

    async fn write_bots(&mut self, message: EngineCommunication) -> Result<(), GameError> {
        self.write_bot(WhichBot::Defender, &message).await?;
        self.write_bot(WhichBot::Challenger, &message).await?;
        Ok(())
    }

    async fn play_round(
        &mut self,
        defender_reader: &mut BufReader<ChildStdout>,
        challenger_reader: &mut BufReader<ChildStdout>,
    ) -> Result<GameState, shared::GameError> {
        let mut rng = thread_rng();
        let mut state = crate::poker::game::GameState::new(
            match self.sb {
                WhichBot::Defender => [self.stacks[0], self.stacks[1]],
                WhichBot::Challenger => [self.stacks[1], self.stacks[0]],
            },
            GameState::get_shuffled_deck(&mut rng),
        );

        //log::debug!("Game state: {:?}. ", state);

        let mut round = None;

        self.write_bots(EngineCommunication::StartGame { sb: self.sb });

        while !state.round_over() {
            // Print community cards to both bots
            if round != Some(state.round) {
                //log::debug!("Printing community cards.");
                round = Some(state.round);
                match round {
                    Some(Round::PreFlop) => {
                        self.write_bots(EngineCommunication::PreFlopCards(
                            state.player_states[0].hole_cards,
                            state.player_states[1].hole_cards,
                        ))
                        .await?;
                    }
                    Some(Round::Flop) => {
                        self.write_bots(EngineCommunication::FlopCards([
                            state.community_cards[0],
                            state.community_cards[1],
                            state.community_cards[2],
                        ]))
                        .await?;
                    }
                    Some(Round::River) => {
                        self.write_bots(EngineCommunication::RiverCard(state.community_cards[3]))
                            .await?;
                    }
                    Some(Round::Turn) => {
                        self.write_bots(EngineCommunication::TurnCard(state.community_cards[4]))
                            .await?;
                    }
                    _ => Err(GameError::InternalError)?,
                }
            }
            let whose_turn: WhichBot =
                self.get_bot_from_position(state.whose_turn().ok_or(GameError::InternalError)?);

            let (target_reader, opponent_gid) = match whose_turn {
                WhichBot::Defender => (
                    &mut *defender_reader,
                    self.challenger.id().unwrap_or_default(),
                ),
                WhichBot::Challenger => (
                    &mut *challenger_reader,
                    self.defender.id().unwrap_or_default(),
                ),
            };

            unsafe {
                let status = kill(-(opponent_gid as i32), 19);
                self.write_log(format!(
                    "Sleeping process group {}, status: {}",
                    opponent_gid, status
                ))
                .await?;
            };
            // write current game state to the bots stream
            //log::debug!("Writing current state.");
            self.write_bots(EngineCommunication::get_betting_state(&state))
                .await
                .map_err(|_| {
                    unsafe { kill(-(opponent_gid as i32), 18) };
                    log::info!("Failed to write current state to bot {:?}.", whose_turn);
                    GameError::RunTimeError(whose_turn)
                })?;

            //log::debug!("Reading action from {:?}.", whose_turn);
            let mut line: String = Default::default();
            tokio::time::timeout(self.timeout, target_reader.read_line(&mut line))
                .await
                .map_err(|_| shared::GameError::TimeoutError(whose_turn))?
                .map_err(|_| shared::GameError::RunTimeError(whose_turn))?;

            self.write_log(format!("{} > {}", whose_turn, line.trim()))
                .await?;
            //log::debug!("Reading action from {:?}.", line);
            state = state
                .post_action(
                    parse_action(line.trim())
                        .map_err(|_| shared::GameError::InvalidActionError(whose_turn.clone()))?,
                )
                .map_err(|_| shared::GameError::InvalidActionError(whose_turn.clone()))?;

            unsafe {
                let status = kill(-(opponent_gid as i32), 18);
                self.write_log(format!(
                    "Waking up process group {}, status: {}",
                    opponent_gid, status
                ))
                .await?;
            };
        }

        Ok(state)
    }

    /// Play a game of poker, returning a [shared::GameResult]
    pub async fn play(&mut self, rounds: usize) -> Result<shared::GameStatus, GameError> {
        log::debug!("Playing game {} with {} rounds", self.id, rounds);
        let mut defender_reader = BufReader::new(
            self.defender
                .stdout
                .take()
                .ok_or(GameError::RunTimeError(WhichBot::Defender))?,
        );
        let mut challenger_reader = BufReader::new(
            self.challenger
                .stdout
                .take()
                .ok_or(GameError::RunTimeError(WhichBot::Challenger))?,
        );

        log::info!("Clients connected for {}", self.id);
        for i in 0..rounds {
            if self.stacks[0] == 0 || self.stacks[1] == 0 {
                self.write_log(format!("System > Ending because a bot has an empty stack"))
                    .await?;
                break;
            }
            self.write_log(format!("System > round {}/{}", i + 1, rounds))
                .await?;
            //log::debug!("Playing round. Current stacks: {:?}.", self.stacks);
            match self
                .play_round(&mut defender_reader, &mut challenger_reader)
                .await
            {
                Err(e) => {
                    self.write_log(format!("System > {:?}", e)).await?;
                    Err(e)?;
                }
                Ok(state) => {
                    self.write_bots(EngineCommunication::get_round_end(&state))
                        .await?;

                    self.stacks = [
                        state.player_states
                            [self.get_position_from_bot(WhichBot::Defender) as usize]
                            .stack,
                        state.player_states
                            [self.get_position_from_bot(WhichBot::Challenger) as usize]
                            .stack,
                    ];
                }
            }
            self.sb = self.sb.other();
        }
        return Ok(shared::GameStatus::ScoreChanged(
            i32::try_from(self.stacks[0]).unwrap() - i32::try_from(self.initial_stacks[0]).unwrap(),
            i32::try_from(self.stacks[1]).unwrap() - i32::try_from(self.initial_stacks[1]).unwrap(),
        ));
    }

    fn get_bot_from_position(&self, position: PlayerPosition) -> WhichBot {
        match position {
            PlayerPosition::SmallBlind => self.sb,
            PlayerPosition::BigBlind => self.sb.other(),
        }
    }

    fn get_position_from_bot(&self, which_bot: WhichBot) -> PlayerPosition {
        let other = which_bot.other();
        match self.sb {
            which_bot => PlayerPosition::SmallBlind,
            other => PlayerPosition::BigBlind,
        }
    }

    pub async fn drop(&mut self) -> Result<(), anyhow::Error> {
        if let Some(id) = self.defender.id() {
            unsafe {
                kill(-(id as i32), 18);
            }
        }
        if let Some(id) = self.challenger.id() {
            unsafe {
                kill(-(id as i32), 18);
            }
        }

        self.defender.kill().await?;
        self.challenger.kill().await?;
        self.cleaned_up = true;
        Ok(())
    }
}
impl Drop for Game {
    fn drop(&mut self) {
        if !self.cleaned_up {
            panic!("Game dropped without manually calling drop")
        }
    }
}
