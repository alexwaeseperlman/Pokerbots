use log::info;
use actix_session::Session;
use actix_web::{get, web, HttpResponse};
use serde_json::json;

use crate::app::login;

#[get("/manage-team")]
pub async fn manage_team(
    hb: web::Data<handlebars::Handlebars<'_>>,
    session: Session,
) -> actix_web::Result<HttpResponse> {
    let user = login::get_user_data(&session);
    let team = login::get_team_data(&session);

    let data = json!({
        "user": user,
        "team": team,
        "microsoft_login": login::microsoft_login_url(),
        "isOwner": user.is_some() && team.is_some() && user.unwrap().email == team.unwrap().owner

    });
    info!("{:?}", data);
    let body = hb.render("manage-team", &data).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;
    Ok(HttpResponse::Ok().body(body))
}
