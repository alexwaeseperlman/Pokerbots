use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayTask {
    pub bota: String,
    pub botb: String,
    pub id: String,
    pub date: i64,
}
