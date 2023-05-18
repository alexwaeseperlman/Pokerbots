use std::sync::Arc;

use crate::{
    app::login,
    app::login::microsoft_login_url,
    config::{BOT_S3_BUCKET, DB_CONNECTION, PFP_S3_BUCKET},
    models::{Game, TeamInvite, User},
    schema::{self, team_invites, teams, users},
};
use actix_session::Session;
use actix_web::{
    get, put,
    web::{self, Bytes},
    HttpResponse,
};
use aws_sdk_s3 as s3;
use aws_sdk_s3::presigning::PresigningConfig;
use chrono;
use diesel::{prelude::*, query_builder::SelectQuery};
use futures_util::FutureExt;
use rand::{self, Rng};
use serde::Deserialize;
use serde_json::json;
use shared::PlayTask;

#[derive(Deserialize)]
pub struct GameResultQuery {
    pub id: String,
    pub change: i32,
}

#[derive(Deserialize)]
pub struct MakeGameQuery {
    pub teamA: i32,
    pub teamB: i32,
}

#[get("/api/game-result")]
pub async fn game_result(
    session: Session,
    web::Query::<GameResultQuery>(GameResultQuery { id, change }): web::Query<GameResultQuery>,
) -> actix_web::Result<HttpResponse> {
    // check for the code in the database and update the game
    // with this value
    // Insert an invite with expiry date 24 hours from now
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    diesel::update(schema::games::dsl::games)
        .filter(schema::games::dsl::id.eq(id))
        .set(schema::games::dsl::score_change.eq(change))
        .execute(conn)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Unable to update game: {}", e))
        })?;
    //TODO: calculate elo
    Ok(HttpResponse::Ok().body(""))
}

//TODO: restrict who can make games
#[get("/api/make-game")]
pub async fn make_game(
    session: Session,
    web::Query::<MakeGameQuery>(MakeGameQuery { teamA, teamB }): web::Query<MakeGameQuery>,
    amqp_channel: web::Data<Arc<lapin::Channel>>,
) -> actix_web::Result<HttpResponse> {
    // generate a random code and insert it into the database
    // also push a batch job to the queue
    let id = format!("{:02x}", rand::thread_rng().gen::<u128>());
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    let game = diesel::insert_into(schema::games::dsl::games)
        .values(crate::models::NewGame {
            teama: teamA,
            teamb: teamB,
            id: id.clone(),
        })
        .get_result::<Game>(conn)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Unable to create game: {}", e))
        })?;
    // push a batch job to the queue
    log::debug!(
        "message sent {:?}",
        amqp_channel
            .basic_publish(
                "",
                "poker",
                lapin::options::BasicPublishOptions::default(),
                &serde_json::to_vec(&PlayTask {
                    bota: game.teama.to_string(),
                    botb: game.teamb.to_string(),
                    id: game.id.clone(),
                    date: game.created
                })
                .map_err(|e| {
                    actix_web::error::ErrorInternalServerError(format!(
                        "Unable to serialize game: {}",
                        e
                    ))
                })?,
                lapin::BasicProperties::default(),
            )
            .await
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Unable to send game: {}", e))
            })?
            .await
            .unwrap()
    );
    Ok(HttpResponse::Ok().json(game))
}

#[derive(Deserialize)]
pub struct GameQuery {
    pub id: Option<String>,
    pub team: Option<i32>,
    pub active: Option<bool>,
    pub page_size: Option<i32>,
    pub page: Option<i32>,
    pub count: Option<bool>,
}

#[get("/api/games")]
pub async fn games(
    session: Session,
    web::Query::<GameQuery>(GameQuery {
        id,
        team,
        active,
        page_size,
        page,
        count,
    }): web::Query<GameQuery>,
) -> actix_web::Result<HttpResponse> {
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    let mut base = schema::games::dsl::games.into_boxed();
    if let Some(active) = active {
        base = base.filter(schema::games::dsl::score_change.is_null().eq(active))
    }
    if let Some(id) = id {
        base = base.filter(schema::games::dsl::id.eq(id));
    }
    if let Some(team) = team {
        base = base.filter(
            schema::games::dsl::teama
                .eq(team)
                .or(schema::games::dsl::teamb.eq(team)),
        );
    }
    let count = count.unwrap_or(false);
    let page_size = page_size.unwrap_or(10).min(100);
    let page = page.unwrap_or(0);
    if count {
        let count = base.count().get_result::<i64>(conn).map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Unable to count games: {}", e))
        })?;
        return Ok(HttpResponse::Ok().json(json!({ "count": count })));
    }
    base = base
        .order_by(schema::games::dsl::created.desc())
        .limit((page_size).into())
        .offset((page * page_size).into());
    let result: Vec<Game> = base
        .load::<Game>(conn)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Unable to update game: {}", e))
        })?
        .into_iter()
        .collect();
    Ok(HttpResponse::Ok().json(result))
}
