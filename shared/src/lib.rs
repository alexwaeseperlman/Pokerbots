pub mod s3;
pub mod sqs;
use std::{fmt::Display, io, str::FromStr};

use aws_config::SdkConfig;
use aws_sdk_s3::config::Credentials;
use reqwest::header::{HeaderMap, HeaderName};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[macro_use]
extern crate num_derive;

#[cfg(feature = "db")]
pub mod db;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct SerializableHeaderMap(Vec<(String, String)>);

impl From<&HeaderMap> for SerializableHeaderMap {
    fn from(map: &HeaderMap) -> Self {
        Self(
            map.into_iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
                .filter(|(k, _v)| k != "")
                .collect(),
        )
    }
}

impl Into<HeaderMap> for SerializableHeaderMap {
    fn into(self) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (k, v) in self.0 {
            if let Ok(k) = HeaderName::from_str(&k) {
                if let Ok(v) = v.parse() {
                    map.insert(k, v);
                }
            }
        }
        map
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct PresignedRequest {
    pub url: String,
    pub headers: SerializableHeaderMap,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct BuildTask {
    pub bot: String,
    pub log_presigned: PresignedRequest,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS, FromPrimitive, ToPrimitive, Copy)]
#[repr(i32)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
#[cfg_attr(feature = "db", derive(diesel::FromSqlRow, diesel::AsExpression))]
#[cfg_attr(feature="db", diesel(sql_type=diesel::sql_types::Integer))]
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
        defender: String,
        challenger: String,
        id: String,
        rounds: usize,
        public_logs_presigned: PresignedRequest,
        defender_logs_presigned: PresignedRequest,
        challenger_logs_presigned: PresignedRequest,
    },
    TestGame {
        bot: String,
        log_presigned: PresignedRequest,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, TS)]
#[repr(i32)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
pub enum WhichBot {
    Defender = 0,
    Challenger = 1,
}

impl Display for WhichBot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WhichBot::Defender => write!(f, "Defender"),
            WhichBot::Challenger => write!(f, "Challenger"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, TS)]
pub enum GameActionError {
    InvalidCheck,
    Raise0,
    GameOver,
    CouldNotParse,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
pub enum GameError {
    RunTimeError(WhichBot),
    TimeoutError(WhichBot),
    MemoryError(WhichBot),
    InvalidActionError(WhichBot),
    InternalError,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub enum GameStatus {
    ScoreChanged(i32),
    TestGameFailed,
    TestGameSucceeded,
}

pub type GameResult = Result<GameStatus, GameError>;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameStatusMessage {
    pub result: GameResult,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct BotJson {
    pub name: String,
    pub description: Option<String>,
    pub build: Option<String>,
    pub run: String,
}

impl From<io::Error> for GameError {
    fn from(e: io::Error) -> Self {
        Self::InternalError
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
