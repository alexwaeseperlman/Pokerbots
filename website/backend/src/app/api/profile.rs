use actix_web::HttpRequest;
use diesel::prelude::*;
use shared::db::models::{TeamWithMembers, UserProfile};

use super::{team::validate_name, *};
#[get("/my-account")]
pub async fn my_account(session: Session) -> ApiResult<Option<User>> {
    Ok(web::Json(auth::get_user(&session)))
}

#[get("/my-team")]
pub async fn my_team(session: Session) -> ApiResult<Option<TeamWithMembers<User>>> {
    Ok(web::Json(auth::get_team(&session)))
}

#[get("/my-email")]
pub async fn my_email(session: Session) -> ApiResult<String> {
    let user = auth::get_user(&session);
    if let Some(user) = user {
        let conn = &mut (*DB_CONNECTION).get()?;
        let email = schema::auth::dsl::auth
            .filter(schema::auth::dsl::id.eq(user.id))
            .select(schema::auth::dsl::email)
            .first::<String>(conn)?;
        Ok(web::Json(email))
    } else {
        Err(ApiError {
            status_code: StatusCode::UNAUTHORIZED,
            message: "Not logged into an account".to_string(),
        })
    }
}

#[get("/schools")]
pub async fn schools() -> ApiResult<Vec<String>> {
    Ok(web::Json(config::schools()))
}

#[get("/profile")]
pub async fn get_profile(session: Session) -> ApiResult<Option<UserProfile>> {
    let user = auth::get_user(&session);
    if let Some(user) = user {
        let conn = &mut (*DB_CONNECTION).get()?;
        let profile = UserProfile::belonging_to(&user)
            .first::<UserProfile>(conn)
            .optional()?;
        Ok(web::Json(profile))
    } else {
        Ok(web::Json(None))
    }
}

#[derive(Deserialize, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
pub struct UpdateProfileRequest {
    pub display_name: String,
    pub first_name: String,
    pub last_name: String,
    pub country: Option<String>,
    pub school: String,
    pub linkedin: Option<String>,
    pub github: Option<String>,
}

#[put("/profile")]
pub async fn put_profile(session: Session, body: web::Json<UpdateProfileRequest>) -> ApiResult<()> {
    validate_update_profile_request(&body)?;
    let user = auth::get_user(&session);
    if let Some(user) = user {
        let conn = &mut (*DB_CONNECTION).get()?;
        let profile = UserProfile {
            first_name: body.first_name.clone(),
            last_name: body.last_name.clone(),
            country: body.country.clone(),
            school: body.school.clone(),
            linkedin: body.linkedin.clone(),
            github: body.github.clone(),
            id: user.id,
        };
        diesel::insert_into(schema::user_profiles::table)
            .values(&profile)
            .on_conflict(schema::user_profiles::dsl::id)
            .do_update()
            .set(&profile)
            .execute(conn)?;

        diesel::update(schema::users::table)
            .filter(schema::users::dsl::id.eq(user.id))
            .set(schema::users::dsl::display_name.eq(body.display_name.clone()))
            .execute(conn)?;

        Ok(web::Json(()))
    } else {
        return Err(ApiError {
            status_code: StatusCode::UNAUTHORIZED,
            message: "Not logged into an account".to_string(),
        });
    }
}

#[delete("/resume")]
pub async fn delete_resume(
    s3_client: actix_web::web::Data<aws_sdk_s3::Client>,
    session: Session,
) -> ApiResult<()> {
    let user =
        auth::get_user(&session).ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;

    s3_client
        .delete_object()
        .bucket(config::resume_s3_bucket())
        .key(user.id.to_string())
        .send()
        .await?;

    Ok(web::Json(()))
}

#[get("/resume-status")]
pub async fn get_resume_status(
    s3_client: actix_web::web::Data<aws_sdk_s3::Client>,
    session: Session,
) -> ApiResult<()> {
    let user =
        auth::get_user(&session).ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;

    let object = s3_client
        .get_object()
        .bucket(config::resume_s3_bucket())
        .key(user.id.to_string())
        .send()
        .await?;

    Ok(web::Json(()))
}

#[get("/resume")]
pub async fn get_resume(
    s3_client: actix_web::web::Data<aws_sdk_s3::Client>,
    session: Session,
) -> Result<HttpResponse, ApiError> {
    let user =
        auth::get_user(&session).ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;

    let object = s3_client
        .get_object()
        .bucket(config::resume_s3_bucket())
        .key(user.id.to_string())
        .send()
        .await?;

    Ok(HttpResponse::Ok()
        .content_type("application/pdf")
        .append_header(("Content-Disposition", "attachment; filename=\"resume.pdf\""))
        .streaming(object.body))
}

#[put("/resume")]
pub async fn put_resume(
    s3_client: actix_web::web::Data<aws_sdk_s3::Client>,
    session: Session,
    request: actix_web::HttpRequest,
    mut payload: web::Payload,
) -> ApiResult<()> {
    let user =
        auth::get_user(&session).ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;

    if request.headers().get("content-type")
        != Some(&reqwest::header::HeaderValue::from_static(
            "application/pdf",
        ))
    {
        return Err(actix_web::error::ErrorBadRequest("Invalid content type").into());
    }

    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > (config::bot_size()).try_into()? {
            return Err(actix_web::error::ErrorBadRequest("File too large").into());
        }
        body.extend_from_slice(&chunk);
    }
    s3_client
        .put_object()
        .bucket(config::resume_s3_bucket())
        .key(user.id.to_string())
        .body(body.to_vec().into())
        .send()
        .await?;

    Ok(web::Json(()))
}

pub fn validate_update_profile_request(body: &UpdateProfileRequest) -> Result<(), ApiError> {
    validate_name(&body.first_name,  "Invalid first name. It must be at most 20 characters and cannot contain consecutive spaces.")?;
    validate_name(&body.last_name,  "Invalid last name. It must be at most 20 characters and cannot contain consecutive spaces.")?;
    validate_name(&body.school,  "Invalid school name. It must be at most 20 characters and cannot contain consecutive spaces.")?;
    validate_name(&body.display_name,  "Invalid display name. It must be at most 20 characters and cannot contain consecutive spaces.")?;
    Ok(())
}
