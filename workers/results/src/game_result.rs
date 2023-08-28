use diesel::prelude::*;
use shared::{
    db::models::{Bot, Team},
    GameError, GameStatus, GameStatusMessage,
};

pub async fn handle_game_result(status: GameStatusMessage) -> Result<(), ()> {
    use shared::db::schema::{bots, games};
    let db_conn = &mut (*shared::db::conn::DB_CONNECTION.get().map_err(|_| ())?);
    let error_type: Option<String>;
    let mut defender_score: i32 = 0;
    let mut challenger_score: i32 = 0;
    let GameStatusMessage { id, result } = status;
    match result {
        Ok(GameStatus::ScoreChanged(defender_score, challenger_score)) => {
            diesel::update(games::table.find(&id))
                .set((
                    games::dsl::defender_score.eq(defender_score),
                    games::dsl::challenger_score.eq(challenger_score),
                ))
                .execute(db_conn)
                .map_err(|e| ())?;
        }
        Ok(GameStatus::TestGameSucceeded) => {
            // set the active bot for the team if they don't have one
            let (bot, team): (Bot, Team) =
                shared::db::schema::bots::dsl::bots
                    .find(id.parse::<i32>().map_err(|_| ())?)
                    .inner_join(shared::db::schema::teams::dsl::teams.on(
                        shared::db::schema::bots::dsl::team.eq(shared::db::schema::teams::dsl::id),
                    ))
                    .first::<(Bot, Team)>(db_conn)
                    .map_err(|_| ())?;
            log::debug!("Bot: {:?}, team: {:?}", bot, team);
            if team.active_bot.is_none() {
                diesel::update(shared::db::schema::teams::dsl::teams)
                    .filter(shared::db::schema::teams::dsl::id.eq(team.id))
                    .set(shared::db::schema::teams::dsl::active_bot.eq(bot.id))
                    .execute(db_conn)
                    .map_err(|_| ())?;
            }
            diesel::update(bots::dsl::bots)
                .filter(bots::dsl::id.eq(id.parse::<i32>().map_err(|_| ())?))
                .set(bots::dsl::build_status.eq(shared::BuildStatus::TestGameSucceeded as i32))
                .execute(db_conn)
                .map_err(|_| ())?;
        }
        Ok(GameStatus::TestGameFailed) => {
            diesel::update(bots::dsl::bots)
                .filter(bots::dsl::id.eq(id.parse::<i32>().map_err(|_| ())?))
                .set(bots::dsl::build_status.eq(shared::BuildStatus::TestGameFailed as i32))
                .execute(db_conn)
                .map_err(|_| ())?;
        }
        Err(e) => {
            // Penalize a bot if it was their fault
            match e {
                GameError::InternalError => {
                    error_type = Some("INTERNAL".into());
                }
                GameError::InvalidActionError(which_bot) => {
                    error_type = Some("INVALID_ACTION".into());
                    (defender_score, challenger_score) = match which_bot {
                        shared::WhichBot::Defender => (-100, 0),
                        shared::WhichBot::Challenger => (0, -100),
                    };
                }
                GameError::MemoryError(which_bot) => {
                    error_type = Some("MEMORY".into());
                    (defender_score, challenger_score) = match which_bot {
                        shared::WhichBot::Defender => (-100, 0),
                        shared::WhichBot::Challenger => (0, -100),
                    };
                }
                GameError::RunTimeError(which_bot) => {
                    error_type = Some("RUNTIME".into());
                    (defender_score, challenger_score) = match which_bot {
                        shared::WhichBot::Defender => (-100, 0),
                        shared::WhichBot::Challenger => (0, -100),
                    }
                }
                GameError::TimeoutError(which_bot) => {
                    error_type = Some("TIMEOUT".into());
                    (defender_score, challenger_score) = match which_bot {
                        shared::WhichBot::Defender => (-100, 0),
                        shared::WhichBot::Challenger => (0, -100),
                    }
                }
            }
            diesel::update(games::table.find(id))
                .set((
                    games::dsl::defender_score.eq(defender_score),
                    games::dsl::challenger_score.eq(challenger_score),
                    games::dsl::error_type.eq(error_type),
                ))
                .execute(db_conn)
                .map_err(|_| ())?;
        }
    };
    Ok(())
}
