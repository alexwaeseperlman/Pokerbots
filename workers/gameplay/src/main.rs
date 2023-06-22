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
                .unwrap();
        },
        |err| {
            log::error!("Error receiving message: {}", err);
        },
    )
    .await;

    /*let addr = std::env::var("AMQP_ADDRESS").expect("AMQP_ADDRESS must be set");
    let conn = lapin::Connection::connect(&addr, lapin::ConnectionProperties::default())
        .await
        .expect("Connection error");

    // listen for messages
    let games_channel = conn.create_channel().await.unwrap();
    let results_channel = conn.create_channel().await.unwrap();
    results_channel
        .queue_declare(
            "game_results",
            lapin::options::QueueDeclareOptions::default(),
            lapin::types::FieldTable::default(),
        )
        .await
        .unwrap();
    games_channel
        .queue_declare(
            "poker",
            lapin::options::QueueDeclareOptions::default(),
            lapin::types::FieldTable::default(),
        )
        .await
        .unwrap();

    let mut consumer = games_channel
        .basic_consume(
            "poker",
            "worker",
            lapin::options::BasicConsumeOptions::default(),
            lapin::types::FieldTable::default(),
        )
        .await
        .unwrap();

    let aws_config = aws_config::load_from_env().await;
    let s3_client = aws_sdk_s3::Client::new(&aws_config);

    while let Some(msg) = consumer.next().await {
        let msg = msg.expect("Error while receiving message");
        if let Ok(payload) = serde_json::from_slice::<PlayTask>(&msg.data) {
            log::debug!("Message received {:?}", payload);
            std::thread::sleep(std::time::Duration::from_secs(5));
            msg.ack(BasicAckOptions::default())
                .await
                .expect("Error while acknowledging message");
            let result: GameResult = gameplay::bots::run_game(
                payload.bot_a,
                payload.bot_b,
                &s3_client,
                payload.id.clone(),
            )
            .await;
            let result: GameMessage = GameMessage {
                result,
                id: payload.id,
            };
            results_channel
                .basic_publish(
                    "",
                    "game_results",
                    lapin::options::BasicPublishOptions::default(),
                    &serde_json::to_vec(&result).unwrap(),
                    lapin::BasicProperties::default(),
                )
                .await
                .unwrap()
                .await
                .unwrap();
        } else {
            msg.reject(BasicRejectOptions::default())
                .await
                .expect("Error while parsing message");
        }
    }*/
}
