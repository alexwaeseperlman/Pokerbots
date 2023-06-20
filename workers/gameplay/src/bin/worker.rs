use std::process::Command;

use futures_lite::stream::StreamExt;
use lapin::options::{BasicAckOptions, BasicRejectOptions};
use shared::{GameResult, GameResultMessage, PlayTask};

use rand::Rng;
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();
    log::info!("Starting worker");

    let config = shared::aws_config().await;
    let sqs_client = shared::sqs_client(&config).await;
    println!("{:?}", sqs_client.list_queues().send().await.unwrap());
    let new_game_queue_url = sqs_client
        .get_queue_url()
        .queue_name("new_games")
        .send()
        .await
        .expect("Error getting queue url")
        .queue_url
        .unwrap();
    let game_result_queue_url = sqs_client
        .get_queue_url()
        .queue_name("game_results")
        .send()
        .await
        .expect("Error getting queue url")
        .queue_url
        .unwrap();

    loop {
        let message = sqs_client
            .receive_message()
            .queue_url(&new_game_queue_url)
            .send()
            .await;
        if let Some(payload) = match message.map(|m| m.messages) {
            Ok(Some(result)) => result,
            Err(e) => {
                log::error!("Error receiving message {}", e);
                continue;
            }
            _ => {
                log::debug!("No messages");
                continue;
            }
        }
        .first()
        {
            log::info!("Message received {:?}", payload.body());
        } else {
            log::debug!("No messages");
            continue;
        }
    }

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
