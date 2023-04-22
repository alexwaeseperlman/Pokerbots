use actix_session::Session;
use serde_json::json;
use actix_web::{web, get, HttpResponse};

use crate::app::login;

#[get("/team")]
pub async fn team(
    hb: web::Data<handlebars::Handlebars<'_>>,
    session: Session,
) -> actix_web::Result<HttpResponse> {
    let user = login::get_user_data(&session);
    let team = login::get_team_data(&session);
    if user.is_none() {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/login"))
            .finish());
    }
    let data = json!({
        "user": user,
        "team": team
    });
    let body = hb.render("team", &data).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;
    Ok(HttpResponse::Ok().body(body))
}
