use actix_session::Session;
use actix_web::web::ReqData;
use serde::Deserialize;
use serde_json::json;
use std::io::Write;

use crate::app::login::*;
use crate::models::User;
use actix::*;
use actix_service::{IntoService, Service, ServiceFactory};
use actix_web::*;
use actix_web::{get, post, HttpResponse};
use futures::{StreamExt, TryStreamExt};

use actix_multipart::Multipart;

use crate::{TeamData, UserData, DB_CONNECTION};
#[derive(Deserialize)]
pub struct CreateTeamQuery {
    pub team_name: String,
}

#[get("/api/create-team")]
pub async fn create_team(
    session: Session,
    web::Query::<CreateTeamQuery>(CreateTeamQuery { team_name }): web::Query<CreateTeamQuery>,
) -> Result<HttpResponse> {
    use diesel::*;
    let user = get_user_data(Some(session.clone()));
    if user.is_none() {
        return Ok(HttpResponse::NotFound()
            .append_header(("Location", "/login"))
            .finish());
    }
    // You can't create a team if you're already in one
    if get_team_data(Some(session)).is_some() {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/manage-team"))
            .finish());
    }
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    use crate::schema::teams::dsl::*;
    use crate::schema::users::dsl::*;
    let team_create_result = diesel::insert_into(teams)
        .values(crate::models::NewTeam {
            teamname: team_name,
            owner: user.clone().unwrap().email,
        })
        .returning(id)
        .get_result::<i32>(conn);

    if let Ok(new_id) = team_create_result {
        println!(
            "{:?}",
            diesel::update(users)
                .filter(email.eq(user.unwrap().email))
                .set(teamid.eq(new_id))
                .get_result::<User>(conn)
        );
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/manage-team"))
            .finish());
    } else {
        println!("{:?}", team_create_result);
    }

    Ok(HttpResponse::Ok().body(format!("Could not create team")))
}

#[get("/api/delete-team")]
pub async fn delete_team(session: Session) -> Result<HttpResponse> {
    use diesel::*;
    let user = get_user_data(Some(session.clone()));
    let team = get_team_data(Some(session.clone()));
    // You can't delete a team if you're not in one
    if user.is_none() || team.is_none() || team.clone().unwrap().owner != user.unwrap().email {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/manage-team"))
            .finish());
    }
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    use crate::schema::teams;
    use crate::schema::users;

    // Make everyone on the team leave the team
    diesel::update(users::dsl::users)
        .filter(users::dsl::teamid.eq(team.clone().unwrap().id))
        .set(users::dsl::teamid.eq::<Option<i32>>(None))
        .execute(conn)
        .unwrap();

    dsl::delete(teams::dsl::teams.filter(teams::dsl::id.eq(team.unwrap().id)))
        .execute(conn)
        .unwrap();

    Ok(HttpResponse::Found()
        .append_header(("Location", "/manage-team"))
        .finish())
}

#[get("/api/leave-team")]
pub async fn leave_team(session: Session) -> Result<HttpResponse> {
    use diesel::*;
    let user = get_user_data(Some(session.clone()));
    let team = get_team_data(Some(session.clone()));
    // You can't delete a team if you're not in one or you're the owner
    if user.is_none()
        || team.is_none()
        || user.clone().unwrap().email == team.clone().unwrap().owner
    {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/manage-team"))
            .finish());
    }
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    use crate::schema::users;

    // Set the current user's team to null
    diesel::update(users::dsl::users)
        .filter(users::dsl::email.eq(user.clone().unwrap().email))
        .set(users::dsl::teamid.eq::<Option<i32>>(None))
        .execute(conn)
        .unwrap();

    Ok(HttpResponse::Found()
        .append_header(("Location", "/manage-team"))
        .finish())
}

#[post("/api/upload-bot")]
pub async fn upload_bot(session: Session, mut payload: Multipart) -> Result<HttpResponse> {
    while let Ok(Some(mut field)) = payload.try_next().await {
        let team = get_team_data(Some(session.clone()));
        let file_string = format!("/tmp/{}.py", team.unwrap().teamname);
        let mut f =
            match web::block(move || std::fs::File::create(std::path::Path::new(&file_string)))
                .await
            {
                Ok(Ok(file)) => file,
                _ => return Ok(HttpResponse::BadRequest().body("Failed to create file")),
            };

        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            f = match web::block(move || f.write_all(&data).map(|_| f)).await {
                Ok(Ok(file)) => file,
                _ => return Ok(HttpResponse::BadRequest().body("Failed to upload bot to file")),
            }
        }
    }
    Ok(HttpResponse::Ok().body(format!("Successfully uploaded bot")))
}

#[get("/manage-team")]
pub async fn manage_team(
    hb: web::Data<handlebars::Handlebars<'_>>,
    session: Session,
) -> Result<HttpResponse> {
    let user = get_user_data(Some(session.clone()));
    let team = get_team_data(Some(session));

    let data = json!({
        "user": user,
        "team": team,
        "microsoft_login": microsoft_login_url(),
        "isOwner": user.is_some() && team.is_some() && user.unwrap().email == team.unwrap().owner

    });
    println!("{:?}", data);
    let body = hb.render("manage-team", &data).unwrap();
    Ok(HttpResponse::Ok().body(body))
}
