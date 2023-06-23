use results::build_result::handle_build_result;
use results::game_result::handle_game_result;
use shared::sqs::listen_on_queue;
use shared::{BuildResultMessage, GameResult, GameStatusMessage};

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
                handle_build_result(task, &sqs).await.is_ok()
            },
            |err| log::error!("Error receiving build result: {}", err),
        ),
        listen_on_queue(
            std::env::var("GAME_RESULTS_QUEUE_URL").unwrap(),
            &sqs,
            |task: GameStatusMessage| async move {
                log::info!("Received game result: {:?}", task);
                handle_game_result(task).await.is_ok()
            },
            |err| log::error!("Error receiving game result: {}", err),
        )
    );
}
