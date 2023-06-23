use std::{error::Error, future::Future, time::Duration};

use tokio::time::sleep;

pub async fn listen_on_queue<
    T: AsRef<str>,
    PayloadType: serde::de::DeserializeOwned,
    U: Fn(PayloadType) -> Fut,
    Fut: Future<Output = bool>,
    V: Fn(Box<dyn Error>) -> (),
>(
    queue: T,
    sqs: &aws_sdk_sqs::Client,
    cb: U,
    err_cb: V,
) {
    loop {
        let message = sqs
            .receive_message()
            .queue_url(queue.as_ref())
            .wait_time_seconds(20)
            //.max_number_of_messages(1)
            .send()
            .await;
        log::debug!("Message: {:?}", message);
        let messages = match message.map(|m| m.messages) {
            Ok(Some(result)) => result,
            Err(e) => {
                err_cb(Box::new(e));
                continue;
            }
            _ => {
                continue;
            }
        };
        for payload in messages {
            let task = match payload
                .body()
                .map(|b| serde_json::from_str::<PayloadType>(&b))
            {
                Some(Ok(task)) => task,
                Some(Err(e)) => {
                    err_cb(Box::new(e));
                    continue;
                }
                None => {
                    log::info!("No message body.");
                    continue;
                }
            };
            if cb(task).await {
                log::info!("Ack.");
                sqs.delete_message()
                    .queue_url(queue.as_ref())
                    .receipt_handle(payload.receipt_handle().unwrap())
                    .send()
                    .await
                    .unwrap();
            } else {
                log::info!("Nack.");
            }
        }
    }
}
