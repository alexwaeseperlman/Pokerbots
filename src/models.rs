use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, diesel::Queryable)]
pub struct Team {
    pub id: i32,
    pub teamname: String,
}

#[derive(Serialize, Deserialize, diesel::Queryable)]
pub struct User {
    pub email: String,
    pub displayName: String,
    pub team: Option<i32>,
}
