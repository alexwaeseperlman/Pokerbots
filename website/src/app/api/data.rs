use actix_session::Session;
use actix_web::{get, web, HttpResponse};
use s3::presigning::PresigningConfig;
use serde::Deserialize;

use crate::app::login;

use super::ServerMessage;
use aws_sdk_s3 as s3;

#[get("/api/my-account")]
pub async fn my_account(session: Session) -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(login::get_user_data(&session)))
}

#[get("/api/my-team")]
pub async fn my_team(session: Session) -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(login::get_team_data(&session)))
}

#[derive(Deserialize)]
pub struct ServerMessageQuery {
    pub clear: Option<bool>,
}

#[get("/api/server-message")]
pub async fn server_message(
    session: Session,
    web::Query::<ServerMessageQuery>(ServerMessageQuery { clear }): web::Query<ServerMessageQuery>,
) -> actix_web::Result<HttpResponse> {
    let msg = HttpResponse::Ok().json(session.get::<ServerMessage>("message")?);
    if clear.unwrap_or(false) {
        session.remove("message");
    }
    Ok(msg)
}

#[get("/api/pfp-url")]
pub async fn pfp_url(
    s3_client: actix_web::web::Data<s3::Client>,
) -> actix_web::Result<HttpResponse> {
    let req = s3_client
        .get_bucket_location()
        .bucket(std::env::var("PFP_S3_BUCKET").unwrap())
        .send()
        .await
        .map_err(|e| {
            log::error!("Error getting pfp URL: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;

    let s = req.location_constraint().unwrap().as_ref().to_string();
    Ok(HttpResponse::Ok().body(format!(
        "https://{}.s3.{}.amazonaws.com",
        std::env::var("PFP_S3_BUCKET").unwrap(),
        if s == "" {
            // https://docs.rs/aws-sdk-s3/latest/aws_sdk_s3/operation/get_bucket_location/struct.GetBucketLocationOutput.html#method.location_constraint
            // null is us-east-1 for some reason
            "us-east-1"
        } else {
            s.as_str()
        }
    )))
}
