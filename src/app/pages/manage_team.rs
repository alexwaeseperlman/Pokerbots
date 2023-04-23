use crate::{
    app::login,
    config::DB_CONNECTION,
    default_view_data,
    models::User,
    schema::{team_invites, teams, users},
};
use actix_multipart::Multipart;
use actix_session::Session;
use actix_web::{get, post, web, HttpResponse};
use chrono;
use diesel::prelude::*;
use futures::{StreamExt, TryStreamExt};
use rand::{self, Rng};
use serde::Deserialize;
use std::io::Write;

#[derive(Deserialize)]
pub struct CreateTeamQuery {
    pub team_name: String,
}

#[derive(Deserialize)]
pub struct JoinTeamQuery {
    pub invite_code: String,
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
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/manage-team"))
            .finish());
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

    // Make everyone on the team leave the team
    diesel::update(users::dsl::users)
        .filter(users::dsl::team_id.eq(team.clone().unwrap().id))
        .set(users::dsl::team_id.eq::<Option<i32>>(None))
        .execute(conn)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Database update error {}", e))
        })?;

    diesel::delete(teams::dsl::teams.filter(teams::dsl::id.eq(team.unwrap().id)))
        .execute(conn)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Database delete error {}", e))
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
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/manage-team"))
            .finish());
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
    if user.is_none() || team.is_none() || user.unwrap().email != team.clone().unwrap().owner {
        session.insert("message", "You don't have permission to do that.")?;
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/manage-team"))
            .finish());
    }
    // Insert an invite with expiry date 24 hours from now
    let day: i64 = 24 * 3600 * 1000;
    let now: i64 = chrono::offset::Utc::now().timestamp();
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    let out = diesel::insert_into(team_invites::dsl::team_invites)
        .values(crate::models::NewInvite {
            expires: now + day,
            invite_code: rand::thread_rng().gen::<i64>(),
            teamid: team.clone().unwrap().id,
        })
        .returning(team_invites::dsl::invite_code)
        .get_result::<i64>(conn)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Database insert error: {}", e))
        })?;
    Ok(HttpResponse::Ok().body(format!("{:02x}", out)))
}

#[get("/manage-team")]
pub async fn manage_team(
    hb: web::Data<handlebars::Handlebars<'_>>,
    session: Session,
) -> actix_web::Result<HttpResponse> {
    let body = hb
        .render("manage-team", &default_view_data(session)?)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
        })?;
    Ok(HttpResponse::Ok().body(body))
}
