use std::{error::Error, process::Stdio, time::Duration};

use builder::bots::{build_bot, download_bot};
use shared::{BuildStatus, BuildTask};
use tokio::{fs, process::Command, time::sleep};

async fn process(
    BuildTask { bot }: BuildTask,
    s3: aws_sdk_s3::Client,
) -> Result<(), Box<dyn Error>> {
    let bot_bucket = std::env::var("BOT_S3_BUCKET")?;
    let compiled_bot_bucket = std::env::var("COMPILED_BOT_S3_BUCKET")?;
    fs::create_dir_all(format!("/tmp/{}", bot)).await?;
    let bot_path = std::path::Path::new("/tmp").join(&bot);
    download_bot(&bot, &bot_path, &bot_bucket, &s3).await?;
    build_bot(bot_path).await?;
    // zip up the bot
    Command::new("zip")
        .arg("-r")
        .arg(format!("{}.zip", bot))
        .arg(&bot)
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .current_dir("/tmp")
        .status()
        .await?;
    // upload the file to s3
    if let Err(e) = s3
        .put_object()
        .bucket(compiled_bot_bucket)
        .key(format!("{}", &bot))
        .body(fs::read(format!("/tmp/{}.zip", &bot)).await?.into())
        .send()
        .await
    {
        log::error!("Failed to upload bot to s3: {}", e);
        return Err(e.into());
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenvy::dotenv().ok();
    let config = shared::aws_config().await;
    let s3 = shared::s3_client(&config).await;
    let sqs = shared::sqs_client(&config).await;

    log::info!("Listening for messages.");
    loop {
        let message = match std::env::var("BOT_UPLOADS_QUEUE_URL") {
            Ok(val) => sqs.receive_message().queue_url(val).send().await,
            Err(_) => {
                continue;
            }
        };
        if let Some(payload) = match message.map(|m| m.messages) {
            Ok(Some(result)) => result,
            Err(e) => {
                log::info!("Error receiving message {}", e);
                continue;
            }
            _ => {
                log::debug!("No messages");
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        }
        .first()
        {
            let task = match payload
                .body()
                .map(|b| serde_json::from_str::<BuildTask>(&b))
            {
                Some(Ok(task)) => task,
                Some(Err(e)) => {
                    log::error!("Error parsing message {}", e);
                    continue;
                }
                None => {
                    log::error!("Empty payload");
                    continue;
                }
            };
            log::info!("Received build task for {}", task.bot);
            // TODO: send a message when the build starts
            // right now we just send a message when it finishes
            let result = process(task.clone(), s3.clone()).await;

            let message = shared::BuildResultMessage {
                bot: task.bot,
                status: if result.is_ok() {
                    BuildStatus::BuildSucceeded
                } else {
                    BuildStatus::BuildFailed
                },
                error: if result.is_err() {
                    Some(format!("{}", result.unwrap_err()))
                } else {
                    None
                },
            };
            log::info!("Completed build: {:?}", message);
            let body = serde_json::to_string(&message);
            if let Ok(s) = body {
                if match std::env::var("BUILD_RESULTS_QUEUE_URL") {
                    Ok(url) => sqs
                        .send_message()
                        .queue_url(url)
                        .message_body(s)
                        .send()
                        .await
                        .is_err(),
                    Err(e) => true,
                } {
                    log::error!("Error sending message.");
                } else {
                    log::info!("Message sent.");
                }
            } else if let Err(e) = body {
                log::error!("Error serializing message {}", e);
                continue;
            }
        } else {
            log::info!("No messages");
            continue;
        }
    }
}
