use diesel::alias;
use itertools::Itertools;
use shared::{
    db::{
        dao::bots::BotsDao,
        models::{BotWithTeam, GameWithBots, Team},
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
    diesel::insert_into(schema::games::dsl::games)
        .values(NewGame {
            defender,
            challenger,
            id: id.clone(),
        })
        .get_result::<Game>(conn)?;
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
            defender: defender.to_string(),
            challenger: challenger.to_string(),
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

#[derive(Deserialize)]
pub struct GameQuery {
    pub id: Option<String>,
    pub team: Option<i32>,
    pub active: Option<bool>,
    pub page_size: Option<i32>,
    pub page: Option<i32>,
    pub count: Option<bool>,
    pub join_bots: Option<bool>,
}

#[derive(Serialize, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
pub enum GamesResponse {
    Count(i64),
    Games(Vec<Game>),
    GamesWithBots(Vec<GameWithBots<BotWithTeam<Team>>>),
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
        join_bots,
    }): web::Query<GameQuery>,
) -> ApiResult<GamesResponse> {
    use schema::*;
    let conn = &mut (*DB_CONNECTION).get()?;
    // Is joining unconditionally bad?
    let mut base = games::table
        .inner_join(defender_bots.on(games::dsl::defender.eq(defender_bots.field(bots::dsl::id))))
        .inner_join(
            challenger_bots.on(games::dsl::challenger.eq(challenger_bots.field(bots::dsl::id))),
        )
        .inner_join(
            defender_teams.on(defender_bots
                .field(bots::dsl::team)
                .eq(defender_teams.field(teams::dsl::id))),
        )
        .inner_join(
            challenger_teams.on(challenger_bots
                .field(bots::dsl::team)
                .eq(challenger_teams.field(teams::dsl::id))),
        )
        .into_boxed();
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
            schema::games::dsl::defender
                .eq_any(bots.clone())
                .or(schema::games::dsl::challenger.eq_any(bots.clone())),
        );
    }
    let count = count.unwrap_or(false);
    let page_size = page_size.unwrap_or(10).min(100);
    let page = page.unwrap_or(0);
    if count {
        let count = base.count().get_result::<i64>(conn)?;
        return Ok(web::Json(GamesResponse::Count(count)));
    }
    base = base
        .order_by(schema::games::dsl::created.desc())
        .limit((page_size).into())
        .offset((page * page_size).into());

    let result: Vec<(Game, Bot, Bot, Team, Team)> =
        base.load::<(Game, Bot, Bot, Team, Team)>(conn)?;
    if join_bots.unwrap_or(false) {
        let result: Vec<GameWithBots<BotWithTeam<Team>>> = result
            .into_iter()
            .map(
                |(game, defender, challenger, defender_team, challenger_team)| GameWithBots {
                    id: game.id,
                    defender: BotWithTeam::from_bot_and_team(defender, defender_team),
                    challenger: BotWithTeam::from_bot_and_team(challenger, challenger_team),
                    score_change: game.score_change,
                    created: game.created,
                    error_type: game.error_type,
                    error_message: game.error_message,
                },
            )
            .collect();
        return Ok(web::Json(GamesResponse::GamesWithBots(result)));
    }
    Ok(web::Json(GamesResponse::Games(
        result.into_iter().map(|(g, _, _, _, _)| g).collect(),
    )))
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
    let team = login::get_team_data(&session)
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
