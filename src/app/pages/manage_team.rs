use actix_session::Session;
use actix_web::web::ReqData;
use serde_json::json;

use crate::app::login::*;
use actix::*;
use actix_service::{IntoService, Service, ServiceFactory};
use actix_web::*;
use actix_web::{get, HttpResponse};

use crate::{TeamData, UserData, DB_CONNECTION};

#[get("/api/create-team")]
pub async fn create_team(session: Session, teamName: String) -> Result<HttpResponse> {
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
    let inserted: i32 = diesel::insert_into(teams)
        .values(crate::models::NewTeam { teamname: teamName })
        .returning(id)
        .get_result(conn)
        .unwrap();

    Ok(HttpResponse::Ok().body("Team created"))
}

#[get("/manage-team")]
pub async fn manage_team(
    hb: web::Data<handlebars::Handlebars<'_>>,
    session: Session,
) -> Result<HttpResponse> {
    let user = get_user_data(Some(session.clone()));

    if user.is_none() {
        return Ok(HttpResponse::Ok()
            .append_header(("Location", "/login"))
            .finish());
    }

    let team = get_team_data(Some(session));
    if user.is_none() {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/login"))
            .finish());
    }
    let data = json!({
        "user": user,
        "team": team
    });
    let body = hb.render("manage-team", &data).unwrap();
    Ok(HttpResponse::Ok().body(body))
}
