use shared::db::models::NewBot;
use std::io::Read;

use crate::config::{BOT_S3_BUCKET, BOT_SIZE, BUILD_LOGS_S3_BUCKET};

use super::*;

#[derive(Deserialize)]
pub struct DeleteBot {
    pub id: i32,
}

#[get("/delete-bot")]
pub async fn delete_bot(
    session: Session,
    web::Query::<DeleteBot>(DeleteBot { id }): web::Query<DeleteBot>,
) -> ApiResult<()> {
    use shared::db::schema::bots;
    let team = login::get_team_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;

    let conn = &mut (*DB_CONNECTION).get()?;
    diesel::delete(bots::dsl::bots)
        .filter(bots::dsl::id.eq(id))
        .filter(bots::dsl::team.eq(team.id))
        .execute(conn)?;

    Ok(web::Json(()))
}

#[derive(Deserialize)]
pub struct ActiveBot {
    pub id: Option<i32>,
}
#[get("/set-active-bot")]
pub async fn set_active_bot(
    session: Session,
    web::Query::<ActiveBot>(ActiveBot { id }): web::Query<ActiveBot>,
) -> ApiResult<()> {
    use shared::db::schema::teams;
    let user = login::get_user_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    let team = login::get_team_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;

    let conn = &mut (*DB_CONNECTION).get()?;
    // ensure the bot belongs to the team
    if let Some(id) = id {
        let bot: Vec<Bot> = schema::bots::dsl::bots
            .filter(schema::bots::dsl::id.eq(id))
            .filter(schema::bots::dsl::team.eq(team.id))
            .load::<Bot>(conn)?;
        if bot.len() == 0 {
            return Err(actix_web::error::ErrorUnauthorized(
                "Only the owner can set a bot as active.",
            )
            .into());
        }
    }

    diesel::update(teams::dsl::teams)
        .filter(teams::dsl::id.eq(team.id))
        .filter(teams::dsl::owner.eq(user.clone().email))
        .set(teams::dsl::active_bot.eq(id))
        .execute(conn)?;

    Ok(web::Json(()))
}

#[derive(Serialize, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
pub struct UploadBotResponse {
    id: i32,
}

#[post("/upload-bot")]
pub async fn upload_bot(
    s3_client: actix_web::web::Data<aws_sdk_s3::Client>,
    sqs_client: actix_web::web::Data<aws_sdk_sqs::Client>,
    session: Session,
    mut payload: web::Payload,
) -> ApiResult<UploadBotResponse> {
    use shared::db::schema::{bots, teams};
    let user = login::get_user_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    let team = login::get_team_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;

    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > (*BOT_SIZE).try_into()? {
            return Err(actix_web::error::ErrorBadRequest("Bot too large").into());
        }
        body.extend_from_slice(&chunk);
    }
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(body.to_vec()))
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("{}", e)))?;
    // TODO: if the zip file is one big folder, we should change it to be the root.
    let mut bot_file = archive
        .by_name("bot/bot.json")
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("{}", e)))?;
    if bot_file.is_dir() {
        return Err(actix_web::error::ErrorBadRequest("bot.json is a directory").into());
    }
    let mut bot_json = String::new();
    bot_file.read_to_string(&mut bot_json)?;
    log::debug!("bot.json: {}", bot_json);

    let bot: shared::BotJson = serde_json::from_str(&bot_json)?;

    println!("{:?}", bot);
    // Create a bot entry in the database
    let conn = &mut (*DB_CONNECTION).get()?;
    let id = diesel::insert_into(bots::dsl::bots)
        .values(&NewBot {
            team: team.id,
            name: bot.name,
            description: bot.description,
            score: 0.0,
            uploaded_by: user.email,
            build_status: shared::BuildStatus::Queued,
        })
        .returning(bots::dsl::id)
        .get_result::<i32>(conn)?;
    // upload the file to s3
    if let Err(e) = s3_client
        .put_object()
        .bucket(&*BOT_S3_BUCKET)
        .key(format!("{}", id))
        .body(body.to_vec().into())
        .send()
        .await
    {
        log::warn!("Unable to upload bot: {}", e);

        // delete the bot entry on upload fail
        diesel::delete(bots::dsl::bots.filter(bots::dsl::id.eq(id))).execute(conn)?;
        return Err(e.into());
    }

    let presign_config =
        PresigningConfig::expires_in(std::time::Duration::from_secs(60 * 60 * 24 * 7))?;
    let log_presigned = s3_client
        .put_object()
        .bucket(&*BUILD_LOGS_S3_BUCKET)
        .key(format!("{}/build", id))
        .presigned(presign_config.clone())
        .await?;
    let log_presigned = PresignedRequest {
        url: log_presigned.uri().to_string(),
        headers: log_presigned.headers().into(),
    };
    // push the bot to the 'bot_uploads' queue
    // TODO: Handle errors by deleting the bot from the database
    sqs_client
        .send_message()
        .queue_url(std::env::var("BOT_UPLOADS_QUEUE_URL")?)
        .message_body(serde_json::to_string(&shared::BuildTask {
            bot: id.to_string(),
            log_presigned,
        })?)
        .send()
        .await?;
    Ok(web::Json(UploadBotResponse { id }))
}

#[derive(Deserialize)]
pub struct BuildLogQuery {
    bot: i32,
}
#[get("/build-log")]
pub async fn build_log(
    session: Session,
    web::Query::<BuildLogQuery>(BuildLogQuery { bot }): web::Query<BuildLogQuery>,
    sqs_client: web::Data<aws_sdk_sqs::Client>,
    s3_client: web::Data<aws_sdk_s3::Client>,
) -> Result<HttpResponse, ApiError> {
    let team = login::get_team_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;
    let conn = &mut (*DB_CONNECTION).get()?;
    // If the bot is specified, make sure it belongs to the team
    let bots: Vec<Bot> = schema::bots::dsl::bots
        .filter(schema::bots::dsl::id.eq(bot))
        .filter(schema::bots::dsl::team.eq(team.id))
        .load::<Bot>(conn)?;
    if bots.len() == 0 {
        return Err(
            actix_web::error::ErrorUnauthorized("Only the owner can view a bot's logs.").into(),
        );
    }
    let key = format!("{}/build", bot);
    let presign_config =
        PresigningConfig::expires_in(std::time::Duration::from_secs(60 * 60 * 24 * 7))?;
    let response = s3_client
        .get_object()
        .bucket(&*BUILD_LOGS_S3_BUCKET)
        .key(key)
        .send()
        .await?;
    Ok(HttpResponse::Ok().streaming(response.body))
}
