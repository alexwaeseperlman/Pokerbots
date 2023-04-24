use serde::{Deserialize, Serialize};

use crate::schema::{team_invites, teams, users};

#[derive(Serialize, Deserialize, diesel::Queryable, Debug)]
pub struct Team {
    pub id: i32,
    pub team_name: String,
    pub owner: String,
}

#[derive(diesel::Insertable, Debug)]
#[diesel(table_name = teams)]
pub struct NewTeam {
    pub team_name: String,
    pub owner: String,
}

#[derive(Serialize, Deserialize, diesel::Queryable, Debug)]
pub struct User {
    pub email: String,
    pub display_name: String,
    pub team_id: Option<i32>,
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

#[derive(Serialize, Deserialize, diesel::Queryable, Debug)]
pub struct TeamInvite {
    pub invite_code: String,
    pub teamid: i32,
    pub expires: i64,
}
