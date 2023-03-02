use actix_session::Session;
use actix_web::web::ReqData;
use serde_json::json;

use crate::app::login::*;
use actix::*;
use actix_service::{IntoService, Service, ServiceFactory};
use actix_web::*;
use actix_web::{get, HttpResponse};

use crate::{TeamData, UserData};

#[get("/team")]
pub async fn team(
    hb: web::Data<handlebars::Handlebars<'_>>,
    session: Session,
) -> Result<HttpResponse> {
    let user = get_user_data(Some(session.clone()));
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
    let body = hb.render("team", &data).unwrap();
    Ok(HttpResponse::Ok().body(body))
}
