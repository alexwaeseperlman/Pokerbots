use std::process::Command;

use futures_lite::stream::StreamExt;
use lapin::options::{BasicAckOptions, BasicRejectOptions};
use shared::{GameResult, PlayTask};

use rand::Rng;
#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("Starting worker");
    let addr = std::env::var("AMQP_URL").expect("AMQP_URL must be set");
    let conn = lapin::Connection::connect(&addr, lapin::ConnectionProperties::default())
        .await
        .expect("Connection error");

    // listen for messages
    let channel = conn.create_channel().await.unwrap();
    let channel_b = conn.create_channel().await.unwrap();
    channel_b
        .queue_declare(
            "game_results",
            lapin::options::QueueDeclareOptions::default(),
            lapin::types::FieldTable::default(),
        )
        .await
        .unwrap();
    channel
        .queue_declare(
            "poker",
            lapin::options::QueueDeclareOptions::default(),
            lapin::types::FieldTable::default(),
        )
        .await
        .unwrap();

    let mut consumer = channel
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
            let result: GameResult =
                pokergame::bots::run_game(payload.bot_a, payload.bot_b, &s3_client, payload.id)
                    .await;
            channel_b
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
    }
}
