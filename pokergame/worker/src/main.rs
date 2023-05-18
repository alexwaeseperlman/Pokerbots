use std::process::Command;

pub mod bots;
pub mod poker;
use futures_lite::stream::StreamExt;
use lapin::options::{BasicAckOptions, BasicRejectOptions};
use shared::PlayTask;

#[tokio::main]
async fn main() {
    env_logger::init();
    let addr = std::env::var("AMQP_URL").expect("AMQP_URL must be set");
    let conn = lapin::Connection::connect(&addr, lapin::ConnectionProperties::default())
        .await
        .expect("Connection error");

    // listen for messages
    let channel = conn.create_channel().await.unwrap();
    let queue = channel
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
    while let Some(msg) = consumer.next().await {
        let msg = msg.expect("Error while receiving message");
        if let Ok(payload) = serde_json::from_slice::<PlayTask>(&msg.data) {
            log::debug!("Message received {:?}", payload);
            std::thread::sleep(std::time::Duration::from_secs(5));
            msg.ack(BasicAckOptions::default())
                .await
                .expect("Error while acknowledging message");
        } else {
            msg.reject(BasicRejectOptions::default())
                .await
                .expect("Error while parsing message");
        }
    }
}
