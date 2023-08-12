use diesel::prelude::*;
use shared::{GameError, GameStatus, GameStatusMessage};

pub async fn handle_game_result(status: GameStatusMessage) -> Result<(), ()> {
    use shared::db::schema::{bots, games};
    let db_conn = &mut (*shared::db::conn::DB_CONNECTION.get().map_err(|_| ())?);
    let mut error_type: Option<String> = None;
    let mut score_change: i32 = 0;
    let GameStatusMessage { id, result } = status;
    match result {
        Ok(GameStatus::ScoreChanged(score_change)) => {
            diesel::update(games::table.find(&id))
                .set(games::dsl::score_change.eq(score_change))
                .execute(db_conn)
                .map_err(|e| ())?;
        }
        Ok(GameStatus::TestGameSucceeded) => {
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
                    score_change = match which_bot {
                        shared::WhichBot::Defender => -100,
                        shared::WhichBot::Challenger => 100,
                    };
                }
                GameError::MemoryError(which_bot) => {
                    error_type = Some("MEMORY".into());
                    score_change = match which_bot {
                        shared::WhichBot::Defender => -100,
                        shared::WhichBot::Challenger => 100,
                    };
                }
                GameError::RunTimeError(which_bot) => {
                    error_type = Some("RUNTIME".into());
                    score_change = match which_bot {
                        shared::WhichBot::Defender => -100,
                        shared::WhichBot::Challenger => 100,
                    };
                }
                GameError::TimeoutError(which_bot) => {
                    error_type = Some("TIMEOUT".into());
                    score_change = match which_bot {
                        shared::WhichBot::Defender => -100,
                        shared::WhichBot::Challenger => 100,
                    };
                }
            }
            diesel::update(games::table.find(id))
                .set((
                    games::dsl::score_change.eq(score_change),
                    games::dsl::error_type.eq(error_type),
                ))
                .execute(db_conn)
                .map_err(|_| ())?;
        }
    };
    Ok(())
}
