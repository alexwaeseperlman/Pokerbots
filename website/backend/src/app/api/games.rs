use std::sync::Arc;

use crate::{
    app::login,
    app::login::microsoft_login_url,
    config::{BOT_S3_BUCKET, DB_CONNECTION, PFP_S3_BUCKET},
    models::{Game, TeamInvite, User},
    schema::{games, team_invites, teams, users},
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
use diesel::prelude::*;
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
    diesel::update(games::dsl::games)
        .filter(games::dsl::id.eq(id))
        .set(games::dsl::score_change.eq(change))
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
    let game = diesel::insert_into(games::dsl::games)
        .values(crate::models::NewGame {
            teama: teamA,
            teamb: teamB,
            id: id.clone(),
        })
        .get_result::<Game>(conn)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Unable to create game: {}", e))
        })?;
    log::debug!("Channel state {:?}", amqp_channel.status());
    // push a batch job to the queue
    log::debug!(
        "message sent {:?}",
        amqp_channel
            .basic_publish(
                "",
                "poker",
                lapin::options::BasicPublishOptions {
                    mandatory: true,
                    ..Default::default()
                },
                &serde_json::to_vec(&PlayTask {
                    bota: game.teama.to_string(),
                    botb: game.teamb.to_string(),
                    id: game.id.clone(),
                    date: chrono::Utc::now().naive_utc().timestamp_millis(),
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
            .take_message()
    );
    Ok(HttpResponse::Ok().json(game))
}
