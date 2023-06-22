use crate::app::api::ApiResult;
use actix_session::Session;
use actix_web::{
    get,
    web::{self},
    HttpResponse,
};
use diesel::prelude::*;
use rand::{self, Rng};
use serde::Deserialize;
use serde_json::json;
use shared::db::conn::DB_CONNECTION;
use shared::db::{
    models::{Game, NewGame},
    schema,
};
use shared::GameTask;
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
        .values(NewGame {
            bot_a,
            bot_b,
            id: id.clone(),
        })
        .get_result::<Game>(conn)?;
    // push a batch job to the queue
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
    log::info!("Game created {:?}", job);
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
