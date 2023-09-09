use diesel::{
    deserialize::FromSql,
    pg::{self, PgValue},
    prelude::{Associations, Identifiable, Insertable},
    serialize::ToSql,
    sql_types::Integer,
    AsChangeset, Queryable, Selectable,
};

use chrono;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::{
    db::schema::{auth, bots, game_results, games, team_invites, teams, user_profiles, users},
    BuildStatus, WhichBot,
};

#[derive(Serialize, Deserialize, Queryable, Debug, Selectable)]
#[cfg_attr(feature = "ts-bindings", derive(TS), ts(export))]
#[diesel(table_name = teams)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub owner: Uuid,
    pub score: Option<i32>,
    pub active_bot: Option<i32>,
    pub deleted_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "ts-bindings", derive(TS), ts(export))]
pub struct TeamWithMembers<T> {
    pub id: i32,
    pub name: String,
    pub owner: Uuid,
    pub score: Option<i32>,
    pub active_bot: Option<i32>,
    pub members: Vec<T>,
    pub invites: Option<Vec<TeamInvite>>,
    pub deleted_at: Option<i64>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = teams)]
pub struct NewTeam {
    pub name: String,
    pub owner: Uuid,
}

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, Selectable, Identifiable)]
#[cfg_attr(feature = "ts-bindings", derive(TS), ts(export))]
#[diesel(table_name = users)]
#[diesel(primary_key(id))]
pub struct User {
    pub display_name: String,
    pub team: Option<i32>,
    pub id: Uuid,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub display_name: String,
    pub id: Uuid,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = team_invites)]
pub struct NewInvite {
    pub expires: i64,
    pub code: String,
    pub team: i32,
}

#[derive(Serialize, Deserialize, Queryable, Debug, Clone)]
#[cfg_attr(feature = "ts-bindings", derive(TS), ts(export))]
pub struct TeamInvite {
    pub code: String,
    //TODO: rename this to team
    pub team: i32,
    pub expires: i64,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = games)]
pub struct NewGame {
    pub id: String,
    pub defender: i32,
    pub challenger: i32,
    pub defender_rating: f32,
    pub challenger_rating: f32,
}

#[derive(Queryable, Serialize, Deserialize, Debug, Selectable)]
#[cfg_attr(feature = "ts-bindings", derive(TS), ts(export))]
#[diesel(table_name = games)]
pub struct Game {
    pub id: String,
    pub defender: i32,
    pub challenger: i32,
    pub created: i64,
    pub defender_rating: f32,
    pub challenger_rating: f32,
}

#[derive(
    Queryable, Serialize, diesel::Identifiable, Deserialize, Debug, Selectable, Insertable,
)]
#[cfg_attr(feature = "ts-bindings", derive(TS), ts(export))]
#[diesel(belongs_to(Game))]
#[diesel(table_name = game_results)]
pub struct GameResult {
    pub id: String,
    pub challenger_rating_change: f32,
    pub defender_rating_change: f32,
    pub defender_score: i32,
    pub challenger_score: i32,
    pub error_type: Option<String>,
    pub error_bot: Option<i32>,
    pub updated_at: i64,
    pub defender_rating: f32,
    pub challenger_rating: f32,
}

#[derive(Deserialize, Debug, Selectable, Insertable)]
#[cfg_attr(feature = "ts-bindings", derive(TS), ts(export))]
#[diesel(table_name = game_results)]
pub struct NewGameResult {
    pub id: String,
    pub challenger_rating_change: f32,
    pub defender_rating_change: f32,
    pub defender_score: i32,
    pub challenger_score: i32,
    pub error_type: Option<String>,
    pub error_bot: Option<i32>,
    pub defender_rating: f32,
    pub challenger_rating: f32,
}

#[derive(Serialize)]
#[cfg_attr(feature = "ts-bindings", derive(TS), ts(export))]
pub struct GameWithResult {
    pub id: String,
    pub defender: i32,
    pub challenger: i32,
    pub created: i64,
    pub defender_rating: f32,
    pub challenger_rating: f32,
    pub result: Option<GameResult>,
}

#[derive(Serialize)]
#[cfg_attr(feature = "ts-bindings", derive(TS), ts(export))]
pub struct GameWithBotsWithResult<T> {
    pub id: String,
    pub defender: T,
    pub challenger: T,
    pub created: i64,
    pub defender_rating: f32,
    pub challenger_rating: f32,
    pub result: Option<GameResult>,
}

#[derive(Serialize, Deserialize, Debug, Queryable, Selectable)]
#[cfg_attr(feature = "ts-bindings", derive(TS), ts(export))]
#[diesel(table_name = bots)]
pub struct Bot {
    pub id: i32,
    pub team: i32,
    pub name: String,
    pub description: Option<String>,
    pub created: i64,
    pub uploaded_by: Uuid,
    pub build_status: BuildStatus,
    pub deleted_at: Option<i64>,
    pub rating: f32,
}

#[derive(Serialize, Deserialize, Debug, Queryable)]
#[cfg_attr(feature = "ts-bindings", derive(TS), ts(export))]
pub struct BotWithTeam<T> {
    pub id: i32,
    pub team: T,
    pub name: String,
    pub description: Option<String>,
    pub rating: f32,
    pub created: i64,
    pub uploaded_by: Uuid,
    pub build_status: BuildStatus,
}

impl<T> BotWithTeam<T> {
    pub fn from_bot_and_team(bot: Bot, team: T) -> Self {
        BotWithTeam {
            id: bot.id,
            team,
            name: bot.name,
            description: bot.description,
            rating: bot.rating,
            created: bot.created,
            uploaded_by: bot.uploaded_by,
            build_status: bot.build_status,
        }
    }
}

#[derive(Serialize)]
#[cfg_attr(feature = "ts-bindings", derive(TS), ts(export))]
pub struct GameWithBots<T> {
    pub id: String,
    pub defender: T,
    pub challenger: T,
    pub created: i64,
}

#[derive(Debug, diesel::Insertable)]
#[diesel(table_name = bots)]
pub struct NewBot {
    pub team: i32,
    pub name: String,
    pub description: Option<String>,
    pub rating: f32,
    pub uploaded_by: Uuid,
    pub build_status: BuildStatus,
}

#[derive(Serialize, Deserialize, Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = auth)]
pub struct Auth {
    pub email: String,
    pub mangled_password: Option<String>,
    pub email_verification_link: Option<String>,
    pub email_verification_link_expiration: Option<chrono::NaiveDateTime>,
    pub password_reset_link: Option<String>,
    pub password_reset_link_expiration: Option<chrono::NaiveDateTime>,
    pub email_confirmed: bool,
    pub is_admin: bool,
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Insertable, AsChangeset)]
#[diesel(table_name = auth)]
pub struct NewAuth {
    pub email: String,
    pub mangled_password: Option<String>,
    pub email_verification_link: Option<String>,
    pub email_verification_link_expiration: Option<chrono::NaiveDateTime>,
    pub email_confirmed: bool,
}

#[derive(
    Serialize, Deserialize, Debug, Queryable, Selectable, Insertable, Associations, Identifiable,
)]
#[cfg_attr(feature = "ts-bindings", derive(TS), ts(export))]
#[diesel(table_name = user_profiles)]
#[diesel(belongs_to(User, foreign_key = id))]
#[diesel(primary_key(id))]
pub struct UserProfile {
    pub first_name: String,
    pub last_name: String,
    pub country: Option<String>,
    pub school: String,
    pub linkedin: Option<String>,
    pub github: Option<String>,
    pub resume_s3_key: Option<String>,
    pub id: Uuid,
}

impl ToSql<Integer, pg::Pg> for BuildStatus {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, pg::Pg>,
    ) -> diesel::serialize::Result {
        let val = *self as i32;
        ToSql::<Integer, pg::Pg>::to_sql(&val, &mut out.reborrow())
    }
}

impl FromSql<Integer, pg::Pg> for BuildStatus {
    fn from_sql(bytes: PgValue) -> diesel::deserialize::Result<Self> {
        if let Some(result) = num::FromPrimitive::from_i32(i32::from_sql(bytes)?) {
            Ok(result)
        } else {
            Err("Invalid build status".into())
        }
    }
}

impl ToSql<Integer, pg::Pg> for WhichBot {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, pg::Pg>,
    ) -> diesel::serialize::Result {
        let val = *self as i32;
        ToSql::<Integer, pg::Pg>::to_sql(&val, &mut out.reborrow())
    }
}

impl FromSql<Integer, pg::Pg> for WhichBot {
    fn from_sql(bytes: PgValue) -> diesel::deserialize::Result<Self> {
        if let Some(result) = num::FromPrimitive::from_i32(i32::from_sql(bytes)?) {
            Ok(result)
        } else {
            Err("Invalid build status".into())
        }
    }
}
