use aws_sdk_s3::operation::get_object::builders::GetObjectFluentBuilder;
use rand::{thread_rng, Rng};
use shared::{GameMessage, GameResult, WhichBot};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{self, Command},
};
use tokio::net::UnixStream;

use crate::poker::game::GameState;

pub mod bot;
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
    fs::create_dir(&tmp_dir)
        .map_err(|e| shared::GameError::InternalError("Unable to make tmp dir".to_owned()))?;

    let bot_bucket = std::env::var("BOT_S3_BUCKET")
        .map_err(|e| shared::GameError::InternalError("Unable to get BOT_S3_BUCKET".to_owned()))?;

    // download bots from s3
    let bot_a_path = tmp_dir.join("bot_a");
    fs::create_dir(&bot_a_path)
        .map_err(|e| shared::GameError::InternalError("Unable to make bot_a dir".to_owned()))?;
    let bot_b_path = tmp_dir.join("bot_b");
    fs::create_dir(&bot_b_path)
        .map_err(|e| shared::GameError::InternalError("Unable to make bot_b dir".to_owned()))?;
    let bot_a = bot::download_bot(&bot_a, &bot_a_path, &bot_bucket, s3_client.clone()).await?;
    let bot_b = bot::download_bot(&bot_b, &bot_b_path, &bot_bucket, s3_client.clone()).await?;

    // make a server and socket files
    let socket_path_a = tmp_dir.join("a.sock");
    let socket_path_b = tmp_dir.join("b.sock");

    // run game
    let runner_a = Command::new("runner")
        .arg("--socket-path")
        .arg(&socket_path_a)
        .arg("--bot-path")
        .arg("bot.zip")
        .current_dir(bot_a_path)
        .spawn()?;
    let runner_b = Command::new("runner")
        .arg("--socket-path")
        .arg(&socket_path_b)
        .arg("--bot-path")
        .arg("bot.zip")
        .current_dir(bot_b_path)
        .spawn()?;

    let game = Game::new(
        socket_path_a.to_path_buf(),
        socket_path_b.to_path_buf(),
        runner_a,
        runner_b,
        game_id,
    );

    GameResult::Err(shared::GameError::InternalError(
        "This hasn't been implement yet".to_owned(),
    ))
}

pub struct Game {
    pub socket_path_a: PathBuf,
    pub socket_path_b: PathBuf,
    pub runner_a: process::Child,
    pub runner_b: process::Child,
    pub stacks: [u32; 2],
    pub initial_stacks: [u32; 2],
    pub button: usize,
    pub id: String,
}
impl Game {
    pub fn new(
        socket_path_a: PathBuf,
        socket_path_b: PathBuf,
        runner_a: process::Child,
        runner_b: process::Child,
        id: String,
    ) -> Self {
        Self {
            socket_path_a,
            socket_path_b,
            runner_a,
            runner_b,
            stacks: [50, 50],
            initial_stacks: [50, 50],
            button: false,
            id,
        }
    }

    async fn play_round(&mut self) -> Result<(), shared::GameError> {
        let mut rng = thread_rng();
        let mut stacks = self.stacks;
        if self.button == 1 {
            stacks = [stacks[1], stacks[0]];
        }
        let mut state =
            crate::poker::game::GameState::new(&stacks, GameState::get_shuffled_deck(&mut rng));
        let mut stream_a = UnixStream::connect(&self.socket_path_a).await?;
        let mut stream_b = UnixStream::connect(&self.socket_path_b).await?;

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
            let target_stream = if state.whose_turn() == Some(self.button) {
                &mut stream_a
            } else {
                &mut stream_b
            };

            let target_process = if state.whose_turn() == Some(self.button) {
                &mut self.runner_a
            } else {
                &mut self.runner_b
            };

            if target_process.try_wait()?.is_some() {
                return Err(shared::GameError::RunTimeError(
                    "Bot process exited early".to_owned(),
                    whose_turn,
                ));
            }
            // write current game state to the bots stream

            // read and parse line from bot
            let mut buf = [0; 1024];
            let mut read = target_stream.read(&mut buf).await?;
        }

        Ok(())
    }
    pub async fn play(&mut self, rounds: usize, task_id: String) -> shared::GameResult {
        for i in 0..rounds {
            self.play_round().await?;
        }
        Ok(GameMessage {
            score: shared::ScoringResult::ScoreChanged(
                i32::try_from(self.stacks[0]).unwrap()
                    - i32::try_from(self.initial_stacks[0]).unwrap(),
            ),
            id: task_id,
        })
    }
}
