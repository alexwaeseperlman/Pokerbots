use super::*;
use actix_session::Session;
use actix_web::get;
use serde::Serialize;

#[derive(Serialize, TS)]
#[ts(export)]
pub struct SignoutResponse {
    pub message: String,
    pub message_type: String,
}

#[get("/signout")]
pub async fn signout(session: Session) -> ApiResult<SignoutResponse> {
    session.remove("me");
    Ok(actix_web::web::Json(SignoutResponse {
        message: "You have been signed out.".to_string(),
        message_type: "success".to_string(),
    }))
}
