use std::path::PathBuf;

use gameplay::bots::run_game;
use shared::{GameError, GameStatus, GameStatusMessage, GameTask};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();
    log::info!("Starting gameplay worker");

    let config = shared::aws_config().await;
    let sqs = shared::sqs_client(&config).await;
    let s3 = shared::s3_client(&config).await;
    let game_results_queue = std::env::var("GAME_RESULTS_QUEUE_URL").unwrap();
    let reqwest_client = reqwest::Client::new();

    shared::sqs::listen_on_queue(
        std::env::var("NEW_GAMES_QUEUE_URL").unwrap(),
        &sqs,
        |message: GameTask| async {
            log::info!("Received message: {:?}", message);
            let result = match message.clone() {
                GameTask::Game {
                    defender,
                    challenger,
                    id,
                    rounds,
                    public_logs_presigned,
                    defender_logs_presigned,
                    challenger_logs_presigned,
                } => {
                    let result = run_game(defender, challenger, &s3, &id, rounds).await;

                    match result {
                        Err(e) => {
                            log::error!("Game failed: {:?}", e);
                            Err(GameError::InternalError)
                        }
                        Ok(result) => {
                            // upload logs
                            // ignore if they have errors
                            if let Err(e) = tokio::try_join!(
                                reqwest_client
                                    .put(defender_logs_presigned.url)
                                    .headers(defender_logs_presigned.headers.into())
                                    .body(result.defender_log)
                                    .send(),
                                reqwest_client
                                    .put(challenger_logs_presigned.url)
                                    .headers(challenger_logs_presigned.headers.into())
                                    .body(result.challenger_log)
                                    .send(),
                                reqwest_client
                                    .put(public_logs_presigned.url)
                                    .headers(public_logs_presigned.headers.into())
                                    .body(result.public_log)
                                    .send(),
                            ) {
                                log::error!("Error uploading logs: {:?}", e);
                            };
                            result.status
                        }
                    }
                }
                GameTask::TestGame { bot, log_presigned } => {
                    let mut path = PathBuf::default();
                    if let Err(_) = run_game(bot, bot, &s3, &bot.to_string(), 5).await {
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
                            GameTask::TestGame { bot, .. } => bot.to_string(),
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
