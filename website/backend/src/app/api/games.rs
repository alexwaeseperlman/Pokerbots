use std::usize;

use diesel::alias;
use futures_util::TryStreamExt;
use itertools::Itertools;
use shared::{
    db::{
        dao::{
            bots::BotsDao,
            games::{GameQueryOptions, GamesDao, PageOptions},
        },
        models::{BotWithTeam, GameStateSQL, GameWithBots, GameWithBotsWithResult, Team},
        schema_aliases::*,
    },
    WhichBot,
};

use super::*;

#[derive(Deserialize)]
pub struct MakeGameQuery {
    pub defender: i32,
    pub challenger: i32,
}

#[derive(Serialize, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
pub struct CreateGameResponse {
    pub id: String,
}

#[get("/games")]
pub async fn games(
    session: Session,
    web::Query::<GameQueryOptions>(game_query_options): web::Query<GameQueryOptions>,
    web::Query::<PageOptions>(page_options): web::Query<PageOptions>,
) -> ApiResult<Vec<GameWithBotsWithResult<BotWithTeam<Team>>>> {
    let conn = &mut (*DB_CONNECTION).get()?;

    Ok(web::Json(
        conn.get_games(game_query_options, page_options).await?,
    ))
}

#[get("/count-games")]
pub async fn count_games(
    session: Session,
    web::Query::<GameQueryOptions>(game_query_options): web::Query<GameQueryOptions>,
) -> ApiResult<i64> {
    let conn = &mut (*DB_CONNECTION).get()?;

    Ok(web::Json(conn.count_games(game_query_options).await?))
}

#[derive(Deserialize)]
pub struct GameLogQuery {
    id: String,
    which_bot: Option<WhichBot>,
}

#[derive(Deserialize)]
pub struct GameRecordQuery {
    id: String,
    round: usize,
}

#[get("/game-log")]
pub async fn game_log(
    session: Session,
    web::Query::<GameLogQuery>(GameLogQuery { id, which_bot }): web::Query<GameLogQuery>,
    s3_client: web::Data<aws_sdk_s3::Client>,
) -> Result<HttpResponse, ApiError> {
    let team =
        auth::get_team(&session).ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;
    let conn = &mut (*DB_CONNECTION).get()?;
    // If the bot is specified, make sure it belongs to the team
    if let Some(which_bot) = which_bot {
        let game: Game = schema::games::dsl::games
            .filter(schema::games::dsl::id.eq(&id))
            .first::<Game>(conn)?;
        let bot = match which_bot {
            WhichBot::Defender => game.defender,
            WhichBot::Challenger => game.challenger,
        };
        let bot: Vec<Bot> = schema::bots::dsl::bots
            .filter(schema::bots::dsl::id.eq(bot))
            .filter(schema::bots::dsl::team.eq(team.id))
            .load::<Bot>(conn)?;
        if bot.len() == 0 {
            return Err(actix_web::error::ErrorUnauthorized(
                "Only the owner can view a bot's logs.",
            )
            .into());
        }
    }
    let key = format!(
        "{}/{}",
        which_bot.map(|b| b.to_string()).unwrap_or("public".into()),
        id
    );
    let response = s3_client
        .get_object()
        .bucket(game_logs_s3_bucket())
        .key(key)
        .send()
        .await?;

    Ok(HttpResponse::Ok().streaming(response.body))
}

#[get("/game-record")]
pub async fn game_record(
    session: Session,
    web::Query::<GameRecordQuery>(GameRecordQuery { id, round }): web::Query<GameRecordQuery>,
    s3_client: web::Data<aws_sdk_s3::Client>,
) -> Result<HttpResponse, ApiError> {
    let key = format!("game_record/{}", id);
    let mut response = s3_client
        .get_object()
        .bucket(game_logs_s3_bucket())
        .key(key)
        .send()
        .await?;
    let mut out = Vec::new();
    while let Some(bytes) = response
        .body
        .try_next()
        .await
        .map_err(|_| actix_web::error::ErrorBadRequest("Not working"))?
    {
        out.extend_from_slice(&bytes);
    }
    let line = out
        .split(|b| *b == 0xA)
        .nth(round)
        .unwrap_or_default()
        .to_vec();
    Ok(HttpResponse::Ok().body(line))
}

#[derive(Deserialize)]
pub struct GameLengthQuery {
    game_id: String,
}

use diesel::prelude::*;
#[get("/game-length")]
pub async fn game_length(
    session: Session,
    web::Query::<GameLengthQuery>(GameLengthQuery { game_id }): web::Query<GameLengthQuery>,
) -> Result<HttpResponse, ApiError> {
    let conn = &mut (*DB_CONNECTION).get()?;
    let max = schema::game_states::dsl::game_states
        .filter(schema::game_states::dsl::game_id.eq(game_id.clone()))
        .order(schema::game_states::dsl::step.desc())
        .select(schema::game_states::dsl::step)
        .first::<i32>(conn)
        .map(|obj| obj.to_string());
    match max {
        Ok(max) => return Ok(HttpResponse::Ok().body(max)),
        Err(err) => return Err(actix_web::error::ErrorNotFound(err).into()),
    }
}
