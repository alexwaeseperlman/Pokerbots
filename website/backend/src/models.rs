use serde::{Deserialize, Serialize};

use crate::schema::{bots, games, sql_types::GameError, team_invites, teams, users};

#[derive(Serialize, Deserialize, diesel::Queryable, Debug)]
pub struct Team {
    pub id: i32,
    pub team_name: String,
    pub owner: String,
    pub score: Option<i32>,
    pub active_bot: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TeamWithMembers {
    pub id: i32,
    pub team_name: String,
    pub owner: String,
    pub score: Option<i32>,
    pub active_bot: Option<i32>,
    pub members: Vec<User>,
    pub invites: Option<Vec<TeamInvite>>,
}

#[derive(diesel::Insertable, Debug)]
#[diesel(table_name = teams)]
pub struct NewTeam {
    pub team_name: String,
    pub owner: String,
}

#[derive(Serialize, Deserialize, diesel::Queryable, Debug, Clone)]
pub struct User {
    pub email: String,
    pub display_name: String,
    pub team_id: Option<i32>,
    pub is_admin: bool,
}

#[derive(diesel::Insertable, Debug)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub email: String,
    pub display_name: String,
}

#[derive(diesel::Insertable, Debug)]
#[diesel(table_name = team_invites)]
pub struct NewInvite {
    pub expires: i64,
    pub invite_code: String,
    pub teamid: i32,
}

#[derive(Serialize, Deserialize, diesel::Queryable, Debug, Clone)]
pub struct TeamInvite {
    pub invite_code: String,
    pub teamid: i32,
    pub expires: i64,
}

#[derive(diesel::Insertable, Debug)]
#[diesel(table_name = games)]
pub struct NewGame {
    pub id: String,
    pub bot_a: i32,
    pub bot_b: i32,
}

#[derive(Serialize, Deserialize, diesel::Queryable, Debug)]
pub struct Game {
    pub id: String,
    pub bot_a: i32,
    pub bot_b: i32,
    pub score_change: Option<i32>,
    pub created: i64,

    pub error_type: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, diesel::Queryable)]
pub struct Bot {
    pub id: i32,
    pub team: i32,
    pub name: String,
    pub description: Option<String>,
    pub score: f32,
    pub created: i64,
    pub uploaded_by: String,
}

#[derive(Debug, diesel::Insertable)]
#[diesel(table_name = bots)]
pub struct NewBot {
    pub team: i32,
    pub name: String,
    pub description: Option<String>,
    pub score: f32,
    pub uploaded_by: String,
}
