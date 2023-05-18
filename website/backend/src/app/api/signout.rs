use actix_session::Session;
use actix_web::get;
use actix_web::HttpResponse;
use actix_web::Result;
use serde_json::json;

#[get("/api/signout")]
pub async fn signout(session: Session) -> Result<HttpResponse> {
    session.remove("me");
    Ok(HttpResponse::Ok().json(json!({
        "message": "You have been signed out.",
        "message_type": "success"
    })))
}
