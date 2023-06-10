use std::sync::Arc;

use diesel::*;
use futures_util::StreamExt;
use lapin::Channel;
use shared::{GameError, GameMessage, GameResult, ScoringResult};

use crate::{
    config::DB_CONNECTION,
    schema::{self, games},
};

pub async fn listen_for_game_results(channel: Channel) {
    let mut consumer = channel
        .basic_consume(
            "game_results",
            "game_result_consumer",
            lapin::options::BasicConsumeOptions::default(),
            lapin::types::FieldTable::default(),
        )
        .await
        .unwrap();

    let db_conn = &mut (*DB_CONNECTION).get().unwrap();
    while let Some(delivery) = consumer.next().await {
        log::debug!("Message received {:?}", delivery);
        if let Ok(delivery) = delivery {
            if let Ok(payload) = serde_json::from_slice::<GameMessage>(&delivery.data) {
                log::debug!("Message received {:?}", payload);
                let mut score_change = 0;
                let mut message: String = "".into();
                // CREATE TYPE game_error AS ENUM ('RUNTIME', 'COMPILE', 'TIMEOUT', 'MEMORY', 'INTERNAL', 'INVALID_ACTION', 'UNKNOWN');
                let mut error_type: Option<String> = None;
                match payload.result {
                    GameResult::Ok(ScoringResult::ScoreChanged(score_change)) => {
                        diesel::update(games::table.find(payload.id.clone()))
                            .set(games::dsl::score_change.eq(score_change))
                            .execute(db_conn)
                            .unwrap();
                    }
                    GameResult::Err(e) => {
                        // Penalize a bot if it was their fault
                        match e {
                            GameError::CompileError(err, which_bot) => {
                                error_type = Some("COMPILE".into());
                                message = err;
                                score_change = match which_bot {
                                    shared::WhichBot::BotA => -100,
                                    shared::WhichBot::BotB => 100,
                                };
                            }
                            GameError::InternalError(err) => {
                                error_type = Some("INTERNAL".into());
                                message = err;
                            }
                            GameError::InvalidActionError(err, which_bot) => {
                                error_type = Some("INVALID_ACTION".into());
                                message = format!("{:?}", err);
                                score_change = match which_bot {
                                    shared::WhichBot::BotA => -100,
                                    shared::WhichBot::BotB => 100,
                                };
                            }
                            GameError::MemoryError(err, which_bot) => {
                                error_type = Some("MEMORY".into());
                                message = err;
                                score_change = match which_bot {
                                    shared::WhichBot::BotA => -100,
                                    shared::WhichBot::BotB => 100,
                                };
                            }
                            GameError::RunTimeError(err, which_bot) => {
                                error_type = Some("RUNTIME".into());
                                message = err;
                                score_change = match which_bot {
                                    shared::WhichBot::BotA => -100,
                                    shared::WhichBot::BotB => 100,
                                };
                            }
                            GameError::TimeoutError(err, which_bot) => {
                                error_type = Some("TIMEOUT".into());
                                message = err;
                                score_change = match which_bot {
                                    shared::WhichBot::BotA => -100,
                                    shared::WhichBot::BotB => 100,
                                };
                            }
                        }
                    }
                    _ => {
                        log::error!("Unexpected game result {:?}", payload.result);
                    }
                }
                if let Err(e) = diesel::update(games::dsl::games)
                    .filter(games::dsl::id.eq(payload.id))
                    .set((
                        games::dsl::score_change.eq(score_change),
                        games::dsl::error_type.eq(error_type),
                        games::dsl::error_message.eq(message),
                    ))
                    .execute(db_conn)
                {
                    log::error!("Failed to update database for game result {}.", e);
                }

                //TODO: Handle elo changes
                if let Err(e) = channel
                    .basic_ack(
                        delivery.delivery_tag,
                        lapin::options::BasicAckOptions::default(),
                    )
                    .await
                {
                    log::error!("Failed to update ack for game result {}.", e);
                }
            } else {
                if let Err(e) = channel
                    .basic_nack(
                        delivery.delivery_tag,
                        lapin::options::BasicNackOptions::default(),
                    )
                    .await
                {
                    log::error!("Failed to update ack for game result {}.", e);
                }
            }
        }
    }
}
