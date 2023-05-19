use std::io::Read;

use crate::{
    app::login,
    app::login::microsoft_login_url,
    config::{BOT_S3_BUCKET, BOT_SIZE, DB_CONNECTION, PFP_S3_BUCKET},
    models::{NewBot, TeamInvite, User},
    schema::{bots, team_invites, teams, users},
};
use actix_session::Session;
use actix_web::{get, post, put, web, HttpResponse};
use aws_sdk_s3 as s3;
use aws_sdk_s3::presigning::PresigningConfig;
use chrono;
use diesel::prelude::*;
use futures_util::StreamExt;
use rand::{self, Rng};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct CreateTeamQuery {
    pub team_name: String,
}

pub fn validate_team_name(name: &String) -> bool {
    if name.len() < 3 || name.len() > 20 {
        return false;
    }
    for c in name.chars() {
        if !c.is_alphanumeric() && c != ' ' && c != '-' {
            return false;
        }
    }
    return true;
}
#[get("/api/create-team")]
pub async fn create_team(
    session: Session,
    web::Query::<CreateTeamQuery>(CreateTeamQuery { team_name }): web::Query<CreateTeamQuery>,
) -> actix_web::Result<HttpResponse> {
    let user = login::get_user_data(&session);
    if user.is_none() {
        return Ok(HttpResponse::NotFound()
            .append_header(("Location", "/login"))
            .finish());
    }
    // You can't create a team if you're already in one
    if login::get_team_data(&session).is_some() {
        return Ok(HttpResponse::Conflict().body("{\"error\": \"You are already in a team.\"}"));
    }
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    let new_id = diesel::insert_into(teams::dsl::teams)
        .values(crate::models::NewTeam {
            team_name,
            owner: user.clone().unwrap().email,
        })
        .returning(teams::dsl::id)
        .get_result::<i32>(conn)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Database insert error: {}", e))
        })?;

    diesel::update(users::dsl::users)
        .filter(users::dsl::email.eq(user.unwrap().email))
        .set(users::dsl::team_id.eq(new_id))
        .get_result::<User>(conn)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Database update error: {}", e))
        })?;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/manage-team"))
        .finish())
}

#[derive(Deserialize)]
pub struct JoinTeamQuery {
    pub invite_code: String,
}

#[get("/api/delete-team")]
pub async fn delete_team(session: Session) -> actix_web::Result<HttpResponse> {
    let user = login::get_user_data(&session);
    let team = login::get_team_data(&session);
    // You can't delete a team if you're not in one
    if user.is_none() || team.is_none() || team.clone().unwrap().owner != user.unwrap().email {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/manage-team"))
            .finish());
    }
    let conn = &mut (*DB_CONNECTION).get().unwrap();

    diesel::delete(teams::dsl::teams.filter(teams::dsl::id.eq(team.clone().unwrap().id)))
        .execute(conn)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Database delete error {}", e))
        })?;
    // Make everyone on the team leave the team
    diesel::update(users::dsl::users)
        .filter(users::dsl::team_id.eq(team.unwrap().id))
        .set(users::dsl::team_id.eq::<Option<i32>>(None))
        .execute(conn)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Database update error {}", e))
        })?;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/manage-team"))
        .finish())
}

#[get("/api/leave-team")]
pub async fn leave_team(session: Session) -> actix_web::Result<HttpResponse> {
    let user = login::get_user_data(&session);
    let team = login::get_team_data(&session);
    // You can't delete a team if you're not in one or you're the owner
    if user.is_none() || team.is_none() || user.clone().unwrap().email == team.unwrap().owner {
        return Ok(HttpResponse::NotAcceptable().body("{\"error\": \"You can't leave the team\""));
    }
    let conn = &mut (*DB_CONNECTION).get().unwrap();

    // Set the current user's team to null
    diesel::update(users::dsl::users)
        .filter(users::dsl::email.eq(user.unwrap().email))
        .set(users::dsl::team_id.eq::<Option<i32>>(None))
        .execute(conn)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Database update error {}", e))
        })?;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/manage-team"))
        .finish())
}
#[get("/api/make-invite")]
pub async fn make_invite(session: Session) -> actix_web::Result<HttpResponse> {
    let user = login::get_user_data(&session);
    let team = login::get_team_data(&session);
    // You can't join a team if you are already on one or if you aren't logged in
    // Also only the owner can create a team

    // if the number of invites plus the number of users is at the limit, then don't create an invite
    if let Some(team) = &team {
        if team.invites.len() + team.members.len() >= crate::config::TEAM_SIZE as usize {
            return Ok(HttpResponse::NotAcceptable().body("{\"error\": \"Team is full\"}"));
        }
    }

    if user.is_none() || team.is_none() || user.unwrap().email != team.clone().unwrap().owner {
        return Ok(HttpResponse::NotAcceptable().body("{\"error\": \"Not able to make invite\"}"));
    }
    // Insert an invite with expiry date 24 hours from now
    let day: i64 = 24 * 3600 * 1000;
    let now: i64 = chrono::offset::Utc::now().timestamp();
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    let out = diesel::insert_into(team_invites::dsl::team_invites)
        .values(crate::models::NewInvite {
            expires: now + day,
            invite_code: format!("{:02x}", rand::thread_rng().gen::<u128>()),
            teamid: team.clone().unwrap().id,
        })
        .returning(team_invites::dsl::invite_code)
        .get_result::<String>(conn)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Database insert error: {}", e))
        })?;
    Ok(HttpResponse::Ok().body(out))
}

#[derive(Deserialize)]
pub struct CancelTeamQuery {
    pub invite_code: String,
}

#[get("/api/cancel-invite")]
pub async fn cancel_invite(
    session: Session,
    web::Query(CancelTeamQuery { invite_code }): web::Query<CancelTeamQuery>,
) -> actix_web::Result<HttpResponse> {
    let user = login::get_user_data(&session);
    let team = login::get_team_data(&session);

    if user.is_none() || team.is_none() || user.unwrap().email != team.clone().unwrap().owner {
        return Ok(HttpResponse::NotAcceptable()
            .body("{\"error\": \"Only team owner can cancel invites.\"}"));
    }
    // Insert an invite with expiry date 24 hours from now
    let day: i64 = 24 * 3600 * 1000;
    let now: i64 = chrono::offset::Utc::now().timestamp();
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    let out = diesel::delete(team_invites::dsl::team_invites)
        .filter(team_invites::dsl::invite_code.eq(&invite_code))
        .returning(team_invites::dsl::invite_code)
        .get_result::<String>(conn)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Database insert error: {}", e))
        })?;
    Ok(HttpResponse::Ok().body(out))
}

#[get("/api/join-team")]
pub async fn join_team(
    session: Session,
    web::Query::<JoinTeamQuery>(JoinTeamQuery { invite_code }): web::Query<JoinTeamQuery>,
    req: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse> {
    let user = login::get_user_data(&session);
    let team = login::get_team_data(&session);
    // You can't join a team if you are already on one or if you aren't logged in
    if user.is_none() {
        return Ok(HttpResponse::Found()
            .append_header(("Location", microsoft_login_url(&req.uri().to_string())))
            .finish());
    }
    if team.is_some() {
        return Ok(
            HttpResponse::NotAcceptable().json(json!({"error": "You are already on a team."}))
        );
    } else {
        let conn = &mut (*DB_CONNECTION).get().unwrap();
        //.map_err(|e| actix_web::error::ErrorInternalServerError("No database connection"))?;
        // Check if there is an existing team invite with this code
        let codes: Vec<TeamInvite> = team_invites::dsl::team_invites
            .find(invite_code.clone())
            .load::<TeamInvite>(conn)
            .map_err(|e| actix_web::error::ErrorNotFound("Unable to load invite"))?;
        if let Some(code) = codes.first() {
            let now: i64 = chrono::offset::Utc::now().timestamp();
            if code.expires < now {
                return Ok(HttpResponse::NotAcceptable().json(json!({"error": "Invalid code."})));
            } else {
                // Set the users team and set the code to used
                diesel::delete(team_invites::dsl::team_invites)
                    .filter(team_invites::dsl::invite_code.eq(invite_code))
                    .execute(conn)
                    .map_err(|e| {
                        actix_web::error::ErrorInternalServerError(format!(
                            "Failed to delete team invite: {}",
                            e
                        ))
                    })?;
                diesel::update(users::dsl::users)
                    .filter(users::dsl::email.eq(user.unwrap().email))
                    .set(users::dsl::team_id.eq(code.teamid))
                    .execute(conn)
                    .map_err(|e| {
                        actix_web::error::ErrorInternalServerError(format!(
                            "Failed to update user team: {}",
                            e
                        ))
                    })?;
            }
        } else {
            return Ok(HttpResponse::NotAcceptable().json(json!({"error": "Invalid code."})));
        }
    }
    Ok(HttpResponse::Found()
        .append_header(("Location", "/manage-team"))
        .finish())
}

#[put("/api/upload-pfp")]
pub async fn upload_pfp(
    s3_client: actix_web::web::Data<s3::Client>,
    session: Session,
    mut payload: web::Payload,
) -> actix_web::Result<HttpResponse> {
    let user = login::get_user_data(&session);
    let team = login::get_team_data(&session);
    if user.is_none() {
        return Ok(HttpResponse::Unauthorized().body("{\"error\": \"Not logged in\"}"));
    }
    if team.is_none() || team.clone().unwrap().owner != user.clone().unwrap().email {
        return Ok(HttpResponse::Unauthorized().body("{\"error\": \"Not team owner\"}"));
    }

    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk.unwrap();
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > 500000 {
            return Err(actix_web::error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }
    s3_client
        .put_object()
        .bucket(&*PFP_S3_BUCKET)
        .key(format!("{}.png", team.unwrap().id))
        .body(body.to_vec().into())
        .acl(s3::types::ObjectCannedAcl::PublicRead)
        .send()
        .await
        .map_err(|e| {
            log::warn!("Unable to upload pfp: {}", e);
            actix_web::error::ErrorNotFound(format!("Unable to make upload link {}", e))
        })?;

    // TODO: Maybe run the image through a sanitizer/thumbnailer
    // TODO: Maybe check for inappropriate content using Rekognition

    Ok(HttpResponse::Ok().body(""))
}

#[post("/api/upload-bot")]
pub async fn upload_bot(
    s3_client: actix_web::web::Data<s3::Client>,
    session: Session,
    mut payload: web::Payload,
) -> actix_web::Result<HttpResponse> {
    let user = login::get_user_data(&session);
    let team = login::get_team_data(&session);
    if user.is_none() {
        return Ok(HttpResponse::Unauthorized().body("{\"error\": \"Not logged in\"}"));
    }
    if team.is_none() {
        return Ok(HttpResponse::Unauthorized().body("{\"error\": \"Not on a team\"}"));
    }

    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk.unwrap();
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > (*BOT_SIZE).try_into().unwrap() {
            return Err(actix_web::error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(body.to_vec())).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Unable to parse zip file: {}", e))
    })?;
    // TODO: if the zip file is one big folder, we should change it to be the root.
    let mut bot_file = archive.by_name("bot.json").map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Unable to find bot.json: {}", e))
    })?;
    if bot_file.is_dir() {
        return Err(actix_web::error::ErrorBadRequest("bot.json is a directory"));
    }
    let mut bot_json = String::new();
    bot_file.read_to_string(&mut bot_json).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Unable to read bot.json: {}", e))
    })?;

    let bot: shared::Bot = serde_json::from_str(&bot_json).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Unable to parse bot.json: {}", e))
    })?;

    println!("{:?}", bot);
    // Create a bot entry in the database
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    let id = diesel::insert_into(bots::dsl::bots)
        .values(&NewBot {
            team: team.unwrap().id,
            name: bot.name,
            description: bot.description,
            score: 0.0,
            uploaded_by: user.unwrap().email,
        })
        .returning(bots::dsl::id)
        .get_result::<i32>(conn)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to create bot: {}", e))
        })?;
    // upload the file to s3
    s3_client
        .put_object()
        .bucket(&*BOT_S3_BUCKET)
        .key(format!("{}.zip", id))
        .body(body.to_vec().into())
        .send()
        .await
        .map_err(|e| {
            log::warn!("Unable to upload bot: {}", e);

            // delete the bot entry on upload fail
            diesel::delete(bots::dsl::bots.filter(bots::dsl::id.eq(id)))
                .execute(conn)
                .map_err(|e| {
                    actix_web::error::ErrorInternalServerError(format!(
                        "Failed to delete bot: {}",
                        e
                    ))
                })
                .unwrap();
            actix_web::error::ErrorNotFound(format!("Unable to make upload link {}", e))
        })?;

    Ok(HttpResponse::Ok().json(json!({ "id": id })))
}
