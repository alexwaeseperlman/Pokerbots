use std::io::Read;

use crate::{
    app::login,
    app::{api::ApiResult, login::microsoft_login_url},
    config::{BOT_S3_BUCKET, BOT_SIZE, DB_CONNECTION, PFP_S3_BUCKET},
};
use actix_session::Session;
use actix_web::{get, post, put, web, HttpResponse};
use aws_sdk_s3 as s3;
use aws_sdk_sqs as sqs;
use chrono;
use diesel::prelude::*;
use futures_util::StreamExt;
use rand::{self, Rng};
use serde::Deserialize;
use serde_json::json;
use shared::db::{
    models::{NewBot, NewInvite, NewTeam, TeamInvite, User},
    schema::{bots, team_invites, teams, users},
};

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
    if name.contains("  ") {
        return false;
    }
    return true;
}
#[get("/create-team")]
pub async fn create_team(
    session: Session,
    web::Query::<CreateTeamQuery>(CreateTeamQuery { team_name }): web::Query<CreateTeamQuery>,
) -> ApiResult {
    let user = login::get_user_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    // You can't create a team if you're already in one
    if login::get_team_data(&session).is_some() {
        return Err(actix_web::error::ErrorConflict("You are already on a team.").into());
    }
    let conn = &mut (*DB_CONNECTION).get()?;
    let new_id = diesel::insert_into(teams::dsl::teams)
        .values(NewTeam {
            team_name,
            owner: user.clone().email,
        })
        .returning(teams::dsl::id)
        .get_result::<i32>(conn)?;

    diesel::update(users::dsl::users)
        .filter(users::dsl::email.eq(user.email))
        .set(users::dsl::team_id.eq(new_id))
        .get_result::<User>(conn)?;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/manage-team"))
        .finish())
}

#[derive(Deserialize)]
pub struct JoinTeamQuery {
    pub invite_code: String,
}

#[get("/delete-team")]
pub async fn delete_team(session: Session) -> ApiResult {
    let user = login::get_user_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    let team = login::get_team_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;
    // You can't delete a team if you're not in one
    if team.clone().owner != user.email {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/manage-team"))
            .finish());
    }
    let conn = &mut (*DB_CONNECTION).get()?;

    diesel::delete(teams::dsl::teams.filter(teams::dsl::id.eq(team.clone().id))).execute(conn)?;
    // Make everyone on the team leave the team
    diesel::update(users::dsl::users)
        .filter(users::dsl::team_id.eq(team.id))
        .set(users::dsl::team_id.eq::<Option<i32>>(None))
        .execute(conn)?;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/manage-team"))
        .finish())
}

#[get("/leave-team")]
pub async fn leave_team(session: Session) -> ApiResult {
    let user = login::get_user_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    let team = login::get_team_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;
    // You can't delete a team if you're not in one or you're the owner
    if user.clone().email == team.owner {
        return Err(actix_web::error::ErrorNotAcceptable(
            "You can't leave the team if you are the owner.",
        )
        .into());
    }
    let conn = &mut (*DB_CONNECTION).get()?;

    // Set the current user's team to null
    diesel::update(users::dsl::users)
        .filter(users::dsl::email.eq(user.email))
        .set(users::dsl::team_id.eq::<Option<i32>>(None))
        .execute(conn)?;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/manage-team"))
        .finish())
}
#[get("/make-invite")]
pub async fn make_invite(session: Session) -> ApiResult {
    let user = login::get_user_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    let team = login::get_team_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;
    // You can't join a team if you are already on one or if you aren't logged in
    // Also only the owner can create a team

    // if the number of invites plus the number of users is at the limit, then don't create an invite
    if team.invites.len() + team.members.len() >= crate::config::TEAM_SIZE as usize {
        return Err(actix_web::error::ErrorNotAcceptable("Team is full.").into());
    }

    if user.email != team.clone().owner {
        return Err(actix_web::error::ErrorNotAcceptable("Not able to make invite.").into());
    }
    // Insert an invite with expiry date 100 years from now
    // We are not using expiry dates for invites
    let day: i64 = 24 * 3600 * 1000 * 365 * 100;
    let now: i64 = chrono::offset::Utc::now().timestamp();
    let conn = &mut (*DB_CONNECTION).get()?;
    let out = diesel::insert_into(team_invites::dsl::team_invites)
        .values(NewInvite {
            expires: now + day,
            invite_code: format!("{:02x}", rand::thread_rng().gen::<u128>()),
            teamid: team.clone().id,
        })
        .returning(team_invites::dsl::invite_code)
        .get_result::<String>(conn)?;
    Ok(HttpResponse::Ok().body(out))
}

#[derive(Deserialize)]
pub struct CancelTeamQuery {
    pub invite_code: String,
}

#[get("/cancel-invite")]
pub async fn cancel_invite(
    session: Session,
    web::Query(CancelTeamQuery { invite_code }): web::Query<CancelTeamQuery>,
) -> ApiResult {
    let user = login::get_user_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    let team = login::get_team_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;

    // Insert an invite with expiry date 24 hours from now
    let conn = &mut (*DB_CONNECTION).get()?;
    let out = diesel::delete(team_invites::dsl::team_invites)
        .filter(team_invites::dsl::invite_code.eq(&invite_code))
        .filter(team_invites::dsl::teamid.eq(team.clone().id))
        .returning(team_invites::dsl::invite_code)
        .get_result::<String>(conn)?;
    Ok(HttpResponse::Ok().body(out))
}

#[get("/join-team")]
pub async fn join_team(
    session: Session,
    web::Query::<JoinTeamQuery>(JoinTeamQuery { invite_code }): web::Query<JoinTeamQuery>,
    req: actix_web::HttpRequest,
) -> ApiResult {
    let user = match login::get_user_data(&session) {
        Some(user) => user,
        None => {
            return Ok(HttpResponse::Found()
                //TODO: Redirect to a general login page
                .append_header(("Location", microsoft_login_url(&req.uri().to_string())))
                .finish());
        }
    };
    let team = login::get_team_data(&session);
    // You can't join a team if you are already on one or if you aren't logged in

    if team.is_some() {
        return Err(actix_web::error::ErrorNotAcceptable("You are already on a team.").into());
    } else {
        let conn = &mut (*DB_CONNECTION).get()?;
        // Check if there is an existing team invite with this code
        let codes: Vec<TeamInvite> = team_invites::dsl::team_invites
            .find(invite_code.clone())
            .load::<TeamInvite>(conn)?;
        if let Some(code) = codes.first() {
            let now: i64 = chrono::offset::Utc::now().timestamp();
            if code.expires < now {
                return Err(actix_web::error::ErrorNotAcceptable("Invalid code.").into());
            } else {
                // Set the users team and set the code to used
                diesel::delete(team_invites::dsl::team_invites)
                    .filter(team_invites::dsl::invite_code.eq(invite_code))
                    .execute(conn)?;
                diesel::update(users::dsl::users)
                    .filter(users::dsl::email.eq(user.email))
                    .set(users::dsl::team_id.eq(code.teamid))
                    .execute(conn)?;
            }
        } else {
            return Err(actix_web::error::ErrorNotAcceptable("Invalid code.").into());
        }
    }
    Ok(HttpResponse::Found()
        .append_header(("Location", "/manage-team"))
        .finish())
}

#[put("/upload-pfp")]
pub async fn upload_pfp(
    s3_client: actix_web::web::Data<s3::Client>,
    session: Session,
    mut payload: web::Payload,
) -> ApiResult {
    let user = login::get_user_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    let team = login::get_team_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;

    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > 500000 {
            return Err(actix_web::error::ErrorBadRequest("PFP too large").into());
        }
        body.extend_from_slice(&chunk);
    }
    s3_client
        .put_object()
        .bucket(&*PFP_S3_BUCKET)
        .key(format!("{}.png", team.id))
        .body(body.to_vec().into())
        .acl(s3::types::ObjectCannedAcl::PublicRead)
        .send()
        .await?;

    // TODO: Maybe run the image through a sanitizer/thumbnailer
    // TODO: Maybe check for inappropriate content using Rekognition

    Ok(HttpResponse::Ok().body("{}"))
}

#[post("/upload-bot")]
pub async fn upload_bot(
    s3_client: actix_web::web::Data<s3::Client>,
    sqs_client: actix_web::web::Data<sqs::Client>,
    session: Session,
    mut payload: web::Payload,
) -> ApiResult {
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

    let bot: shared::Bot = serde_json::from_str(&bot_json)?;

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

    // push the bot to the 'bot_uploads' queue
    if let Some(s) = sqs_client
        .get_queue_url()
        .queue_name("bot_uploads")
        .send()
        .await?
        .queue_url()
    {
        sqs_client
            .send_message()
            .queue_url(s)
            .message_body(serde_json::to_string(&shared::BuildTask {
                bot: id.to_string(),
            })?)
            .send()
            .await?;
    }
    Ok(HttpResponse::Ok().json(json!({ "id": id })))
}

#[derive(Deserialize)]
pub struct KickMemberQuery {
    pub email: String,
}

#[get("/kick-member")]
pub async fn kick_member(
    session: Session,
    web::Query::<KickMemberQuery>(KickMemberQuery { email }): web::Query<KickMemberQuery>,
) -> ApiResult {
    let user = login::get_user_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    let team = login::get_team_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;
    if team.clone().owner != user.clone().email {
        return Err(
            actix_web::error::ErrorUnauthorized("Only the team owner can kick members.").into(),
        );
    }

    let conn = &mut (*DB_CONNECTION).get()?;
    diesel::update(users::dsl::users)
        .filter(users::dsl::email.eq(email))
        .filter(users::dsl::team_id.eq(team.id))
        .set(users::dsl::team_id.eq::<Option<i32>>(None))
        .execute(conn)?;
    // TODO: Maybe some kind of message should show for the user next time they log in?
    Ok(HttpResponse::Ok().body("{}"))
}

#[derive(Deserialize)]
pub struct RenameTeamQuery {
    pub to: String,
}

#[get("/rename-team")]
pub async fn rename_team(
    session: Session,
    web::Query::<RenameTeamQuery>(RenameTeamQuery { to }): web::Query<RenameTeamQuery>,
) -> ApiResult {
    let user = login::get_user_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    let team = login::get_team_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;

    if !validate_team_name(&to) {
        return Err(actix_web::error::ErrorNotAcceptable(
            "Invalid team name. It must be at most 20 characters and cannot contain consecutive spaces.",
        )
        .into()).into();
    }

    let conn = &mut (*DB_CONNECTION).get()?;
    diesel::update(teams::dsl::teams)
        .filter(teams::dsl::id.eq(team.clone().id))
        .filter(teams::dsl::owner.eq(user.clone().email))
        .set(teams::dsl::team_name.eq(to))
        .execute(conn)?;
    Ok(HttpResponse::Ok().body("{}")).into()
}

#[derive(Deserialize)]
pub struct DeleteBot {
    pub id: i32,
}

#[get("/delete-bot")]
pub async fn delete_bot(
    session: Session,
    web::Query::<DeleteBot>(DeleteBot { id }): web::Query<DeleteBot>,
) -> ApiResult {
    let user = login::get_user_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    let team = login::get_team_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;

    let conn = &mut (*DB_CONNECTION).get()?;
    diesel::delete(bots::dsl::bots)
        .filter(bots::dsl::id.eq(id))
        .filter(bots::dsl::team.eq(team.id))
        .execute(conn)?;

    Ok(HttpResponse::Ok().body("{}"))
}

#[derive(Deserialize)]
pub struct ActiveBot {
    pub id: Option<i32>,
}
#[get("/set-active-bot")]
pub async fn set_active_bot(
    session: Session,
    web::Query::<ActiveBot>(ActiveBot { id }): web::Query<ActiveBot>,
) -> ApiResult {
    let user = login::get_user_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    let team = login::get_team_data(&session)
        .ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;

    let conn = &mut (*DB_CONNECTION).get()?;
    diesel::update(teams::dsl::teams)
        .filter(teams::dsl::id.eq(team.id))
        .filter(teams::dsl::owner.eq(user.clone().email))
        .set(teams::dsl::active_bot.eq(id))
        .execute(conn)?;

    Ok(HttpResponse::Ok().body("{}"))
}
