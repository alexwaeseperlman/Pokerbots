use crate::{
    app::login::{self, microsoft_login_url, url_encode},
    config::DB_CONNECTION,
    default_view_data,
    models::{TeamInvite, User},
    schema::{team_invites, teams, users},
};
use actix_session::Session;
use actix_web::{get, web, HttpResponse};

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
