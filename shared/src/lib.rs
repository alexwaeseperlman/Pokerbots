use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayTask {
    pub bot_a: String,
    pub bot_b: String,
    pub id: String,
    pub date: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameResult {
    pub id: String,
    pub score_change: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bot {
    pub name: String,
    pub description: Option<String>,
    pub build: String,
}
