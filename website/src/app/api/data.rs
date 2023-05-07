use actix_session::Session;
use actix_web::{get, HttpResponse};

use crate::app::login;

#[get("/api/my-account")]
pub async fn my_account(session: Session) -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(login::get_user_data(&session)))
}

#[get("/api/my-team")]
pub async fn my_team(session: Session) -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(login::get_team_data(&session)))
}
