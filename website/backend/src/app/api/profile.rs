use diesel::prelude::*;
use shared::db::models::{TeamWithMembers, UserProfile};

use super::*;
#[get("/my-account")]
pub async fn my_account(session: Session) -> ApiResult<Option<User>> {
    Ok(web::Json(auth::get_user(&session)))
}

#[get("/my-team")]
pub async fn my_team(session: Session) -> ApiResult<Option<TeamWithMembers<User>>> {
    Ok(web::Json(auth::get_team(&session)))
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
    pub first_name: String,
    pub last_name: String,
    pub country: Option<String>,
    pub school: String,
    pub linkedin: Option<String>,
    pub github: Option<String>,
}

#[put("/profile")]
pub async fn put_profile(session: Session, body: web::Json<UpdateProfileRequest>) -> ApiResult<()> {
    let user = auth::get_user(&session);
    if let Some(user) = user {
        let conn = &mut (*DB_CONNECTION).get()?;
        let profile = UserProfile::belonging_to(&user)
            .first::<UserProfile>(conn)
            .optional()?;
        diesel::insert_into(schema::user_profiles::table)
            .values(UserProfile {
                first_name: body.first_name.clone(),
                last_name: body.last_name.clone(),
                country: body.country.clone(),
                school: body.school.clone(),
                linkedin: body.linkedin.clone(),
                github: body.github.clone(),
                resume_s3_key: profile.map(|p| p.resume_s3_key).flatten(),
                id: user.id,
            })
            .execute(conn)?;

        Ok(web::Json(()))
    } else {
        return Err(ApiError {
            status_code: StatusCode::UNAUTHORIZED,
            message: "Not logged into an account".to_string(),
        });
    }
}
