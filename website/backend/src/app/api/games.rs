use crate::{app::api::ApiResult, config::GAME_LOGS_S3_BUCKET};
use actix_session::Session;
use actix_web::{
    get,
    web::{self},
    HttpResponse,
};
use aws_sdk_s3::presigning::{self, PresigningConfig};
use diesel::prelude::*;
use futures_util::future::{join, join3, try_join3};
use rand::{self, Rng};
use reqwest::header::ToStrError;
use serde::Deserialize;
use serde_json::json;
use shared::db::{
    models::{Game, NewGame},
    schema,
};
use shared::GameTask;
use shared::{db::conn::DB_CONNECTION, PresignedRequest};
#[derive(Deserialize)]
pub struct MakeGameQuery {
    pub bot_a: i32,
    pub bot_b: i32,
}

//TODO: restrict who can make games
#[get("/create-game")]
pub async fn create_game(
    session: Session,
    web::Query::<MakeGameQuery>(MakeGameQuery { bot_a, bot_b }): web::Query<MakeGameQuery>,
    sqs_client: web::Data<aws_sdk_sqs::Client>,
    s3_client: web::Data<aws_sdk_s3::Client>,
) -> ApiResult {
    // generate a random code and insert it into the database
    // also push a batch job to the queue
    let id = format!("{:02x}", rand::thread_rng().gen::<u128>());
    let conn = &mut (*DB_CONNECTION).get()?;
    let game = diesel::insert_into(schema::games::dsl::games)
        .values(NewGame {
            bot_a,
            bot_b,
            id: id.clone(),
        })
        .get_result::<Game>(conn)?;
    // push a batch job to the queue
    let presign_config =
        PresigningConfig::expires_in(std::time::Duration::from_secs(60 * 60 * 24 * 7))?;
    let (public_logs, bot_a_logs, bot_b_logs) = try_join3(
        s3_client
            .put_object()
            .bucket(&*GAME_LOGS_S3_BUCKET)
            .key(format!("public/{}", game.id))
            .presigned(presign_config.clone()),
        s3_client
            .put_object()
            .bucket(&*GAME_LOGS_S3_BUCKET)
            .key(format!("{}/{}", bot_a, game.id))
            .presigned(presign_config.clone()),
        s3_client
            .put_object()
            .bucket(&*GAME_LOGS_S3_BUCKET)
            .key(format!("{}/{}", bot_b, game.id))
            .presigned(presign_config.clone()),
    )
    .await?;
    let (public_logs_presigned, bot_a_logs_presigned, bot_b_logs_presigned) = (
        PresignedRequest {
            url: public_logs.uri().to_string(),
            headers: public_logs.headers().into(),
        },
        PresignedRequest {
            url: bot_a_logs.uri().to_string(),
            headers: bot_a_logs.headers().into(),
        },
        PresignedRequest {
            url: bot_b_logs.uri().to_string(),
            headers: bot_b_logs.headers().into(),
        },
    );
    let job = sqs_client
        .send_message()
        .queue_url(std::env::var("NEW_GAMES_QUEUE_URL")?)
        .message_body(&serde_json::to_string(&GameTask::Game {
            bot_a: game.bot_a.to_string(),
            bot_b: game.bot_b.to_string(),
            id: game.id.clone(),
            date: game.created,
            // TODO: Choose a number of rounds
            rounds: 100,
            public_logs_presigned,
            bot_a_logs_presigned,
            bot_b_logs_presigned,
        })?)
        .send();
    if let Err(e) = job.await {
        // Remove the game from the database
        diesel::delete(schema::games::dsl::games)
            .filter(schema::games::dsl::id.eq(id))
            .execute(conn)?;
        return Err(e.into());
    }
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
