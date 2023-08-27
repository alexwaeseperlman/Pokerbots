// TODO: oauth
// TODO: sessions
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::Utc;
use shared::db::{models::Auth, schema::auth};

use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    SmtpTransport, Transport,
};

use super::*;

// https://demo.react.email/preview/plaid-verify-identity?view=source&lang=jsx
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

fn mangle(password: &str) -> argon2::password_hash::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password((config::PEPPER.clone() + password).as_bytes(), &salt)
        .map(|m| m.to_string())
}

fn verify(password: &str, mangled: &str) -> argon2::password_hash::Result<()> {
    Argon2::default().verify_password(
        (config::PEPPER.clone() + password).as_bytes(),
        &PasswordHash::new(mangled)?,
    )
}

#[derive(Deserialize)]
pub struct RegisterPayload {
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
                message: "Email already confirmed".to_string(),
            });
        }
    };

    let email_verification_link = random(ALPHANUMERIC, 32);

    // TODO: add plain text fallback and improve html text
    let email_body = Message::builder()
        .from(config::ALIAS_EMAIL.parse().unwrap())
        .to(email.parse().unwrap())
        .subject("UPAC Email Verification")
        .header(ContentType::TEXT_HTML)
        .body(config::EMAIL_EMAIL_VERIFICATION_BODY.replace(
            "{}",
            &config::EMAIL_VERIFICATION_LINK_URI.replace("{}", &email_verification_link),
        ))
        .unwrap();
    config::MAILER.send(&email_body)?;

    let auth = Auth {
        email: email.clone(),
        mangled_password: mangle(&password)?,
        email_verification_link: Some(email_verification_link.clone()),
        email_verification_link_expiration: Some(
            Utc::now().naive_utc() + chrono::Duration::minutes(15),
        ),
        password_reset_link: None,
        password_reset_link_expiration: None,
        email_confirmed: false,
        is_admin: false,
    };

    diesel::insert_into(auth::dsl::auth)
        .values(&auth)
        .on_conflict(auth::dsl::email)
        .do_update()
        .set(&auth)
        .execute(conn)?;

    println!(
        "{:?}",
        Utc::now().naive_utc() + chrono::Duration::minutes(15)
    );

    Ok(web::Json(()))
}

#[derive(Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}
#[post("/email/login")]
async fn login(
    session: Session,
    web::Json(LoginPayload { email, password }): web::Json<LoginPayload>,
) -> ApiResult<()> {
    // add to session
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    let auth: Auth = auth::dsl::auth
        .filter(auth::dsl::email.eq(email))
        .first(conn)?;
    verify(&password, &auth.mangled_password)?;

    if !auth.email_confirmed {
        return Err(ApiError {
            status_code: StatusCode::UNAUTHORIZED,
            message: "Email not confirmed".to_string(),
        });
    }
    Ok(web::Json(()))
}

#[post("/oauth/login")]
async fn oauth_login(session: Session) -> ApiResult<()> {
    // Dummy OAuth login logic
    Ok(web::Json(()))
}

#[derive(Deserialize)]
pub struct LinkPayload {
    pub email: String,
}
#[post("/password-reset/link")]
async fn create_link(web::Json(LinkPayload { email }): web::Json<LinkPayload>) -> ApiResult<()> {
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
        .from(config::ALIAS_EMAIL.parse().unwrap())
        .to(email.parse().unwrap())
        .subject("UPAC Password Reset")
        .header(ContentType::TEXT_HTML)
        .body(config::EMAIL_PASSWORD_RESET_BODY.replace(
            "{}",
            &config::PASSWORD_RESET_LINK_URI.replace("{}", &password_reset_link),
        ))
        .unwrap();

    let creds = Credentials::new(
        config::UNDERLYING_EMAIL.to_string(),
        config::EMAIL_APP_PASSWORD.to_string(),
    );

    let mailer = SmtpTransport::relay(&config::SMTP_SERVER)
        .unwrap()
        .credentials(creds)
        .build();

    mailer.send(&email_body)?;

    diesel::update(auth::dsl::auth)
        .filter(auth::dsl::email.eq(email.clone()))
        .set((
            auth::dsl::password_reset_link.eq(Some(password_reset_link.clone())),
            auth::dsl::password_reset_link_expiration
                .eq(Some(Utc::now().naive_utc() + chrono::Duration::minutes(15))),
        ))
        .execute(conn)?;

    Ok(web::Json(()))
}

#[get("/password-reset/verify/{password_reset_link}")]
async fn verify_reset_link(password_reset_link: web::Path<String>) -> ApiResult<()> {
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    let auth: Auth = auth::dsl::auth
        .filter(auth::dsl::password_reset_link.eq(password_reset_link.as_str()))
        .first(conn)?;

    if auth.password_reset_link_expiration < Some(Utc::now().naive_utc()) {
        return Err(ApiError {
            status_code: StatusCode::UNAUTHORIZED,
            message: "Link expired".to_string(),
        });
    }
    Ok(web::Json(()))
}

#[derive(Deserialize)]
pub struct UpdatePasswordPayload {
    pub password: String,
}
#[put("/password-reset/{password_reset_link}")]
async fn update_password(
    password_reset_link: web::Path<String>,
    web::Json(UpdatePasswordPayload { password }): web::Json<UpdatePasswordPayload>,
) -> ApiResult<()> {
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    let auth: Auth = auth::dsl::auth
        .filter(auth::dsl::password_reset_link.eq(password_reset_link.clone()))
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
