use actix_session::Session;
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;

use crate::app::login;

use super::ServerMessage;

#[get("/api/my-account")]
pub async fn my_account(session: Session) -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(login::get_user_data(&session)))
}

#[get("/api/my-team")]
pub async fn my_team(session: Session) -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(login::get_team_data(&session)))
}

#[derive(Deserialize)]
pub struct ServerMessageQuery {
    pub clear: Option<bool>,
}

#[get("/api/server-message")]
pub async fn server_message(
    session: Session,
    web::Query::<ServerMessageQuery>(ServerMessageQuery { clear }): web::Query<ServerMessageQuery>,
) -> actix_web::Result<HttpResponse> {
    let msg = HttpResponse::Ok().json(session.get::<ServerMessage>("message")?);
    if clear.unwrap_or(false) {
        session.remove("message");
    }
    Ok(msg)
}
