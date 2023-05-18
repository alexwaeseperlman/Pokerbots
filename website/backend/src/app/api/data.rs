use actix_session::Session;
use actix_web::{get, web, HttpResponse};
use diesel::*;
use s3::presigning::PresigningConfig;
use serde::Deserialize;

use crate::{
    app::login,
    config::{DB_CONNECTION, PFP_S3_BUCKET},
    models::Team,
    schema,
};

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
        .bucket(&*PFP_S3_BUCKET)
        .send()
        .await
        .map_err(|e| {
            log::error!("Error getting pfp URL: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;

    let s = req.location_constraint().unwrap().as_ref().to_string();
    Ok(HttpResponse::Ok().body(format!(
        "https://{}.s3.{}.amazonaws.com",
        *crate::config::PFP_S3_BUCKET,
        if s == "" {
            // https://docs.rs/aws-sdk-s3/latest/aws_sdk_s3/operation/get_bucket_location/struct.GetBucketLocationOutput.html#method.location_constraint
            // null is us-east-1 for some reason
            "us-east-1"
        } else {
            s.as_str()
        }
    )))
}

#[derive(Deserialize)]
pub struct TeamQuery {
    pub ids: Option<Vec<i32>>,
    pub page_size: Option<i32>,
    pub page: Option<i32>,
}

#[get("/api/teams")]
pub async fn teams(
    session: Session,
    web::Query::<TeamQuery>(TeamQuery {
        ids,
        page_size,
        page,
    }): web::Query<TeamQuery>,
) -> actix_web::Result<HttpResponse> {
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    let mut base = schema::teams::dsl::teams
        .order_by(schema::teams::dsl::id.desc())
        .into_boxed();
    if let Some(ids) = ids {
        base = base.filter(schema::teams::dsl::id.eq_any(ids));
    }
    let page_size = page_size.unwrap_or(10).min(100);
    let page = page.unwrap_or(0);
    base = base
        .limit((page_size).into())
        .offset((page * page_size).into());
    let result: Vec<Team> = base
        .load::<Team>(conn)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Unable to update game: {}", e))
        })?
        .into_iter()
        .collect();
    Ok(HttpResponse::Ok().json(result))
}
