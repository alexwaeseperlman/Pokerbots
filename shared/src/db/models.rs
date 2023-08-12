use diesel::{sql_types::Integer, deserialize::FromSql, pg::{self, PgValue}, serialize::ToSql};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{db::schema::{bots, games, team_invites, teams, users}, BuildStatus};

#[derive(Serialize, Deserialize, diesel::Queryable, Debug, TS)]

pub struct Team {
    pub id: i32,
    pub team_name: String,
    pub owner: String,
    pub score: Option<i32>,
    pub active_bot: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
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

#[derive(Serialize, Deserialize, diesel::Queryable, Debug, Clone, TS)]
#[ts(export)]
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

#[derive(Serialize, Deserialize, diesel::Queryable, Debug, Clone, TS)]
#[ts(export)]
pub struct TeamInvite {
    pub invite_code: String,
    pub teamid: i32,
    pub expires: i64,
}

#[derive(diesel::Insertable, Debug)]
#[diesel(table_name = games)]
pub struct NewGame {
    pub id: String,
    pub defender: i32,
    pub challenger: i32,
}

#[derive(Serialize, Deserialize, diesel::Queryable, Debug, TS)]
#[ts(export)]
pub struct Game {
    pub id: String,
    pub defender: i32,
    pub challenger: i32,
    pub score_change: Option<i32>,
    pub created: i64,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, diesel::Queryable, TS)]
#[ts(export)]
pub struct Bot {
    pub id: i32,
    pub team: i32,
    pub name: String,
    pub description: Option<String>,
    pub score: f32,
    pub created: i64,
    pub uploaded_by: String,
    pub build_status: BuildStatus,
}

#[derive(Debug, diesel::Insertable)]
#[diesel(table_name = bots)]
pub struct NewBot {
    pub team: i32,
    pub name: String,
    pub description: Option<String>,
    pub score: f32,
    pub uploaded_by: String,
    pub build_status: BuildStatus,
}

impl ToSql<Integer, pg::Pg> for BuildStatus {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, pg::Pg>) -> diesel::serialize::Result {
        let val = *self as i32;
        ToSql::<Integer, pg::Pg>::to_sql(&val, &mut out.reborrow())
    }
}

impl FromSql<Integer, pg::Pg> for BuildStatus {
    fn from_sql(
        bytes: PgValue, 
    ) -> diesel::deserialize::Result<Self> {
        if let Some(result) = num::FromPrimitive::from_i32(i32::from_sql(bytes)?) {
            Ok(result)
        }
        else {
            Err("Invalid build status".into())
        }
    }
}