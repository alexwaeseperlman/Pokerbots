pub mod pages {
    pub mod manage_team;
    pub mod team;
}

pub mod login;
pub mod upload_bot;

use serde_json::json;

use actix::*;
use actix_service::{IntoService, Service, ServiceFactory};
use actix_web::*;
use actix_web::{get, HttpResponse};
use login::*;
use pages::team;

use crate::UserData;

pub fn all_routes() -> actix_web::Scope {
    web::scope("/").service(home_page).service(team::team)
}

#[get("/")]
pub async fn home_page(
    hb: web::Data<handlebars::Handlebars<'_>>,
    user: Option<web::ReqData<UserData>>,
) -> Result<HttpResponse> {
    let u: Option<UserData> = user.and_then(|x| Some(x.into_inner()));
    let data = json!({
        "microsoft_login": microsoft_login_url(),
        "user": u
    });
    let body = hb.render("homepage", &data).unwrap();
    Ok(HttpResponse::Ok().body(body))
}
