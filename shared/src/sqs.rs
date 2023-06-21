use std::{error::Error, future::Future};

pub async fn listen_on_queue<
    T: AsRef<str>,
    PayloadType: serde::de::DeserializeOwned,
    U: Fn(PayloadType) -> Fut,
    Fut: Future<Output = ()>,
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
            .wait_time_seconds(10)
            .queue_url(queue.as_ref())
            .send()
            .await;
        if let Some(payload) = match message.map(|m| m.messages) {
            Ok(Some(result)) => result,
            Err(e) => {
                err_cb(Box::new(e));
                continue;
            }
            _ => {
                continue;
            }
        }
        .first()
        {
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
                    continue;
                }
            };
            cb(task).await;
        }
    }
}
