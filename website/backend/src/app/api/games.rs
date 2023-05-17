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
use rand::{self, Rng};
use serde::Deserialize;
use serde_json::json;

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
    batch_client: web::Data<aws_sdk_batch::Client>,
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
    // push a batch job to the queue
    batch_client
        .submit_job()
        .set_job_name(Some(id.clone()))
        .set_job_queue(Some((*crate::config::JOB_QUEUE).clone()))
        .set_job_definition(Some((*crate::config::PLAY_JOB_DEFINITION).clone()))
        .parameters("botA", teamA.to_string())
        .parameters("botB", teamB.to_string())
        .parameters("id", id)
        .send()
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Unable to submit job: {}", e))
        })?;
    Ok(HttpResponse::Ok().json(game))
}
