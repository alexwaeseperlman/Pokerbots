use std::io::Write;

use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::{self, PgValue},
    prelude::{Associations, Identifiable, Insertable},
    serialize::ToSql,
    sql_types::{Integer, Text, VarChar},
    AsChangeset, Expression, Queryable, Selectable,
};

use chrono;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::{
    db::schema::{
        auth, bots, game_results, game_states, games, team_invites, teams, user_profiles, users,
    },
    poker::game::{Action, CommunityCards, EndReason, HoleCards, PlayerPosition},
    BuildStatus, GameError, WhichBot,
};

#[derive(Serialize, Deserialize, Queryable, Debug, Selectable, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
#[diesel(table_name = teams)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub owner: Uuid,
    pub active_bot: Option<i32>,
    pub deleted_at: Option<i64>,
    pub rating: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
pub struct TeamWithMembers<T> {
    pub id: i32,
    pub name: String,
    pub owner: Uuid,
    pub active_bot: Option<i32>,
    pub members: Vec<T>,
    pub invites: Option<Vec<TeamInvite>>,
    pub deleted_at: Option<i64>,
    pub rating: f32,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = teams)]
pub struct NewTeam {
    pub name: String,
    pub owner: Uuid,
}

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, Selectable, Identifiable, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
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

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
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
    pub rated: bool,
}

#[derive(Queryable, Serialize, Deserialize, Debug, Selectable, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
#[diesel(table_name = games)]
pub struct Game {
    pub id: String,
    pub defender: i32,
    pub challenger: i32,
    pub created: i64,
    pub defender_rating: f32,
    pub challenger_rating: f32,
    pub rated: bool,
}

#[derive(
    Queryable, Serialize, diesel::Identifiable, Deserialize, Debug, Selectable, Insertable, TS,
)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
#[diesel(belongs_to(Game))]
#[diesel(table_name = game_results)]
pub struct GameResult {
    pub id: String,
    pub challenger_rating_change: f32,
    pub defender_rating_change: f32,
    pub defender_score: i32,
    pub challenger_score: i32,
    pub error_type: Option<GameError>,
    pub updated_at: i64,
    pub defender_rating: f32,
    pub challenger_rating: f32,
}

#[derive(Deserialize, Debug, Selectable, Insertable, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
#[diesel(table_name = game_results)]
pub struct NewGameResult {
    pub id: String,
    pub challenger_rating_change: f32,
    pub defender_rating_change: f32,
    pub defender_score: i32,
    pub challenger_score: i32,
    pub error_type: Option<GameError>,
    pub defender_rating: f32,
    pub challenger_rating: f32,
}

#[derive(Serialize, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
pub struct GameWithResult {
    pub id: String,
    pub defender: i32,
    pub challenger: i32,
    pub created: i64,
    pub defender_rating: f32,
    pub challenger_rating: f32,
    pub result: Option<GameResult>,
    pub rated: bool,
}

#[derive(Serialize, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
pub struct GameWithBotsWithResult<T> {
    pub id: String,
    pub defender: T,
    pub challenger: T,
    pub created: i64,
    pub defender_rating: f32,
    pub challenger_rating: f32,
    pub result: Option<GameResult>,
    pub rated: bool,
}

#[derive(Serialize, Deserialize, Debug, Queryable, Selectable, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
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
}

#[derive(Serialize, Deserialize, Debug, Queryable, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
pub struct BotWithTeam<T> {
    pub id: i32,
    pub team: T,
    pub name: String,
    pub description: Option<String>,
    pub created: i64,
    pub uploaded_by: User,
    pub build_status: BuildStatus,
}

impl<T> BotWithTeam<T> {
    pub fn from_bot_team_user(bot: Bot, team: T, user: User) -> Self {
        BotWithTeam {
            id: bot.id,
            team,
            name: bot.name,
            description: bot.description,
            created: bot.created,
            uploaded_by: user,
            build_status: bot.build_status,
        }
    }
}

#[derive(Serialize, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
pub struct GameWithBots<T> {
    pub id: String,
    pub defender: T,
    pub challenger: T,
    pub created: i64,
    pub rated: bool,
}

#[derive(Debug, diesel::Insertable)]
#[diesel(table_name = bots)]
pub struct NewBot {
    pub team: i32,
    pub name: String,
    pub description: Option<String>,
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
    pub id: Uuid,
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Queryable,
    Selectable,
    Insertable,
    Associations,
    Identifiable,
    TS,
    AsChangeset,
)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
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
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Queryable, Selectable, Insertable, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
#[diesel(table_name = game_states)]
pub struct GameStateSQL {
    pub game_id: String,
    pub step: i32,
    pub challenger_stack: i32,
    pub defender_stack: i32,
    pub challenger_pushed: i32,
    pub defender_pushed: i32,
    pub challenger_hand: HoleCards,
    pub defender_hand: HoleCards,
    pub community_cards: CommunityCards,
    pub sb: WhichBot,
    pub action_time: i32,
    pub whose_turn: Option<PlayerPosition>,
    pub action_val: Action,
    pub end_reason: Option<EndReason>,
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

impl ToSql<Text, pg::Pg> for GameError {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        out.write_all(serde_json::to_vec(self)?.as_slice())?;
        Ok(diesel::serialize::IsNull::No)
    }
}

impl FromSql<Text, pg::Pg> for GameError {
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        let s = String::from_sql(bytes)?;
        serde_json::from_str(&s)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}

impl ToSql<VarChar, pg::Pg> for HoleCards {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        out.write_all(serde_json::to_vec(self)?.as_slice())?;
        Ok(diesel::serialize::IsNull::No)
    }
}

impl FromSql<VarChar, pg::Pg> for HoleCards {
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        let s = String::from_sql(bytes)?;
        serde_json::from_str(&s)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}

impl ToSql<VarChar, pg::Pg> for CommunityCards {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        out.write_all(serde_json::to_vec(self)?.as_slice())?;
        Ok(diesel::serialize::IsNull::No)
    }
}

impl FromSql<VarChar, pg::Pg> for CommunityCards {
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        let s = String::from_sql(bytes)?;
        serde_json::from_str(&s)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}

impl ToSql<VarChar, pg::Pg> for Action {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        out.write_all(serde_json::to_vec(self)?.as_slice())?;
        Ok(diesel::serialize::IsNull::No)
    }
}

impl FromSql<VarChar, pg::Pg> for Action {
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        let s = String::from_sql(bytes)?;
        serde_json::from_str(&s)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}

impl ToSql<Integer, pg::Pg> for PlayerPosition {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, pg::Pg>,
    ) -> diesel::serialize::Result {
        let val = *self as i32;
        ToSql::<Integer, pg::Pg>::to_sql(&val, &mut out.reborrow())
    }
}

impl FromSql<Integer, pg::Pg> for PlayerPosition {
    fn from_sql(bytes: PgValue) -> diesel::deserialize::Result<Self> {
        if let Some(result) = num::FromPrimitive::from_i32(i32::from_sql(bytes)?) {
            Ok(result)
        } else {
            Err("Invalid build status".into())
        }
    }
}

impl ToSql<VarChar, pg::Pg> for EndReason {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, pg::Pg>,
    ) -> diesel::serialize::Result {
        out.write_all(serde_json::to_vec(self)?.as_slice())?;
        Ok(diesel::serialize::IsNull::No)
    }
}

impl FromSql<VarChar, pg::Pg> for EndReason {
    fn from_sql(bytes: PgValue) -> diesel::deserialize::Result<Self> {
        let s = String::from_sql(bytes)?;
        serde_json::from_str(&s)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}