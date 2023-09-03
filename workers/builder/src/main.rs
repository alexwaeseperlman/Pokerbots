use std::{error::Error, process::Stdio, sync::Arc};

use builder::bots::build_bot;
use shared::{sqs::listen_on_queue, BuildStatus, BuildTask};
use tokio::{fs, process::Command};

async fn process(
    BuildTask { bot, log_presigned }: &BuildTask,
    s3: &aws_sdk_s3::Client,
    reqwest_client: &reqwest::Client,
) -> Result<(), Box<dyn Error>> {
    let bot_bucket = std::env::var("BOT_S3_BUCKET")?;
    let compiled_bot_bucket = std::env::var("COMPILED_BOT_S3_BUCKET")?;
    fs::create_dir_all(format!("/tmp/{}", bot)).await?;
    let bot_path = std::path::Path::new("/tmp").join(&bot.to_string());
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
    shared::s3::download_file(
        &bot.to_string(),
        &bot_path.join("bot.zip"),
        &bot_bucket,
        &s3,
    )
    .await?;
    let result = build_bot(bot_path).await;
    // upload the logs
    let log = fs::read(format!("/tmp/{}/logs", bot)).await?;
    match reqwest_client
        .put(&log_presigned.url)
        .headers(log_presigned.headers.clone().into())
        .body(log)
        .send()
        .await
    {
        Err(e) => {
            log::error!("Failed to upload logs to s3: {}", e);
        }
        _ => (),
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
    let s3 = Arc::from(shared::s3_client(&config).await);
    let sqs = Arc::from(shared::sqs_client(&config).await);
    let reqwest_client = Arc::from(reqwest::Client::new());
    log::info!("Listening for messages.");
    listen_on_queue(
        std::env::var("BOT_UPLOADS_QUEUE_URL").unwrap(),
        &sqs.clone(),
        |task: BuildTask| {
            let s3 = s3.clone();
            let reqwest_client = reqwest_client.clone();
            let sqs = sqs.clone();

            async move {
                log::warn!("Received build task for {}", task.bot);
                // TODO: send a message when the build starts
                // right now we just send a message when it finishes
                let result = process(&task, &s3, &reqwest_client).await;

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
            }
        },
        |e| {
            log::error!("Error receiving message {}", e);
        },
    )
    .await;
}
