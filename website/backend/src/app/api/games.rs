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

//TODO: restrict who can make games
#[get("/create-game")]
pub async fn create_game(
    session: Session,
    web::Query::<MakeGameQuery>(MakeGameQuery {
        defender,
        challenger,
    }): web::Query<MakeGameQuery>,
    sqs_client: web::Data<aws_sdk_sqs::Client>,
    s3_client: web::Data<aws_sdk_s3::Client>,
) -> ApiResult<CreateGameResponse> {
    // generate a random code and insert it into the database
    // also push a batch job to the queue
    let id = format!("{:02x}", rand::thread_rng().gen::<u128>());
    let conn = &mut (*DB_CONNECTION).get()?;
    let defender_bot: Bot = schema::bots::dsl::bots
        .filter(schema::bots::dsl::id.eq(defender))
        .first::<Bot>(conn)?;
    let challenger_bot: Bot = schema::bots::dsl::bots
        .filter(schema::bots::dsl::id.eq(challenger))
        .first::<Bot>(conn)?;
    diesel::insert_into(schema::games::dsl::games)
        .values(NewGame {
            defender,
            challenger,
            id: id.clone(),
            challenger_rating: challenger_bot.rating,
            defender_rating: defender_bot.rating,
        })
        .execute(conn)?;
    // push a batch job to the queue
    let presign_config =
        PresigningConfig::expires_in(std::time::Duration::from_secs(60 * 60 * 24 * 7))?;
    let (public_logs, defender_logs, challenger_logs) = try_join3(
        s3_client
            .put_object()
            .bucket(&*GAME_LOGS_S3_BUCKET)
            .key(format!("public/{}", id))
            .presigned(presign_config.clone()),
        s3_client
            .put_object()
            .bucket(&*GAME_LOGS_S3_BUCKET)
            .key(format!("{}/{}", WhichBot::Defender.to_string(), id))
            .presigned(presign_config.clone()),
        s3_client
            .put_object()
            .bucket(&*GAME_LOGS_S3_BUCKET)
            .key(format!("{}/{}", WhichBot::Challenger.to_string(), id))
            .presigned(presign_config.clone()),
    )
    .await?;
    let (public_logs_presigned, defender_logs_presigned, challenger_logs_presigned) = (
        PresignedRequest {
            url: public_logs.uri().to_string(),
            headers: public_logs.headers().into(),
        },
        PresignedRequest {
            url: defender_logs.uri().to_string(),
            headers: defender_logs.headers().into(),
        },
        PresignedRequest {
            url: challenger_logs.uri().to_string(),
            headers: challenger_logs.headers().into(),
        },
    );
    let job = sqs_client
        .send_message()
        .queue_url(std::env::var("NEW_GAMES_QUEUE_URL")?)
        .message_body(&serde_json::to_string(&GameTask::Game {
            defender: defender,
            challenger: challenger,
            id: id.clone(),
            // TODO: Choose a number of rounds
            rounds: 100,
            public_logs_presigned,
            defender_logs_presigned,
            challenger_logs_presigned,
        })?)
        .send();
    if let Err(e) = job.await {
        // Remove the game from the database
        diesel::delete(schema::games::dsl::games)
            .filter(schema::games::dsl::id.eq(id))
            .execute(conn)?;
        return Err(e.into());
    }
    Ok(web::Json(CreateGameResponse { id }))
}

#[get("/games")]
pub async fn games(
    session: Session,
    web::Query::<GameQueryOptions>(game_query_options): web::Query<GameQueryOptions>,
    web::Query::<PageOptions>(page_options): web::Query<PageOptions>,
) -> ApiResult<Vec<GameWithBotsWithResult<BotWithTeam<Team>>>> {
    let conn = &mut (*DB_CONNECTION).get()?;

    Ok(web::Json(conn.get_games(game_query_options, page_options)?))
}

#[get("/count-games")]
pub async fn count_games(
    session: Session,
    web::Query::<GameQueryOptions>(game_query_options): web::Query<GameQueryOptions>,
) -> ApiResult<i64> {
    let conn = &mut (*DB_CONNECTION).get()?;

    Ok(web::Json(conn.count_games(game_query_options)?))
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
    let team = auth::get_team(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;
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
        .bucket(&*GAME_LOGS_S3_BUCKET)
        .key(key)
        .send()
        .await?;

    Ok(HttpResponse::Ok().streaming(response.body))
}
