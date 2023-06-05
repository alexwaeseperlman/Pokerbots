use std::io;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayTask {
    pub bot_a: String,
    pub bot_b: String,
    pub id: String,
    pub date: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WhichBot {
    BotA,
    BotB,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum GameActionError {
    InvalidCheck,
    Raise0,
    GameOver,
    CouldNotParse,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameError {
    RunTimeError(String, WhichBot),
    CompileError(String, WhichBot),
    TimeoutError(String, WhichBot),
    MemoryError(String, WhichBot),
    InvalidActionError(GameActionError, WhichBot),
    InternalError(String),
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ScoringResult {
    ScoreChanged(i32),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameMessage {
    pub score: ScoringResult,
    pub id: String,
}

pub type GameResult = Result<GameMessage, GameError>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bot {
    pub name: String,
    pub description: Option<String>,
    pub build: String,
}

impl From<io::Error> for GameError {
    fn from(e: io::Error) -> Self {
        Self::InternalError(format!("IO Error: {}", e))
    }
}
