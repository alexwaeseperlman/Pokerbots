use std::{error::Error, process::Stdio};

use builder::bots::build_bot;
use shared::{sqs::listen_on_queue, BuildStatus, BuildTask};
use tokio::{fs, process::Command};

async fn process(
    BuildTask { bot, log_presigned }: BuildTask,
    s3: &aws_sdk_s3::Client,
    reqwest_client: &reqwest::Client,
) -> Result<(), Box<dyn Error>> {
    let bot_bucket = std::env::var("BOT_S3_BUCKET")?;
    let compiled_bot_bucket = std::env::var("COMPILED_BOT_S3_BUCKET")?;
    fs::create_dir_all(format!("/tmp/{}", bot)).await?;
    let bot_path = std::path::Path::new("/tmp").join(&bot);
    // Command::new("mount")
    //     .arg("-t")
    //     .arg("tmpfs")
    //     .arg("-o")
    //     .arg("rw,size=2G")
    //     .arg(format!("{}", bot))
    //     .arg(format!("/tmp/{}", bot))
    //     .stderr(Stdio::null())
    //     .stdout(Stdio::null())
    //     .current_dir(format!("/tmp/{}", bot))
    //     .status()
    //     .await?;
    shared::s3::download_file(&bot, &bot_path.join("bot.zip"), &bot_bucket, &s3).await?;
    let result = build_bot(bot_path).await;
    // upload the logs
    let log = fs::read(format!("/tmp/{}/logs", bot)).await?;
    if let Err(e) = reqwest_client
        .put(log_presigned.url)
        .headers(log_presigned.headers.into())
        .body(log)
        .send()
        .await
    {
        log::error!("Failed to upload logs to s3: {}", e);
    }
    if result.is_err() {
        result?;
    }
    // zip up the bot
    log::debug!("Uploaded logs for {}", bot);
    Command::new("zip")
        .arg("-r")
        .arg("compiled_bot.zip")
        .arg("bot")
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .current_dir(format!("/tmp/{}/", bot))
        .status()
        .await?;
    log::debug!("Zipped bot");
    // upload the file to s3
    // TODO: this should use a presigned url, like the logs
    if let Err(e) = s3
        .put_object()
        .bucket(compiled_bot_bucket)
        .key(format!("{}", &bot))
        .body(
            fs::read(format!("/tmp/{}/compiled_bot.zip", &bot))
                .await?
                .into(),
        )
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
    let reqwest_client = reqwest::Client::new();
    log::info!("Listening for messages.");
    listen_on_queue(
        std::env::var("BOT_UPLOADS_QUEUE_URL").unwrap(),
        &sqs,
        |task: BuildTask| async {
            log::warn!("Received build task for {}", task.bot);
            // TODO: send a message when the build starts
            // right now we just send a message when it finishes
            let result = process(task.clone(), &s3, &reqwest_client).await;

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
                    Err(_) => true,
                } {
                    log::error!("Error sending message.");
                    return false;
                } else {
                    log::info!("Message sent.");
                }
            } else if let Err(e) = body {
                log::error!("Error serializing message {}", e);
                return false;
            }
            true
        },
        |e| {
            log::error!("Error receiving message {}", e);
        },
    )
    .await;
}
