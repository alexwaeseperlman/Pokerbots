use serde::{Deserialize, Serialize};

pub mod data;
pub mod games;
pub mod manage_team;
pub mod signout;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ServerMessage {
    pub message: String,
    pub message_type: String,
}
