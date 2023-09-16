use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::Utc;
use shared::db::{
    models::{Auth, NewAuth, NewUser, TeamWithMembers, UserProfile},
    schema::{auth, user_profiles},
};

use lettre::{message::header::ContentType, Message, Transport};
use uuid::Uuid;

use crate::config::website_origin;

use super::*;

const ALPHANUMERIC: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

fn random(charset: &[u8], len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset[idx] as char
        })
        .collect()
}

pub fn mangle(password: &str) -> argon2::password_hash::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|m| m.to_string())
}

pub fn get_display_name_from_email(email: &str) -> String {
    email.split('@').next().unwrap().to_string()
}

fn verify(password: &str, mangled: &str) -> argon2::password_hash::Result<()> {
    Argon2::default().verify_password(password.as_bytes(), &PasswordHash::new(mangled)?)
}

fn validate_password(password: &str) -> Result<(), ApiError> {
    if password.len() < 6 {
        return Err(ApiError {
            status_code: StatusCode::BAD_REQUEST,
            message: "Password must be at least 6 characters".to_string(),
        });
    }
    Ok(())
}

#[derive(Deserialize)]
struct RegisterPayload {
    pub email: String,
    pub password: String,
}
#[post("/email/register")]
async fn register(
    web::Json(RegisterPayload { email, password }): web::Json<RegisterPayload>,
) -> ApiResult<()> {
    let conn = &mut (*DB_CONNECTION).get().unwrap();

    if let Ok(auth) = auth::dsl::auth
        .filter(auth::dsl::email.eq(email.clone()))
        .first::<Auth>(conn)
    {
        if auth.email_confirmed {
            return Err(ApiError {
                status_code: StatusCode::BAD_REQUEST,
                message: "There is already an account with this email.".to_string(),
            });
        }
    };

    validate_password(&password)?;

    let email_verification_link = random(ALPHANUMERIC, 32);

    // TODO: add plain text fallback and improve html text
    let email_body = Message::builder()
        .from(config::email_address().parse().unwrap())
        .to(email.parse().unwrap())
        .subject("UPAC Email Verification")
        .header(ContentType::TEXT_HTML)
        .body(
            config::EMAIL_VERIFICATION_BODY.replace(
                "{}",
                format!(
                    "{}/verify-email/{}",
                    website_origin(),
                    email_verification_link
                )
                .as_str(),
            ),
        )
        .unwrap();
    config::MAILER.send(&email_body)?;

    let id = Uuid::new_v4();

    let auth = NewAuth {
        email: email.clone(),
        mangled_password: Some(mangle(&password)?),
        email_verification_link: Some(email_verification_link.clone()),
        email_verification_link_expiration: Some(
            Utc::now().naive_utc() + chrono::Duration::minutes(15),
        ),
        email_confirmed: false,
        id,
    };
    diesel::insert_into(users::dsl::users)
        .values(NewUser {
            display_name: get_display_name_from_email(&email),
            id,
        })
        .execute(conn)?;

    diesel::insert_into(auth::dsl::auth)
        .values(&auth)
        .on_conflict(auth::dsl::email)
        .do_update()
        .set(&auth)
        .execute(conn)?;
    let auth: Auth = auth::dsl::auth
        .filter(auth::dsl::email.eq(email.clone()))
        .first::<Auth>(conn)?;

    Ok(web::Json(()))
}

#[derive(Deserialize)]
struct LoginPayload {
    pub email: String,
    pub password: String,
}
#[post("/email/login")]
async fn login(
    session: Session,
    web::Json(LoginPayload { email, password }): web::Json<LoginPayload>,
) -> ApiResult<()> {
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    let auth: Auth = auth::dsl::auth
        .filter(auth::dsl::email.eq(&email))
        .first(conn)?;
    verify(
        &password,
        &auth.mangled_password.ok_or(ApiError {
            status_code: StatusCode::UNAUTHORIZED,
            message: "No password set".to_string(),
        })?,
    )?;

    if !auth.email_confirmed {
        return Err(ApiError {
            status_code: StatusCode::UNAUTHORIZED,
            message: "Email not confirmed".to_string(),
        });
    }

    session.insert("user", &auth.id)?;

    Ok(web::Json(()))
}

#[derive(Deserialize)]
struct LinkPayload {
    pub email: String,
}
#[post("/email/reset-password")]
async fn reset_password(web::Json(LinkPayload { email }): web::Json<LinkPayload>) -> ApiResult<()> {
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    let auth: Auth = auth::dsl::auth
        .filter(auth::dsl::email.eq(email.clone()))
        .first(conn)?;

    if !auth.email_confirmed {
        return Err(ApiError {
            status_code: StatusCode::UNAUTHORIZED,
            message: "Email not confirmed".to_string(),
        });
    }

    let password_reset_link = random(ALPHANUMERIC, 32);

    // send password reset link
    // TODO: add plain text fallback and improve html text
    let email_body = Message::builder()
        .from(config::email_address().parse().unwrap())
        .to(email.parse().unwrap())
        .subject("UPAC Password Reset")
        .header(ContentType::TEXT_HTML)
        .body(
            config::EMAIL_PASSWORD_RESET_BODY.replace(
                "{}",
                format!(
                    "{}/update-password/{}",
                    website_origin(),
                    password_reset_link
                )
                .as_str(),
            ),
        )
        .unwrap();

    config::MAILER.send(&email_body)?;

    diesel::update(auth::dsl::auth)
        .filter(auth::dsl::email.eq(&email))
        .set((
            auth::dsl::password_reset_link.eq(Some(&password_reset_link)),
            auth::dsl::password_reset_link_expiration
                .eq(Some(Utc::now().naive_utc() + chrono::Duration::minutes(15))),
        ))
        .execute(conn)?;

    Ok(web::Json(()))
}

#[derive(Deserialize)]
struct UpdatePasswordPayload {
    pub password: String,
}
#[post("/email/reset-password/{password_reset_link}")]
async fn update_password(
    password_reset_link: web::Path<String>,
    web::Json(UpdatePasswordPayload { password }): web::Json<UpdatePasswordPayload>,
) -> ApiResult<()> {
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    let auth: Auth = auth::dsl::auth
        .filter(auth::dsl::password_reset_link.eq(&password_reset_link.clone()))
        .first(conn)?;

    if auth.password_reset_link_expiration < Some(Utc::now().naive_utc()) {
        return Err(ApiError {
            status_code: StatusCode::UNAUTHORIZED,
            message: "Reset link expired".to_string(),
        });
    }

    diesel::update(auth::dsl::auth)
        .filter(auth::dsl::password_reset_link.eq(password_reset_link.as_str()))
        .set((
            auth::dsl::mangled_password.eq(mangle(&password)?),
            auth::dsl::password_reset_link.eq(None::<String>),
            auth::dsl::password_reset_link_expiration.eq(None::<chrono::NaiveDateTime>),
        ))
        .execute(conn)?;

    Ok(web::Json(()))
}

#[post("/email/verify/{email_verification_link}")]
async fn verify_verification_link(email_verification_link: web::Path<String>) -> ApiResult<()> {
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    let auth: Auth = auth::dsl::auth
        .filter(auth::dsl::email_verification_link.eq(email_verification_link.as_str()))
        .first(conn)?;
    if auth.email_confirmed {
        return Err(ApiError {
            status_code: StatusCode::CONFLICT,
            message: "Email already confirmed".to_string(),
        });
    }
    if auth.email_verification_link_expiration < Some(Utc::now().naive_utc()) {
        return Err(ApiError {
            status_code: StatusCode::UNAUTHORIZED,
            message: "Verification link expired".to_string(),
        });
    }
    if auth.email_verification_link != Some(email_verification_link.to_string()) {
        return Err(ApiError {
            status_code: StatusCode::UNAUTHORIZED,
            message: "Invalid verification link".to_string(),
        });
    }

    diesel::update(auth::dsl::auth)
        .filter(auth::dsl::email_verification_link.eq(email_verification_link.as_str()))
        .set((
            auth::dsl::email_confirmed.eq(true),
            auth::dsl::email_verification_link.eq(None::<String>),
            auth::dsl::email_verification_link_expiration.eq(None::<chrono::NaiveDateTime>),
        ))
        .execute(conn)?;

    Ok(web::Json(()))
}

#[derive(Serialize, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
pub struct SignoutResponse {
    pub message: String,
    pub message_type: String,
}
#[get("/signout")]
pub async fn signout(session: Session) -> ApiResult<SignoutResponse> {
    session.remove("user");

    Ok(actix_web::web::Json(SignoutResponse {
        message: "You have been signed out.".to_string(),
        message_type: "success".to_string(),
    }))
}

pub fn get_user(session: &Session) -> Option<User> {
    let id: Uuid = session.get("user").ok()??;
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    users::dsl::users
        .filter(users::dsl::id.eq(id))
        .first(conn)
        .ok()
}

pub fn get_profile(session: &Session) -> Option<UserProfile> {
    let id: Uuid = session.get("user").ok()??;
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    user_profiles::dsl::user_profiles
        .filter(user_profiles::dsl::id.eq(id))
        .first(conn)
        .ok()
}

pub fn get_team(session: &Session) -> Option<TeamWithMembers<User>> {
    let user = get_user(session)?;
    let conn = &mut (*DB_CONNECTION).get().unwrap();

    // TODO: make transaction
    let team: Team = teams::dsl::teams
        .filter(teams::dsl::id.eq(user.team?))
        .first(conn)
        .ok()?;

    let members: Vec<User> = users::dsl::users
        .filter(users::dsl::team.eq(team.id))
        .load(conn)
        .ok()?;

    let invites: Option<Vec<TeamInvite>> = Some(
        team_invites::dsl::team_invites
            .filter(team_invites::dsl::team.eq(team.id))
            .load(conn)
            .ok()?,
    );

    Some(TeamWithMembers {
        id: team.id,
        name: team.name,
        owner: team.owner,
        score: team.score,
        active_bot: team.active_bot,
        members,
        invites,
        deleted_at: None,
    })
}
