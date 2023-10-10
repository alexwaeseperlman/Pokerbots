use diesel::alias;
use itertools::Itertools;
use shared::{
    db::{
        dao::{
            bots::BotsDao,
            games::{GameQueryOptions, GamesDao, PageOptions},
        },
        models::{BotWithTeam, GameWithBots, GameWithBotsWithResult, Team},
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
