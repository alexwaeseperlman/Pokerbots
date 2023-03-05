pub mod pages {
    pub mod manage_team;
    pub mod team;
}

pub mod api {
    pub mod signout;
}

pub mod login;

use actix_session::Session;
use serde_json::json;

use actix::*;
use actix_service::{IntoService, Service, ServiceFactory};
use actix_web::*;
use actix_web::{get, HttpResponse};
use login::*;
use pages::team;

use crate::{default_view_data, UserData};

pub fn all_routes() -> actix_web::Scope {
    web::scope("/").service(home_page).service(team::team)
}

#[get("/")]
pub async fn home_page(
    hb: web::Data<handlebars::Handlebars<'_>>,
    session: Session,
) -> Result<HttpResponse> {
    let body = hb.render("homepage", &default_view_data(session)).unwrap();
    Ok(HttpResponse::Ok().body(body))
}
