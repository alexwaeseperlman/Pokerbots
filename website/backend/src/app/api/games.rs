use std::sync::Arc;

use crate::{
    app::login,
    app::{
        api::{ApiError, ApiResult},
        login::microsoft_login_url,
    },
    config::{BOT_S3_BUCKET, DB_CONNECTION, PFP_S3_BUCKET},
    models::{Bot, Game, TeamInvite, User},
    schema,
};
use actix_session::Session;
use actix_web::{
    error::InternalError,
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
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;
use shared::PlayTask;
#[derive(Deserialize)]
pub struct MakeGameQuery {
    pub bot_a: i32,
    pub bot_b: i32,
}

//TODO: restrict who can make games
#[get("/make-game")]
pub async fn make_game(
    session: Session,
    web::Query::<MakeGameQuery>(MakeGameQuery { bot_a, bot_b }): web::Query<MakeGameQuery>,
    sqs_client: web::Data<aws_sdk_sqs::Client>,
) -> ApiResult {
    // generate a random code and insert it into the database
    // also push a batch job to the queue
    let id = format!("{:02x}", rand::thread_rng().gen::<u128>());
    let conn = &mut (*DB_CONNECTION).get()?;
    let game = diesel::insert_into(schema::games::dsl::games)
        .values(crate::models::NewGame {
            bot_a,
            bot_b,
            id: id.clone(),
        })
        .get_result::<Game>(conn)?;
    // push a batch job to the queue
    let job = sqs_client
        .send_message()
        .queue_url(
            sqs_client
                .get_queue_url()
                .queue_name("new_games")
                .send()
                .await?
                .queue_url()
                .ok_or(ApiError {
                    message: "no queue url".into(),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                })?,
        )
        .message_body(&serde_json::to_string(&PlayTask {
            bot_a: game.bot_a.to_string(),
            bot_b: game.bot_b.to_string(),
            id: game.id.clone(),
            date: game.created,
        })?)
        .send()
        .await;
    if let Err(e) = job {
        // Remove the game from the database
        diesel::delete(schema::games::dsl::games)
            .filter(schema::games::dsl::id.eq(id))
            .execute(conn)?;
        return Err(e.into());
    }
    log::debug!("message sent {:?}", job);
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

#[get("/games")]
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
) -> ApiResult {
    let conn = &mut (*DB_CONNECTION).get()?;
    let mut base = schema::games::dsl::games.into_boxed();
    if let Some(active) = active {
        base = base.filter(schema::games::dsl::score_change.is_null().eq(active))
    }
    if let Some(id) = id {
        base = base.filter(schema::games::dsl::id.eq(id));
    }
    if let Some(team) = team {
        // get bots belonging to the team
        let bots: Vec<i32> = schema::bots::dsl::bots
            .filter(schema::bots::dsl::team.eq(team))
            .select(schema::bots::dsl::id)
            .load::<i32>(conn)?
            .into_iter()
            .collect();
        base = base.filter(
            schema::games::dsl::bot_a
                .eq_any(bots.clone())
                .or(schema::games::dsl::bot_b.eq_any(bots.clone())),
        );
    }
    let count = count.unwrap_or(false);
    let page_size = page_size.unwrap_or(10).min(100);
    let page = page.unwrap_or(0);
    if count {
        let count = base.count().get_result::<i64>(conn)?;
        return Ok(HttpResponse::Ok().json(json!({ "count": count })));
    }
    base = base
        .order_by(schema::games::dsl::created.desc())
        .limit((page_size).into())
        .offset((page * page_size).into());
    let result: Vec<Game> = base.load::<Game>(conn)?.into_iter().collect();
    Ok(HttpResponse::Ok().json(result))
}
