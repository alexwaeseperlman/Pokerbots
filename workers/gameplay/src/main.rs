use std::process::Command;

use futures_lite::stream::StreamExt;
use gameplay::bots::run_game;
use rand::Rng;
use shared::{GameResult, GameStatus, GameStatusMessage, GameTask};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();
    log::info!("Starting worker");

    let config = shared::aws_config().await;
    let sqs = shared::sqs_client(&config).await;
    let s3 = shared::s3_client(&config).await;
    let game_results_queue = std::env::var("GAME_RESULTS_QUEUE_URL").unwrap();

    shared::sqs::listen_on_queue(
        std::env::var("NEW_GAMES_QUEUE_URL").unwrap(),
        &sqs,
        |message: GameTask| async {
            log::info!("Received message: {:?}", message);
            let result = match message.clone() {
                GameTask::Game {
                    bot_a,
                    bot_b,
                    id,
                    date,
                    rounds,
                } => run_game(&bot_a, &bot_b, &s3, &id, rounds).await,
                GameTask::TestGame { bot } => {
                    if let Err(e) = run_game(&bot, &bot, &s3, &bot, 5).await {
                        Ok(GameStatus::TestGameFailed)
                    } else {
                        Ok(GameStatus::TestGameSucceeded)
                    }
                }
            };
            sqs.send_message()
                .queue_url(&game_results_queue)
                .message_body(
                    serde_json::to_string::<GameStatusMessage>(&GameStatusMessage {
                        id: match message {
                            GameTask::Game { id, .. } => id,
                            GameTask::TestGame { bot } => bot,
                        },
                        result,
                    })
                    .unwrap(),
                )
                .send()
                .await
                .is_ok()
        },
        |err| {
            log::error!("Error receiving message: {}", err);
        },
    )
    .await;
}
