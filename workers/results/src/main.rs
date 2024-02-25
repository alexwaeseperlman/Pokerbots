use aws_sdk_s3::presigning::PresigningConfig;
use diesel::prelude::*;
use results::build_result::handle_build_result;
use results::game_result::{handle_game_result, save_game_details};
use shared::sqs::listen_on_queue;
use shared::{BuildResultMessage, GameStatusMessage};

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenvy::dotenv().ok();
    let config = shared::aws_config().await;
    let s3 = shared::s3_client(&config).await;
    let sqs = shared::sqs_client(&config).await;

    log::info!("Listening for messages.");
    tokio::join!(
        listen_on_queue(
            std::env::var("BUILD_RESULTS_QUEUE_URL").unwrap(),
            &sqs,
            |task: BuildResultMessage| async {
                log::info!("Received build result: {:?}", task);

                if let Ok(presign_config) =
                    PresigningConfig::expires_in(std::time::Duration::from_secs(60 * 60 * 24 * 7))
                {
                    if let Ok(log_presigned) = s3
                        .put_object()
                        .bucket(std::env::var("BUILD_LOGS_S3_BUCKET").unwrap())
                        .key(format!("{}/test_game", task.bot))
                        .presigned(presign_config.clone())
                        .await
                    {
                        handle_build_result(
                            task,
                            &sqs,
                            shared::PresignedRequest {
                                url: log_presigned.uri().to_string(),
                                headers: log_presigned.headers().into(),
                            },
                        )
                        .await
                        .is_ok()
                    } else {
                        log::error!("Failed to create presigned url for logs");
                        false
                    }
                } else {
                    log::error!("Failed to create presigning config for logs");
                    false
                }
            },
            |err| log::error!("Error receiving build result: {}", err),
        ),
        listen_on_queue(
            std::env::var("GAME_RESULTS_QUEUE_URL").unwrap(),
            &sqs,
            |task: GameStatusMessage| async move {
                log::info!("Received game result: {:?}", task);
                let id = task.id.clone();
                let result = handle_game_result(task).await.is_ok();
                if let Ok(mut db_conn) = shared::db::conn::DB_CONNECTION.get() {
                    let db_conn = &mut (*db_conn);
                    if let Err(_) = diesel::update(shared::db::schema::games::table.find(id.clone()))
                        .set(shared::db::schema::games::dsl::running.eq(false))
                        .execute(db_conn) {
                        log::error!("Failed to update running status of {}", id.clone());
                    }
                }
                else {
                        log::error!("Failed to update running status of {}", id.clone());

                }
                result
            },
            |err| log::error!("Error receiving game result: {}", err),
        ),
        results::matchmaking::matchmake(&s3, &sqs)
    );
}
