use itertools::Itertools;
use rand::{thread_rng, Rng};
use shared::{BotJson, GameError, GameResult, WhichBot};
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

use crate::poker::game::GameState;

pub mod sandbox;
pub async fn download_and_run<T: Into<String>, U: Into<String>, V: Into<PathBuf>>(
    bot: U,
    bot_path: V,
    bot_bucket: T,
    s3_client: &aws_sdk_s3::Client,
) -> Result<tokio::process::Child, GameError> {
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

    Command::new("bwrap")
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
        .spawn()
        .map_err(|e| {
            log::error!("Error running bot: {}", e);
            GameError::InternalError
        })
}
extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
}

pub async fn run_game(
    defender: &String,
    challenger: &String,
    s3_client: &aws_sdk_s3::Client,
    task_id: &String,
    rounds: usize,
    game_path: &mut PathBuf,
) -> GameResult {
    // create tmp directory
    // doesn't have the same id as the task
    let game_id = format!("{:x}", rand::thread_rng().gen::<u32>());
    let tmp_dir = Path::new("/tmp").join(&game_id);
    *game_path = tmp_dir.clone();
    log::debug!("Playing {} against {}", defender, challenger);
    log::info!("Running game {} with local id {}", task_id, game_id);
    let bot_bucket = std::env::var("COMPILED_BOT_S3_BUCKET").map_err(|e| {
        log::error!("Error getting COMPILED_BOT_S3_BUCKET: {}", e);
        GameError::InternalError
    })?;
    log::debug!("Bot bucket: {}", bot_bucket);

    // download bots from s3
    log::debug!("Making bot directories");
    let defender_path = tmp_dir.join("defender");
    fs::create_dir_all(&defender_path).await.map_err(|e| {
        log::error!("Error creating defender directory: {}", e);
        shared::GameError::InternalError
    })?;
    let challenger_path = tmp_dir.join("challenger");
    fs::create_dir_all(&challenger_path).await.map_err(|e| {
        log::error!("Error creating challenger directory: {}", e);
        shared::GameError::InternalError
    })?;
    log::debug!("Downloading bots from aws");
    let (defender, challenger) = try_join!(
        download_and_run(defender, defender_path, &bot_bucket, s3_client),
        download_and_run(challenger, challenger_path, &bot_bucket, s3_client)
    )?;

    // run game
    let mut game = Game::new(
        defender,
        challenger,
        game_id,
        Duration::from_secs(1),
        tokio::fs::File::create(tmp_dir.join("logs")).await?,
    );

    game.play(rounds).await
}

pub struct Game {
    defender: tokio::process::Child,
    challenger: tokio::process::Child,
    stacks: [u32; 2],
    initial_stacks: [u32; 2],
    button: usize,
    id: String,
    timeout: Duration,
    logs: tokio::fs::File,
    start_time: Instant,
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
            button: 0,
            timeout,
            id,
            logs,
            start_time: Instant::now(),
        }
    }

    async fn write_bot<T: Into<String>>(
        &mut self,
        which_bot: WhichBot,
        message: T,
    ) -> Result<(), GameError> {
        let message: String = message.into();
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
                .await?;
            // TODO: determine cause of close
            self.write_log(format!("System > Ending because {} lost stdin", which_bot))
                .await?;
            Err(GameError::RunTimeError(which_bot))
        }
    }

    async fn print_position(&mut self, which_bot: WhichBot) -> Result<(), GameError> {
        let position = format!(
            "P {}",
            match which_bot {
                WhichBot::Defender => self.button,
                WhichBot::Challenger => (self.button + 1) % 2,
            }
        );
        self.write_bot(which_bot, position).await?;
        Ok(())
    }

    async fn print_round_end(&mut self, which_bot: WhichBot) -> Result<(), shared::GameError> {
        self.write_bot(which_bot, "E").await?;
        Ok(())
    }

    async fn print_cards(
        &mut self,
        which_bot: WhichBot,
        state: &GameState,
    ) -> Result<(), shared::GameError> {
        let cards = [
            state.player_states[match which_bot {
                WhichBot::Defender => self.button,
                WhichBot::Challenger => 1 - self.button,
            }]
            .hole_cards
            .clone(),
            state.community_cards.clone(),
        ]
        .concat()
        .iter()
        .map(|card| format!("{}", card))
        .join(" ");
        self.write_bot(which_bot, format!("C {}", cards)).await?;

        Ok(())
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
            .await?;
        Ok(())
    }

    async fn play_round(
        &mut self,
        defender_reader: &mut BufReader<ChildStdout>,
        challenger_reader: &mut BufReader<ChildStdout>,
    ) -> Result<(), shared::GameError> {
        let mut rng = thread_rng();
        let mut state = crate::poker::game::GameState::new(
            if self.button == 1 {
                [self.stacks[1], self.stacks[0]]
            } else {
                [self.stacks[0], self.stacks[1]]
            },
            GameState::get_shuffled_deck(&mut rng),
        );

        log::debug!("Game state: {:?}. ", state);

        let mut round = None;

        self.print_position(WhichBot::Defender).await.map_err(|_| {
            log::info!("Failed to print position to bot A.");
            GameError::RunTimeError(WhichBot::Defender)
        })?;

        self.print_position(WhichBot::Challenger)
            .await
            .map_err(|_| {
                log::info!("Failed to print position to bot B.");
                GameError::RunTimeError(WhichBot::Challenger)
            })?;

        loop {
            self.stacks = if self.button == 1 {
                [state.get_stack(true), state.get_stack(false)]
            } else {
                [state.get_stack(false), state.get_stack(true)]
            };

            if state.round_over() {
                log::debug!("Round ended.");
                self.print_round_end(WhichBot::Defender)
                    .await
                    .map_err(|_| {
                        log::info!("Failed to print round end to bot A.");
                        GameError::RunTimeError(WhichBot::Defender)
                    })?;

                self.print_round_end(WhichBot::Challenger)
                    .await
                    .map_err(|_| {
                        log::info!("Failed to print round end to bot B.");
                        GameError::RunTimeError(WhichBot::Challenger)
                    })?;
                break;
            }
            // Print community cards to both bots
            if round != Some(state.round) {
                log::debug!("Printing community cards.");
                round = Some(state.round);
                self.print_cards(WhichBot::Defender, &state)
                    .await
                    .map_err(|_| {
                        log::info!("Failed to print community cards to bot A.");
                        GameError::RunTimeError(WhichBot::Defender)
                    })?;
                self.print_cards(WhichBot::Challenger, &state)
                    .await
                    .map_err(|_| {
                        log::info!("Failed to print community cards to bot B.");
                        GameError::RunTimeError(WhichBot::Challenger)
                    })?;
            }
            // Assume state.whose_turn() is not None
            let whose_turn: WhichBot =
                if state.whose_turn().ok_or(GameError::InternalError)? == self.button {
                    WhichBot::Defender
                } else {
                    WhichBot::Challenger
                };

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
            log::debug!("Writing current state.");
            let status = format!(
                "S {} {} {} {} {}",
                state.target_push,
                state.player_states[0].pushed,
                state.player_states[1].pushed,
                state.player_states[0].stack,
                state.player_states[1].stack,
            );
            self.write_bot(whose_turn, status).await.map_err(|_| {
                unsafe { kill(-(opponent_gid as i32), 18) };
                log::info!("Failed to write current state to bot {:?}.", whose_turn);
                GameError::RunTimeError(whose_turn)
            })?;

            log::debug!("Reading action from {:?}.", whose_turn);
            let mut line: String = Default::default();
            tokio::time::timeout(self.timeout, target_reader.read_line(&mut line))
                .await
                .map_err(|_| shared::GameError::TimeoutError(whose_turn))?
                .map_err(|_| shared::GameError::RunTimeError(whose_turn))?;

            self.write_log(format!("{} > {}", whose_turn, line.trim()))
                .await?;
            log::debug!("Reading action from {:?}.", line);
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

        Ok(())
    }

    /// Play a game of poker, returning a [shared::GameResult]
    pub async fn play(&mut self, rounds: usize) -> shared::GameResult {
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
            log::debug!("Playing round. Current stacks: {:?}.", self.stacks);
            if let Err(e) = self
                .play_round(&mut defender_reader, &mut challenger_reader)
                .await
            {
                self.write_log(format!("System > {:?}", e)).await?;
                Err(e)?;
            }
            self.button = 1 - self.button;
        }
        return Ok(shared::GameStatus::ScoreChanged(
            i32::try_from(self.stacks[0]).unwrap() - i32::try_from(self.initial_stacks[0]).unwrap(),
        ));
    }
}
impl Drop for Game {
    fn drop(&mut self) {
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
        self.defender.start_kill();
        self.challenger.start_kill();
    }
}

fn parse_action<T: AsRef<str>>(
    line: T,
) -> Result<crate::poker::game::Action, shared::GameActionError> {
    let line = line.as_ref();
    Ok(match line.as_ref() {
        "X" => crate::poker::game::Action::Check,
        "F" => crate::poker::game::Action::Fold,
        "C" => crate::poker::game::Action::Call,
        _ => {
            if line.chars().nth(0) != Some('R') {
                Err(shared::GameActionError::CouldNotParse)?;
            }
            let amount = line[1..]
                .parse::<u32>()
                .map_err(|_| shared::GameActionError::CouldNotParse)?;
            crate::poker::game::Action::Raise { amt: amount }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::parse_action;
    #[test]
    fn parse_action_check() {
        assert_eq!(
            parse_action(&"X".to_owned()).unwrap(),
            crate::poker::game::Action::Check
        );
    }

    #[test]
    fn parse_action_fold() {
        assert_eq!(
            parse_action(&"F".to_owned()).unwrap(),
            crate::poker::game::Action::Fold
        );
    }

    #[test]
    fn parse_action_call() {
        assert_eq!(
            parse_action(&"C".to_owned()).unwrap(),
            crate::poker::game::Action::Call
        );
    }

    #[test]
    fn parse_action_raise() {
        assert_eq!(
            parse_action(&"R1234".to_owned()).unwrap(),
            crate::poker::game::Action::Raise { amt: 1234 }
        );
    }

    #[test]
    fn parse_action_raise_invalid() {
        assert!(parse_action(&"R".to_owned()).is_err());
    }

    #[test]
    fn parse_action_raise_invalid2() {
        assert!(parse_action(&"R1234a".to_owned()).is_err());
    }

    #[test]
    fn parse_action_raise_invalid3() {
        assert!(parse_action(&"R-1234".to_owned()).is_err());
    }

    #[test]
    fn parse_action_raise_invalid4() {
        assert!(parse_action(&"R-1".to_owned()).is_err());
    }

    #[test]
    fn parse_action_raise_invalid5() {
        assert!(parse_action(&"R1234.0".to_owned()).is_err());
    }

    #[test]
    fn parse_action_raise_invalid6() {
        assert!(parse_action(&"B".to_owned()).is_err());
    }
}
