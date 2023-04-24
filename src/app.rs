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
