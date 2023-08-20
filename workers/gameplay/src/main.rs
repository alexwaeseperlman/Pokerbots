use std::path::PathBuf;

use gameplay::bots::run_game;
use shared::{GameStatus, GameStatusMessage, GameTask};

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
                    let mut path = PathBuf::default();
                    let result =
                        run_game(&defender, &challenger, &s3, &id, rounds, &mut path).await;

                    if let Err(e) = result.clone() {
                        log::error!("Game failed: {:?}", e);
                    }

                    // upload logs
                    // ignore if they have errors
                    tokio::join!(
                        async {
                            if let Ok(log) = tokio::fs::read(path.join("defender/logs")).await {
                                let _ = reqwest_client
                                    .put(defender_logs_presigned.url)
                                    .headers(defender_logs_presigned.headers.into())
                                    .body(log)
                                    .send()
                                    .await;
                            }
                        },
                        async {
                            if let Ok(f) = tokio::fs::read(path.join("challenger/logs")).await {
                                let _ = reqwest_client
                                    .put(challenger_logs_presigned.url)
                                    .headers(challenger_logs_presigned.headers.into())
                                    .body(f)
                                    .send()
                                    .await;
                            }
                        },
                        async {
                            if let Ok(f) = tokio::fs::read(path.join("logs")).await {
                                let _ = reqwest_client
                                    .put(public_logs_presigned.url)
                                    .headers(public_logs_presigned.headers.into())
                                    .body(f)
                                    .send()
                                    .await;
                            }
                        },
                    );
                    result
                }
                GameTask::TestGame { bot, log_presigned } => {
                    let mut path = PathBuf::default();
                    if let Err(_) = run_game(&bot, &bot, &s3, &bot, 5, &mut path).await {
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
                            GameTask::TestGame { bot, .. } => bot,
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
