use actix_session::Session;
use actix_web::get;
use actix_web::HttpResponse;
use actix_web::Result;

#[get("/api/signout")]
pub async fn signout(session: Session) -> Result<HttpResponse> {
    session.remove("me");
    Ok(HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish())
}
