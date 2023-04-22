use serde_json::json;
use actix_web::{get, web, HttpResponse};

use crate::app::{
    login,
    login::UserData,
};

#[get("/")]
pub async fn home(
    hb: web::Data<handlebars::Handlebars<'_>>,
    user: Option<web::ReqData<UserData>>,
) -> actix_web::Result<HttpResponse> {
    let u: Option<UserData> = user.map(|x| x.into_inner());
    let data = json!({
        "microsoft_login": login::microsoft_login_url(),
        "user": u
    });

    let body = hb.render("homepage", &data).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;
    Ok(HttpResponse::Ok().body(body))
}
