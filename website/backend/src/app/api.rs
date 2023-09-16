use std::{fmt, num::TryFromIntError};

use actix_web::{error::PayloadError, http::StatusCode, HttpResponse, ResponseError};
use aws_sdk_s3::presigning::PresigningConfigError;
use reqwest::header::ToStrError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use ts_rs::TS;

use crate::config::{self, game_logs_s3_bucket, pfp_s3_bucket};
use actix_session::Session;
use actix_web::{delete, get, web};
use aws_sdk_s3 as s3;
use chrono;
use diesel::prelude::*;
use futures_util::{future::try_join3, StreamExt};

use actix_web::{post, put};
use aws_sdk_s3::presigning::PresigningConfig;
use rand::{self, Rng};
use shared::{
    db::{
        conn::DB_CONNECTION,
        models::{Bot, Game, NewGame, NewInvite, NewTeam, Team, TeamInvite, User},
        schema,
        schema::{team_invites, teams, users},
    },
    GameTask, PresignedRequest,
};

pub mod auth;
pub mod bots;
pub mod data;
pub mod games;
pub mod oauth;
pub mod profile;
pub mod team;

pub fn api_service() -> actix_web::Scope {
    actix_web::web::scope("/api")
        .service(team::create_team)
        .service(team::delete_team)
        .service(team::leave_team)
        .service(team::create_invite)
        .service(team::upload_pfp)
        .service(team::join_team)
        .service(team::cancel_invite)
        .service(team::kick_member)
        .service(team::rename_team)
        .service(team::update_owner)
        .service(bots::upload_bot)
        .service(bots::build_log)
        .service(bots::delete_bot)
        .service(bots::set_active_bot)
        .service(profile::my_account)
        .service(profile::my_team)
        .service(profile::my_email)
        .service(profile::get_profile)
        .service(profile::put_profile)
        .service(profile::get_resume)
        .service(profile::put_resume)
        .service(profile::delete_resume)
        .service(profile::get_resume_status)
        .service(profile::put_profile)
        .service(profile::schools)
        .service(data::teams)
        .service(data::bots)
        .service(data::code)
        .service(data::pfp)
        .service(games::create_game)
        .service(games::games)
        .service(games::count_games)
        .service(games::game_log)
}

pub fn auth_service() -> actix_web::Scope {
    actix_web::web::scope("/auth")
        .service(oauth::google_login)
        .service(oauth::microsoft_login)
        .service(auth::register)
        .service(auth::login)
        .service(auth::signout)
        .service(auth::reset_password)
        .service(auth::update_password)
        .service(auth::verify_verification_link)
}

type ApiResult<T> = Result<actix_web::web::Json<T>, ApiError>;

#[derive(Debug)]
pub struct ApiError {
    pub status_code: actix_web::http::StatusCode,
    pub message: String,
}
impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            json!({
                "status": self.status_code().as_u16(),
                "error": self.message,
            })
        )
    }
}
impl ResponseError for ApiError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        self.status_code
    }
}

impl From<actix_web::error::Error> for ApiError {
    fn from(err: actix_web::error::Error) -> Self {
        ApiError {
            status_code: err.as_response_error().status_code(),
            message: err.to_string(),
        }
    }
}

impl From<PayloadError> for ApiError {
    fn from(err: PayloadError) -> Self {
        ApiError {
            status_code: err.status_code(),
            message: err.to_string(),
        }
    }
}
impl<T> From<aws_sdk_s3::error::SdkError<T>> for ApiError {
    fn from(err: aws_sdk_s3::error::SdkError<T>) -> Self {
        ApiError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message: err.to_string(),
        }
    }
}

macro_rules! define_api_error {
    ($err:ty, $status_code:expr) => {
        impl From<$err> for ApiError {
            fn from(err: $err) -> Self {
                ApiError {
                    status_code: $status_code,
                    message: err.to_string(),
                }
            }
        }
    };
}

define_api_error!(std::num::ParseIntError, StatusCode::BAD_REQUEST);
define_api_error!(std::io::Error, StatusCode::INTERNAL_SERVER_ERROR);

define_api_error!(TryFromIntError, StatusCode::INTERNAL_SERVER_ERROR);
define_api_error!(aws_sdk_sqs::Error, StatusCode::INTERNAL_SERVER_ERROR);

define_api_error!(r2d2::Error, StatusCode::INTERNAL_SERVER_ERROR);
define_api_error!(serde_json::Error, StatusCode::INTERNAL_SERVER_ERROR);
define_api_error!(diesel::result::Error, StatusCode::INTERNAL_SERVER_ERROR);
define_api_error!(std::env::VarError, StatusCode::INTERNAL_SERVER_ERROR);
define_api_error!(PresigningConfigError, StatusCode::INTERNAL_SERVER_ERROR);
define_api_error!(ToStrError, StatusCode::INTERNAL_SERVER_ERROR);

define_api_error!(lettre::error::Error, StatusCode::INTERNAL_SERVER_ERROR);
define_api_error!(
    actix_session::SessionInsertError,
    StatusCode::INTERNAL_SERVER_ERROR
);
define_api_error!(
    actix_session::SessionGetError,
    StatusCode::INTERNAL_SERVER_ERROR
);
define_api_error!(
    lettre::transport::smtp::Error,
    StatusCode::INTERNAL_SERVER_ERROR
);
define_api_error!(reqwest::Error, StatusCode::INTERNAL_SERVER_ERROR);
define_api_error!(argon2::password_hash::Error, StatusCode::UNAUTHORIZED);
