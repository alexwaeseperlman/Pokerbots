use diesel::prelude::*;
use shared::{BuildResultMessage, BuildStatus, GameTask};

/// Handle a build result message.
/// This function should update the database with the build result
/// and send a notification to the events queue.
///
/// If the build result is successful (BuildStatus::BuildSucceeded), it should also queue a test game
/// for the bot.
pub async fn handle_build_result(
    result: BuildResultMessage,
    sqs: &aws_sdk_sqs::Client,
) -> Result<(), ()> {
    use shared::db::schema::bots::dsl::*;
    let conn = &mut (*shared::db::conn::DB_CONNECTION.get().map_err(|_| ())?);
    // update bot with build result
    diesel::update(bots)
        .filter(id.eq(result.bot.parse::<i32>().map_err(|_| ())?))
        .set(build_status.eq::<i32>(result.status.clone() as i32))
        .execute(conn)
        .map_err(|_| ())?;
    log::info!(
        "Updated bot {} with build result {:?}",
        result.bot,
        result.status
    );

    match result.status {
        BuildStatus::BuildSucceeded => {
            // Queue a test game
            let task = GameTask::TestGame { bot: result.bot };
            sqs.send_message()
                .queue_url(std::env::var("NEW_GAMES_QUEUE_URL").unwrap())
                .message_body(serde_json::to_string(&task).unwrap())
                .send()
                .await
                .map_err(|_| ())?;
        }
        _ => {}
    }
    Ok(())
}
