use std::{fmt, num::TryFromIntError};

use actix_web::http::StatusCode;
use actix_web::{error::PayloadError, HttpResponse, ResponseError};
use aws_sdk_s3::presigning::PresigningConfigError;
use reqwest::header::ToStrError;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub mod bots;
pub mod data;
pub mod games;
pub mod manage_team;
pub mod signout;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ServerMessage {
    pub message: String,
    pub message_type: String,
}

pub fn api_service() -> actix_web::Scope {
    actix_web::web::scope("/api")
        .service(manage_team::create_team)
        .service(manage_team::delete_team)
        .service(manage_team::leave_team)
        .service(manage_team::create_invite)
        .service(manage_team::upload_pfp)
        .service(manage_team::join_team)
        .service(manage_team::cancel_invite)
        .service(manage_team::kick_member)
        .service(manage_team::rename_team)
        .service(bots::upload_bot)
        .service(bots::build_log)
        .service(bots::delete_bot)
        .service(bots::set_active_bot)
        .service(data::my_account)
        .service(data::my_team)
        .service(data::teams)
        .service(data::bots)
        .service(data::invite_code)
        .service(data::pfp_endpoint)
        .service(games::create_game)
        .service(games::games)
        .service(games::game_log)
        .service(signout::signout)
}

type ApiResult = Result<HttpResponse, ApiError>;
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
define_api_error!(
    actix_session::SessionGetError,
    StatusCode::INTERNAL_SERVER_ERROR
);

define_api_error!(r2d2::Error, StatusCode::INTERNAL_SERVER_ERROR);
define_api_error!(serde_json::Error, StatusCode::INTERNAL_SERVER_ERROR);
define_api_error!(diesel::result::Error, StatusCode::INTERNAL_SERVER_ERROR);
define_api_error!(std::env::VarError, StatusCode::INTERNAL_SERVER_ERROR);
define_api_error!(PresigningConfigError, StatusCode::INTERNAL_SERVER_ERROR);
define_api_error!(ToStrError, StatusCode::INTERNAL_SERVER_ERROR);
