pub mod process;
pub mod sqs;
use std::io;

use aws_config::SdkConfig;
use aws_sdk_s3::config::Credentials;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[cfg(feature = "db")]
pub mod db;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct BuildTask {
    pub bot: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub enum BuildStatus {
    Unqueued = -1,
    Queued = 0,
    Building = 1,
    BuildSucceeded = 2,
    PlayingTestGame = 3,
    TestGameSucceeded = 4,
    BuildFailed = 5,
    TestGameFailed = 6,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct BuildResultMessage {
    pub status: BuildStatus,
    pub bot: String,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub enum GameTask {
    Game {
        bot_a: String,
        bot_b: String,
        id: String,
        date: i64,
        rounds: i32,
    },
    TestGame {
        bot: String,
        id: String,
        date: i64,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, TS)]
pub enum WhichBot {
    BotA,
    BotB,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, TS)]
pub enum GameActionError {
    InvalidCheck,
    Raise0,
    GameOver,
    CouldNotParse,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub enum GameError {
    RunTimeError(String, WhichBot),
    TimeoutError(String, WhichBot),
    MemoryError(String, WhichBot),
    InvalidActionError(GameActionError, WhichBot),
    InternalError(String),
}
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub enum ScoringResult {
    ScoreChanged(i32),
}

pub type GameResult = Result<ScoringResult, GameError>;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameResultMessage {
    pub result: GameResult,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct Bot {
    pub name: String,
    pub description: Option<String>,
    pub build: Option<String>,
    pub run: String,
}

impl From<io::Error> for GameError {
    fn from(e: io::Error) -> Self {
        Self::InternalError(format!("IO Error: {}", e))
    }
}

pub async fn aws_config() -> SdkConfig {
    aws_config::from_env().load().await
}

pub async fn sqs_client(conf: &aws_config::SdkConfig) -> aws_sdk_sqs::Client {
    let mut sqs_config_builder = aws_sdk_sqs::config::Builder::from(conf);
    match std::env::var("SQS_ADDRESS") {
        Ok(endpoint) => {
            sqs_config_builder = sqs_config_builder.endpoint_url(endpoint);
            return aws_sdk_sqs::Client::from_conf(sqs_config_builder.build());
        }
        Err(_) => {}
    }
    aws_sdk_sqs::Client::new(conf)
}

pub async fn s3_client(conf: &aws_config::SdkConfig) -> aws_sdk_s3::Client {
    let mut s3_config_builder = aws_sdk_s3::config::Builder::from(conf);
    match std::env::var("S3_ADDRESS") {
        Ok(endpoint) => {
            s3_config_builder = s3_config_builder
                .endpoint_url(endpoint)
                // Hours debugging to discover that this is necessary for some reason :(
                .force_path_style(true)
                .credentials_provider(Credentials::new(
                    std::env::var("S3_ACCESS_KEY").expect("S3_ACCESS_KEY not set"),
                    std::env::var("S3_SECRET_KEY").expect("S3_SECRET_KEY not set"),
                    None,
                    None,
                    "",
                ));
            return aws_sdk_s3::Client::from_conf(s3_config_builder.build());
        }
        Err(_) => {}
    }
    aws_sdk_s3::Client::new(conf)
}
