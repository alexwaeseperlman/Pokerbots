use aws_sdk_s3::operation::get_object::builders::GetObjectFluentBuilder;
use rand::{thread_rng, Rng};
use shared::{GameMessage, GameResult, WhichBot};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    join,
    net::UnixStream,
    process, try_join,
};

use crate::{bots::runner::run_bot, poker::game::GameState};

pub mod bot;
pub mod runner;
pub mod sandbox;

pub async fn run_game(
    bot_a: String,
    bot_b: String,
    s3_client: &aws_sdk_s3::Client,
    task_id: String,
) -> GameResult {
    // create tmp directory
    // doesn't have the same id as the task
    let game_id = format!("{:x}", rand::thread_rng().gen::<u32>());
    let tmp_dir = Path::new("/tmp").join(&game_id);
    log::debug!("Playing {} against {}", bot_a, bot_b);
    log::debug!("Running game {} with local id {}.", task_id, game_id);
    fs::create_dir(&tmp_dir)
        .map_err(|e| shared::GameError::InternalError("Unable to make tmp dir".to_owned()))?;

    let bot_bucket = std::env::var("BOT_S3_BUCKET")
        .map_err(|e| shared::GameError::InternalError("Unable to get BOT_S3_BUCKET".to_owned()))?;

    // download bots from s3
    log::debug!("Making bot directories");
    let bot_a_path = tmp_dir.join("bot_a");
    fs::create_dir(&bot_a_path)
        .map_err(|e| shared::GameError::InternalError("Unable to make bot_a dir".to_owned()))?;
    let bot_b_path = tmp_dir.join("bot_b");
    fs::create_dir(&bot_b_path)
        .map_err(|e| shared::GameError::InternalError("Unable to make bot_b dir".to_owned()))?;
    log::debug!("Downloading bots from aws");
    try_join!(
        bot::download_bot(&bot_a, &bot_a_path, &bot_bucket, s3_client.clone()),
        bot::download_bot(&bot_b, &bot_b_path, &bot_bucket, s3_client.clone())
    )?;
    log::debug!("Bots downloaded");

    // run game
    let mut game = Game::new(bot_a_path, bot_b_path, game_id);

    game.play(100, task_id).await
}

pub struct Game {
    bot_a_path: PathBuf,
    bot_b_path: PathBuf,
    stacks: [u32; 2],
    initial_stacks: [u32; 2],
    button: usize,
    id: String,
}
impl Game {
    pub fn new<A: AsRef<Path>, B: AsRef<Path>>(bot_a_path: A, bot_b_path: B, id: String) -> Self {
        Self {
            bot_a_path: PathBuf::from(bot_a_path.as_ref()),
            bot_b_path: PathBuf::from(bot_b_path.as_ref()),
            stacks: [50, 50],
            initial_stacks: [50, 50],
            button: 0,
            id,
        }
    }

    async fn play_round(
        &mut self,
        reader_a: &mut BufReader<process::ChildStdout>,
        reader_b: &mut BufReader<process::ChildStdout>,
        writer_a: &mut BufWriter<process::ChildStdin>,
        writer_b: &mut BufWriter<process::ChildStdin>,
    ) -> Result<(), shared::GameError> {
        let mut rng = thread_rng();
        let mut stacks = self.stacks;
        if self.button == 1 {
            stacks = [stacks[1], stacks[0]];
        }
        let mut state =
            crate::poker::game::GameState::new(&stacks, GameState::get_shuffled_deck(&mut rng));

        log::debug!("Game state: {:?}. ", state);

        loop {
            self.stacks = if self.button == 1 {
                [state.get_stack(true), state.get_stack(false)]
            } else {
                [state.get_stack(false), state.get_stack(true)]
            };

            if state.round_over() {
                break;
            }
            let whose_turn: WhichBot = if state.whose_turn() == Some(self.button) {
                WhichBot::BotA
            } else {
                WhichBot::BotB
            };
            let (target_reader, target_writer) = if state.whose_turn() == Some(self.button) {
                (&mut *reader_a, &mut *writer_a)
            } else {
                (&mut *reader_b, &mut *writer_b)
            };

            // write current game state to the bots stream
            //writer_b.write_all()

            let mut line: String = Default::default();
            let len = tokio::time::timeout(
                std::time::Duration::from_millis(1000),
                target_reader.read_line(&mut line),
            )
            .await
            .map_err(|e| shared::GameError::TimeoutError(format!("{}", e), whose_turn.clone()))?
            .map_err(|e| shared::GameError::RunTimeError(format!("{}", e), whose_turn.clone()))?;
            state = state
                .post_action(parse_action(&line).map_err(|e| {
                    shared::GameError::InvalidActionError(
                        shared::GameActionError::CouldNotParse,
                        whose_turn.clone(),
                    )
                })?)
                .map_err(|e| shared::GameError::InvalidActionError(e, whose_turn.clone()))?;
        }

        Ok(())
    }
    /// Play a game of poker, returning a [shared::GameResult]
    pub async fn play(&mut self, rounds: usize, task_id: String) -> shared::GameResult {
        log::debug!("Playing game {} with {} rounds", self.id, rounds);
        let mut proc_a = run_bot(
            self.bot_a_path.clone(),
            |command| command.stdin(Stdio::piped()).stdout(Stdio::piped()),
            WhichBot::BotA,
        )
        .await?;
        let mut reader_a = BufReader::new(proc_a.stdout.take().ok_or(
            shared::GameError::InternalError("Failed to get stdout of bot a".to_owned()),
        )?);
        let mut writer_a = BufWriter::new(proc_a.stdin.take().ok_or(
            shared::GameError::InternalError("Failed to get stdin of bot a".to_owned()),
        )?);

        let mut proc_b = run_bot(
            self.bot_b_path.clone(),
            |command| command.stdin(Stdio::piped()).stdout(Stdio::piped()),
            WhichBot::BotB,
        )
        .await?;

        let mut reader_b = BufReader::new(proc_b.stdout.take().ok_or(
            shared::GameError::InternalError("Failed to get stdout of bot b".to_owned()),
        )?);
        let mut writer_b = BufWriter::new(proc_b.stdin.take().ok_or(
            shared::GameError::InternalError("Failed to get stdin of bot b".to_owned()),
        )?);
        log::info!("Clients connected for {}", self.id);
        for _ in 0..rounds {
            log::debug!("Playing round. Current stacks: {:?}.", self.stacks);
            self.play_round(&mut reader_a, &mut reader_b, &mut writer_a, &mut writer_b)
                .await?;
            self.button = 1 - self.button;
        }
        Ok(shared::ScoringResult::ScoreChanged(
            i32::try_from(self.stacks[0]).unwrap() - i32::try_from(self.initial_stacks[0]).unwrap(),
        ))
    }
}

fn parse_action(line: &String) -> Result<crate::poker::game::Action, shared::GameActionError> {
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
