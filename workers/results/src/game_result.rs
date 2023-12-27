use diesel::prelude::*;
use shared::{
    db::{
        models::{self, Bot, Game, Team},
        schema::{game_results, teams},
    },
    GameError, GameStatus, GameStatusMessage,
};

use crate::rating::get_rating_change;

pub async fn handle_game_result(status: GameStatusMessage) -> Result<(), ()> {
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
            GameError::InternalError => (50, 50),
            GameError::InvalidActionError(which_bot) => match which_bot {
                shared::WhichBot::Defender => (-100, 100),
                shared::WhichBot::Challenger => (100, -100),
            },
            GameError::MemoryError(which_bot) => match which_bot {
                shared::WhichBot::Defender => (-100, 100),
                shared::WhichBot::Challenger => (100, -100),
            },
            GameError::RunTimeError(which_bot) => match which_bot {
                shared::WhichBot::Defender => (-100, 100),
                shared::WhichBot::Challenger => (100, -100),
            },
            GameError::TimeoutError(which_bot) => match which_bot {
                shared::WhichBot::Defender => (-100, 100),
                shared::WhichBot::Challenger => (100, -100),
            },
        },
    };
    let (mut defender_rating_change, mut challenger_rating_change) = (0f32, 0f32);

    // transaction
    db_conn
        .transaction(move |db_conn| {
            match result {
                Ok(GameStatus::ScoreChanged(_, _)) | Err(_) => {
                    let game: Game =
                        games::table.find(&id).first::<Game>(db_conn).map_err(|e| {
                            log::error!("Failed to find game with id {}", id);
                            e
                        })?;
                    // calculate the bots ratings
                    let score = (50.0 + defender_score as f32) / (100.0f32);
                    log::info!(
                        "Score: {}, defender score {}, challenger score {}",
                        score,
                        defender_score,
                        challenger_score
                    );
                    // Don't rate games that had an internal error
                    (defender_rating_change, challenger_rating_change) = get_rating_change(
                        game.defender_rating,
                        score,
                        game.challenger_rating,
                        1.0 - score,
                    );
                    // don't update rating for internal errors
                    match error_type {
                        Some(GameError::InternalError) => {
                            (defender_rating_change, challenger_rating_change) = (0.0, 0.0);
                        }
                        _ => {}
                    }

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

                    diesel::insert_into(game_results::table)
                        .values(models::NewGameResult {
                            id,
                            challenger_rating_change,
                            defender_rating_change,
                            defender_score,
                            challenger_score,
                            error_type: error_type.clone(),
                            challenger_rating: challenger.rating,
                            defender_rating: defender.rating,
                        })
                        .execute(db_conn)?;
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
    Ok(())
}
