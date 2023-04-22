pub mod pages {
    pub mod home;
    pub mod manage_team;
    pub mod team;
}
pub mod api;

pub mod login;

use actix_session::Session;
use serde_json::json;

use actix::*;
use actix_service::{IntoService, Service, ServiceFactory};
use actix_web::{get, web};
use login::*;
use pages::team;

use crate::{default_view_data, UserData};

#[get("/")]
pub async fn home_page(
    hb: web::Data<handlebars::Handlebars<'_>>,
    session: Session,
) -> actix_web::Result<actix_web::HttpResponse> {
    let body = hb.render("homepage", &default_view_data(session)?).unwrap();
    Ok(actix_web::HttpResponse::Ok().body(body))
}
