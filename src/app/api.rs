use std::{fs, io, io::Write, path::PathBuf};
use actix_multipart::Multipart;
use actix_session::Session;
use actix_web::{get, post, web, HttpResponse};
use diesel::prelude::*;
use futures::{StreamExt, TryStreamExt};
use log::debug;
use serde::Deserialize;

use crate::{
    app::{bots::Bot, login},
    config::DB_CONNECTION,
    models::User,
    schema::{teams, users},
};

pub struct BotsList {
    pub bots: std::sync::Mutex<Vec<Bot>>,
}

#[derive(Deserialize)]
pub struct CreateTeamQuery {
    pub team_name: String,
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

#[post("/api/upload-bot")]
pub async fn upload_bot(
    session: Session,
    data: web::Data<BotsList>,
    mut payload: Multipart,
) -> actix_web::Result<HttpResponse> {
    let team_name = {
        let team = login::get_team_data(&session);
        team.unwrap().team_name
    };
    let team_path = PathBuf::from(format!("/tmp/pokerzero/{}", team_name));

    let mut zip_file = {
        if !team_path.exists() {
            fs::create_dir_all(&team_path)?;
        }
        fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(team_path.join("bot.zip"))?
    };

    while let Ok(Some(mut field)) = payload.try_next().await {
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            zip_file = zip_file.write_all(&data).map(|_| zip_file)?;
        }
    }

    // TODO: IS IT BETTER TO SCOPE VARIABLES AS SMALL AS POSSIBLE?
    // TODO: SHOULD WE web::block these?
    let vals: serde_yaml::Value = {
        let mut archive = zip::ZipArchive::new(zip_file).map_err(io::Error::from)?;
        archive.extract(&team_path).map_err(io::Error::from)?;
        let yaml_file = match std::fs::File::open(team_path.join("bot").join("cmd.yaml")) {
            Ok(f) => f,
            Err(e) => match e.kind() {
                io::ErrorKind::NotFound => fs::File::open(team_path.join("bot").join("cmd.yml"))?,
                _ => return Err(e).map_err(actix_web::Error::from),
            },
        };
        serde_yaml::from_reader(yaml_file).map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Yaml parsing error: {}", e))
        })?
    };

    let bot = Bot::new(
        team_name,
        // team.unwrap().team_name,
        team_path,
        vals.get("build")
            .map_or(None, |v| v.as_str())
            .map(String::from),
        vals.get("run")
            .map_or(None, |v| v.as_str())
            .map(String::from),
    );
    let bot_ = bot.clone();
    // TODO: MAKE IT REPLACE OLD BOT RATHER THAN PUSH
    data.bots.lock().unwrap().push(bot_);
    bot.play(&data.bots.lock().unwrap()).await?;

    Ok(HttpResponse::Ok().body("Successfully uploaded bot"))
}
