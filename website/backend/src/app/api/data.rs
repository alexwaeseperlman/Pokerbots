use actix_session::Session;
use actix_web::{get, web, HttpResponse};
use diesel::*;
use s3::presigning::PresigningConfig;
use serde::Deserialize;
use serde_json::json;

use crate::{
    app::login,
    config::{DB_CONNECTION, PFP_S3_BUCKET},
    models::{Bot, Team},
    schema,
};

use super::ServerMessage;
use aws_sdk_s3 as s3;

#[get("/api/my-account")]
pub async fn my_account(session: Session) -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(login::get_user_data(&session)))
}

#[get("/api/my-team")]
pub async fn my_team(session: Session) -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(login::get_team_data(&session)))
}

#[derive(Deserialize)]
pub struct ServerMessageQuery {
    pub clear: Option<bool>,
}

#[get("/api/server-message")]
pub async fn server_message(
    session: Session,
    web::Query::<ServerMessageQuery>(ServerMessageQuery { clear }): web::Query<ServerMessageQuery>,
) -> actix_web::Result<HttpResponse> {
    let msg = HttpResponse::Ok().json(session.get::<ServerMessage>("message")?);
    if clear.unwrap_or(false) {
        session.remove("message");
    }
    Ok(msg)
}
#[derive(Deserialize)]
pub struct TeamQuery {
    pub ids: Option<Vec<i32>>,
    pub page_size: Option<i32>,
    pub page: Option<i32>,
}

#[get("/api/teams")]
pub async fn teams(
    session: Session,
    web::Query::<TeamQuery>(TeamQuery {
        ids,
        page_size,
        page,
    }): web::Query<TeamQuery>,
) -> actix_web::Result<HttpResponse> {
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    let mut base = schema::teams::dsl::teams
        .order_by(schema::teams::dsl::score.desc())
        .into_boxed();
    if let Some(ids) = ids {
        base = base.filter(schema::teams::dsl::id.eq_any(ids));
    }
    let page_size = page_size.unwrap_or(10).min(100);
    let page = page.unwrap_or(0);
    base = base
        .limit((page_size).into())
        .offset((page * page_size).into());
    let result: Vec<Team> = base
        .load::<Team>(conn)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Unable to read teams: {}", e))
        })?
        .into_iter()
        .collect();
    Ok(HttpResponse::Ok().json(result))
}

#[derive(Deserialize)]
pub struct BotQuery {
    pub ids: Option<Vec<i32>>,
    pub team: Option<i32>,
    pub page_size: Option<i32>,
    pub page: Option<i32>,
    pub count: Option<bool>,
}

#[get("/api/bots")]
pub async fn bots(
    session: Session,
    web::Query::<BotQuery>(BotQuery {
        ids,
        team,
        page_size,
        page,
        count,
    }): web::Query<BotQuery>,
) -> actix_web::Result<HttpResponse> {
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    let mut base = schema::bots::dsl::bots.into_boxed();
    if let Some(ids) = ids {
        base = base.filter(schema::bots::dsl::id.eq_any(ids));
    }
    if let Some(team) = team {
        // get bots belonging to the team
        base = base.filter(schema::bots::dsl::team.eq(team))
    }
    let count = count.unwrap_or(false);
    let page_size = page_size.unwrap_or(10).min(100);
    let page = page.unwrap_or(0);
    if count {
        let count = base.count().get_result::<i64>(conn).map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Unable to count bots: {}", e))
        })?;
        return Ok(HttpResponse::Ok().json(json!({ "count": count })));
    }
    base = base
        .order_by(schema::bots::dsl::created.desc())
        .limit((page_size).into())
        .offset((page * page_size).into());
    let result: Vec<Bot> = base
        .load::<Bot>(conn)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Unable to read bots: {}", e))
        })?
        .into_iter()
        .collect();
    Ok(HttpResponse::Ok().json(result))
}
