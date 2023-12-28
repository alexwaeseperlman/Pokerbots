use aws_sdk_s3::primitives::ByteStreamError;
use futures_lite::AsyncReadExt;
use itertools::Itertools;

use rand::{thread_rng, Rng};
use shared::db::models::GameStateSQL;
use shared::{BotJson, GameError, WhichBot};
use std::process::ChildStderr;
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWriteExt};
use tokio::{
    fs,
    io::{self, BufReader},
    process::{ChildStdout, Command},
    time::Instant,
    try_join,
};

use crate::communication::{parse_action, EngineCommunication};
use crate::poker::game::{GameState, PlayerPosition, Round};

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
        .stderr(Stdio::piped())
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
    pub game_record: Vec<u8>,
}

async fn write_std_err(
    stderr: tokio::process::ChildStderr,
    bot_path: PathBuf,
    start_time: Instant,
) -> Result<(), io::Error> {
    let mut reader = io::BufReader::new(stderr);
    let mut buffer = String::new();
    let mut stderr_file = tokio::fs::File::create(bot_path.join("logs")).await?;

    reader.buffer().read_to_string(&mut buffer).await?;

    while let Ok(n) = reader.read_line(&mut buffer).await {
        if n == 0 {
            break;
        }

        let timestamp = tokio::time::Instant::now()
            .duration_since(start_time)
            .as_millis();
        let formatted_line = format!("[{}] {}", timestamp, buffer);
        stderr_file.write_all(formatted_line.as_bytes()).await?;

        buffer.clear();
    }

    Ok(())
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
    let (mut defender, mut challenger) = try_join!(
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

    let def_stderr = defender.stderr.take().unwrap();
    let chall_stderr = challenger.stderr.take().unwrap();

    let start_time = Instant::now();
    let def_handle = tokio::spawn(write_std_err(def_stderr, defender_path.clone(), start_time));
    let chall_handle = tokio::spawn(write_std_err(
        chall_stderr,
        challenger_path.clone(),
        start_time,
    ));

    // Do other non-blocking tasks here...

    // Await the completion of the task if needed
    let mut game = Game::new(
        defender,
        challenger,
        game_id.clone(),
        Duration::from_secs(1),
        tokio::fs::File::create(tmp_dir.join("logs")).await?,
        start_time,
        tokio::fs::File::create(tmp_dir.join("game_record")).await?,
    );

    let status = game.play(rounds).await;
    game.drop().await?;
    // TODO: issues reading the logs probably shouldn't cause an internal error
    def_handle.await;
    chall_handle.await;
    let defender_log = tokio::fs::read(tmp_dir.join("defender/logs")).await?;
    let challenger_log = tokio::fs::read(tmp_dir.join("challenger/logs")).await?;
    let public_log = tokio::fs::read(tmp_dir.join("logs")).await?;
    let game_record = tokio::fs::read(tmp_dir.join("game_record")).await?;
    Command::new("umount")
        .arg("-l")
        .arg(format!("{}", challenger_path.display()))
        .stdout(Stdio::null())
        .status()
        .await?;

    Command::new("umount")
        .arg("-l")
        .arg(format!("{}", defender_path.display()))
        .stdout(Stdio::null())
        .status()
        .await?;

    fs::remove_dir_all(tmp_dir).await?;
    Ok(GameResult {
        status,
        defender_log,
        challenger_log,
        public_log,
        game_record,
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
    game_record: tokio::fs::File,
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
        start_time: Instant,
        game_record: tokio::fs::File,
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
            game_record,
            start_time,
            cleaned_up: false,
        }
    }

    async fn write_bot(
        &mut self,
        which_bot: WhichBot,
        message: &EngineCommunication,
    ) -> Result<(), GameError> {
        let message: String = message.render_for_bot(self.get_position_from_bot(which_bot));
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

    // fn encode<S>(&self, serializer: S, game_state: &GameState) -> Result<S::Ok, S::Error>
    // where
    //     S: serde::Serializer,
    // {
    //     let mut state = serializer.serialize_struct("GameState", 7)?;
    //     state.serialize_field("player_states", &game_state.player_states)?;
    //     state.serialize_field("community_cards", &game_state.community_cards)?;
    //     state.serialize_field("round", &game_state.round)?;
    //     state.serialize_field("last_aggressor", &game_state.last_aggressor)?;
    //     state.serialize_field("target_push", &game_state.target_push)?;
    //     state.serialize_field("end_reason", &game_state.end_reason)?;
    //     state.serialize_field("sb", &self.sb)?;
    //     state.end()
    // }

    async fn save_round(&mut self, state: &GameState, step: i32) -> Result<(), shared::GameError> {
        let (defender_state, challenger_state) = match self.sb {
            WhichBot::Defender => (&state.player_states[0], &state.player_states[1]),
            WhichBot::Challenger => (&state.player_states[1], &state.player_states[0]),
        };
        let game_state_sql = GameStateSQL {
            game_id: "".to_string(),
            step: step,
            defender_stack: defender_state.stack as i32,
            challenger_stack: challenger_state.stack as i32,
            defender_pushed: defender_state.pushed as i32,
            challenger_pushed: challenger_state.pushed as i32,
            defender_hand: defender_state.hole_cards.map(|c| c.to_string()).join(" "),
            challenger_hand: challenger_state.hole_cards.map(|c| c.to_string()).join(" "),
            flop: state
                .community_cards
                .get(0..3)
                .map(|v| v.iter().map(|c| c.to_string()).join(" ")),
            turn: state.community_cards.get(3).map(|c| c.to_string()),
            river: state.community_cards.get(4).map(|c| c.to_string()),
            button: self.sb.other().to_string(),
            sb: self.sb.to_string(),
            action_time: 0,
            last_action: state.last_aggressor.to_string(),
        };
        match serde_json::to_string(&game_state_sql) {
            Ok(json_str) => {
                let _ = self
                    .game_record
                    .write_all(format!("{}\n", json_str).as_bytes())
                    .await
                    .map_err(|e| {
                        log::error!("Error while writting game state to file: {}", e);
                        shared::GameError::InternalError
                    });
                Ok(())
            }
            Err(e) => {
                log::error!("Error converting game state to json: {}", e);
                Err(shared::GameError::InternalError)
            }
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
        state_id: &mut i32,
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

        self.write_bots(EngineCommunication::StartGame).await?;

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
                    Some(Round::Turn) => {
                        self.write_bots(EngineCommunication::TurnCard(state.community_cards[3]))
                            .await?;
                    }
                    Some(Round::River) => {
                        self.write_bots(EngineCommunication::RiverCard(state.community_cards[4]))
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
            };
            // write current game state to the bots stream
            //log::debug!("Writing current state.");
            self.write_bot(whose_turn, &EngineCommunication::get_betting_state(&state))
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
                kill(-(opponent_gid as i32), 18);
            };
            self.save_round(&state, *state_id).await?;
            *state_id += 1;
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
        let mut state_id: i32 = 0;
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
                .play_round(&mut defender_reader, &mut challenger_reader, &mut state_id)
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
        if self.sb == which_bot {
            PlayerPosition::SmallBlind
        } else {
            PlayerPosition::BigBlind
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
