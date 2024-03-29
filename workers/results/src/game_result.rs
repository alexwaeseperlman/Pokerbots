use aws_sdk_s3::{config, primitives::ByteStreamError};
use diesel::prelude::*;
use futures_lite::StreamExt;
use log::error;
use shared::{
    db::{
        self,
        models::{self, Bot, Game, NewBot, Team},
        schema::{
            game_results,
            game_states::{self, defender_hand},
            teams,
        },
    },
    poker::game::GameStateSQL,
    GameError, GameStatus, GameStatusMessage, WhichBot,
};

use crate::rating::get_rating_change;

pub fn sb_to_team(sb: WhichBot) -> [usize; 2] {
    match sb {
        WhichBot::Defender => [1, 0],
        WhichBot::Challenger => [0, 1],
    }
}

pub async fn save_game_details<T: AsRef<str>>(id: T) -> Result<(), ()> {
    log::info!("Saving game details for {}", id.as_ref());
    let id_str = id.as_ref();
    let config = shared::aws_config().await;
    let s3 = shared::s3_client(&config).await;
    let key = format!("game_record/{}", id_str);
    let response = s3
        .get_object()
        .bucket(std::env::var("GAME_LOGS_S3_BUCKET").unwrap())
        .key(key)
        .send()
        .await
        .map_err(|e| log::error!("Could not access S3 for getting game records: {}", e))?;
    let body = response
        .body
        .collect()
        .await
        .map_err(|e| log::error!("Failed to collect game states: {}", e))?;
    let vec = body.to_vec();
    let lines = vec.split(|b| *b == 0xA);
    let conn = &mut (*shared::db::conn::DB_CONNECTION)
        .get()
        .map_err(|e| log::error!("Failled to open connection to SLQ: {}", e))?;
    for line in lines {
        let mut game_state: GameStateSQL = serde_json::from_slice(line)
            .map_err(|e| log::error!("Failed to convert json to GameStateSQL: {}", e))?;
        game_state.game_id = id_str.into();
        diesel::insert_into(db::schema::game_states::dsl::game_states)
            .values(game_state)
            .execute(conn)
            .map_err(|err| log::error!("Failed to save GameStateSQL to SQL: {}", err))?;
    }
    Ok(())
}

pub async fn handle_game_result(status: GameStatusMessage) -> Result<(), ()> {
    let starting_stack_size = std::env::var("STARTING_STACK_SIZE")
        .unwrap_or("500".to_string())
        .parse::<i32>()
        .unwrap_or(500);

    use shared::db::schema::{bots, games};
    let db_conn = &mut (*shared::db::conn::DB_CONNECTION.get().map_err(|_| ())?);
    let GameStatusMessage { id, result } = status;
    let error_type = result.clone().err();
    let (defender_score, challenger_score) = match result.clone() {
        Ok(GameStatus::ScoreChanged(defender_score, challenger_score)) => {
            (defender_score, challenger_score)
        }
        Ok(GameStatus::TestGameSucceeded) => (0, 0),
        Ok(GameStatus::TestGameFailed) => (0, 0),
        Err(e) => match e {
            GameError::InternalError => (starting_stack_size, starting_stack_size),
            GameError::InvalidActionError(which_bot) => match which_bot {
                shared::WhichBot::Defender => (-starting_stack_size, starting_stack_size),
                shared::WhichBot::Challenger => (starting_stack_size, -starting_stack_size),
            },
            GameError::MemoryError(which_bot) => match which_bot {
                shared::WhichBot::Defender => (-starting_stack_size, starting_stack_size),
                shared::WhichBot::Challenger => (starting_stack_size, -starting_stack_size),
            },
            GameError::RunTimeError(which_bot) => match which_bot {
                shared::WhichBot::Defender => (-starting_stack_size, starting_stack_size),
                shared::WhichBot::Challenger => (starting_stack_size, -starting_stack_size),
            },
            GameError::TimeoutError(which_bot) => match which_bot {
                shared::WhichBot::Defender => (-starting_stack_size, starting_stack_size),
                shared::WhichBot::Challenger => (starting_stack_size, -starting_stack_size),
            },
        },
    };
    let (mut defender_rating_change, mut challenger_rating_change) = (0f32, 0f32);

    // transaction
    db_conn
        .transaction(|db_conn| {
            match result {
                Ok(GameStatus::ScoreChanged(_, _)) | Err(_) => {
                    let game: Game =
                        games::table.find(&id).first::<Game>(db_conn).map_err(|e| {
                            log::error!("Failed to find game with id {}", id);
                            e
                        })?;
                    // calculate the bots ratings
                    let score = (starting_stack_size as f32 + defender_score as f32)
                        / (2.0f32 * starting_stack_size as f32);
                    log::info!(
                        "Score: {}, defender score {}, challenger score {}, starting stack size {}",
                        score,
                        defender_score,
                        challenger_score,
                        starting_stack_size
                    );

                    let (defender_bot, defender_team) = shared::db::schema::bots::dsl::bots
                        .find(game.defender)
                        .inner_join(
                            shared::db::schema::teams::dsl::teams
                                .on(shared::db::schema::bots::dsl::team
                                    .eq(shared::db::schema::teams::dsl::id)),
                        )
                        .first::<(Bot, Team)>(db_conn)?;
                    let (challenger_bot, challenger_team) = shared::db::schema::bots::dsl::bots
                        .find(game.challenger)
                        .inner_join(
                            shared::db::schema::teams::dsl::teams
                                .on(shared::db::schema::bots::dsl::team
                                    .eq(shared::db::schema::teams::dsl::id)),
                        )
                        .first::<(Bot, Team)>(db_conn)?;

                    // Don't rate games that had an internal error
                    (defender_rating_change, challenger_rating_change) = get_rating_change(
                        defender_team.rating,
                        score,
                        challenger_team.rating,
                        1.0 - score,
                    );
                    // don't update rating for internal errors
                    match error_type {
                        Some(GameError::InternalError) => {
                            (defender_rating_change, challenger_rating_change) = (0.0, 0.0);
                        }
                        _ => {}
                    }

                    // Update rating
                    let defender: Team = diesel::update(teams::table.find(defender_team.id))
                        .set(teams::dsl::rating.eq(teams::dsl::rating + defender_rating_change))
                        .get_result::<Team>(db_conn)?;
                    let challenger: Team = diesel::update(teams::table.find(challenger_team.id))
                        .set(teams::dsl::rating.eq(teams::dsl::rating + challenger_rating_change))
                        .get_result::<Team>(db_conn)?;
                    log::debug!(
                        "Defender (+{}): {:?}, challenger (+{}): {:?}",
                        defender_rating_change,
                        defender,
                        challenger_rating_change,
                        challenger
                    );

                    let new_result = models::NewGameResult {
                        id: id.clone(),
                        challenger_rating_change,
                        defender_rating_change,
                        defender_score,
                        challenger_score,
                        error_type: error_type.clone(),
                        challenger_rating: challenger.rating,
                        defender_rating: defender.rating,
                    };
                    diesel::insert_into(game_results::dsl::game_results)
                        .values(&new_result)
                        .on_conflict(game_results::dsl::id)
                        .do_update()
                        .set(&new_result)
                        .execute(db_conn)?;
                    log::debug!("Inserted game result for {}", id.clone());
                }
                Ok(GameStatus::TestGameSucceeded) => {
                    // set the active bot for the team if they don't have one
                    let (bot, team): (Bot, Team) = shared::db::schema::bots::dsl::bots
                        .find(
                            id.parse::<i32>()
                                .map_err(|_| diesel::result::Error::RollbackTransaction)?,
                        )
                        .inner_join(
                            shared::db::schema::teams::dsl::teams
                                .on(shared::db::schema::bots::dsl::team
                                    .eq(shared::db::schema::teams::dsl::id)),
                        )
                        .first::<(Bot, Team)>(db_conn)
                        .map_err(|e| {
                            log::debug!("{:?}", e);
                            e
                        })?;
                    log::debug!("Bot: {:?}, team: {:?}", bot, team);

                    // set the active bot for the team if they don't have one
                    //if team.active_bot.is_none() {
                    diesel::update(shared::db::schema::teams::dsl::teams)
                        .filter(shared::db::schema::teams::dsl::id.eq(team.id))
                        .set(shared::db::schema::teams::dsl::active_bot.eq(bot.id))
                        .execute(db_conn)?;
                    //}
                    diesel::update(bots::dsl::bots)
                        .filter(
                            bots::dsl::id.eq(id
                                .parse::<i32>()
                                .map_err(|_| diesel::result::Error::RollbackTransaction)?),
                        )
                        .set(
                            bots::dsl::build_status
                                .eq(shared::BuildStatus::TestGameSucceeded as i32),
                        )
                        .execute(db_conn)?;
                }
                Ok(GameStatus::TestGameFailed) => {
                    diesel::update(bots::dsl::bots)
                        .filter(
                            bots::dsl::id.eq(id
                                .parse::<i32>()
                                .map_err(|_| diesel::result::Error::RollbackTransaction)?),
                        )
                        .set(bots::dsl::build_status.eq(shared::BuildStatus::TestGameFailed as i32))
                        .execute(db_conn)?;
                }
            };
            Ok::<(), diesel::result::Error>(())
        })
        .map_err(|_| ())?;

    // Don't fail if we can't save the game details
    if let Err(_) = save_game_details(id.clone()).await {
        log::error!("Failed to save game details for {}", id);
    }

    Ok(())
}
