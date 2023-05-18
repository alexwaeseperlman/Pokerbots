use std::sync::Arc;

use diesel::*;
use futures_util::StreamExt;
use lapin::Channel;
use shared::GameResult;

use crate::{config::DB_CONNECTION, schema::games};

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
            if let Ok(payload) = serde_json::from_slice::<GameResult>(&delivery.data) {
                log::debug!("Message received {:?}", payload);
                // Update the database
                diesel::update(games::table.find(payload.id))
                    .set(games::dsl::score_change.eq(payload.score_change))
                    .execute(db_conn)
                    .unwrap();
                //TODO: Handle elo changes
                channel
                    .basic_ack(
                        delivery.delivery_tag,
                        lapin::options::BasicAckOptions::default(),
                    )
                    .await;
            } else {
                channel
                    .basic_nack(
                        delivery.delivery_tag,
                        lapin::options::BasicNackOptions::default(),
                    )
                    .await;
            }
        }
    }
}
