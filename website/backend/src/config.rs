use lazy_static::lazy_static;
use lettre::{
    transport::smtp::{authentication::Credentials, PoolConfig},
    SmtpTransport,
};
use reqwest::Client;
use std::fs;

use lettre::transport::smtp::client::{Tls, TlsParameters};

pub const TEAM_SIZE: usize = 5;

pub fn microsoft_client_id() -> String {
    std::env::var("APP_MICROSOFT_CLIENT_ID").expect("MICROSOFT_CLIENT_ID must be set in .env")
}

pub fn azure_secret() -> String {
    std::env::var("AZURE_SECRET").unwrap_or_else(|_| {
        fs::read_to_string("/run/secrets/azure-secret")
            .expect("AZURE_SECRET must be set in .env or /run/secrets/azure-secret")
    })
}

pub fn website_origin() -> String {
    std::env::var("WEBSITE_ORIGIN").expect("WEBSITE_ORIGIN must be set in .env")
}

pub fn microsoft_redirect_uri() -> String {
    format!("{}/{}", website_origin(), "login/microsoft")
}

pub fn google_client_id() -> String {
    std::env::var("APP_GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set in .env")
}

pub fn google_secret() -> String {
    std::env::var("GOOGLE_SECRET").unwrap_or_else(|_| {
        fs::read_to_string("/run/secrets/google-secret")
            .expect("GOOGLE_SECRET must be set in .env or /run/secrets/google-secret")
    })
}

pub fn google_redirect_uri() -> String {
    format!("{}/{}", website_origin(), "login/google")
}

pub fn pfp_s3_bucket() -> String {
    std::env::var("PFP_S3_BUCKET").expect("PFP_S3_BUCKET must be set in .env")
}

pub fn bot_s3_bucket() -> String {
    std::env::var("BOT_S3_BUCKET").expect("BOT_S3_BUCKET must be set in .env")
}

pub fn build_logs_s3_bucket() -> String {
    std::env::var("BUILD_LOGS_S3_BUCKET").expect("BUILD_LOGS_S3_BUCKET must be set in .env")
}

pub fn game_logs_s3_bucket() -> String {
    std::env::var("GAME_LOGS_S3_BUCKET").expect("GAME_LOGS_S3_BUCKET must be set in .env")
}

pub fn bot_size() -> u64 {
    std::env::var("BOT_SIZE")
        .expect("BOT_SIZE must be set in .env")
        .parse()
        .expect("BOT_SIZE must be a number")
}

pub fn email_address() -> String {
    std::env::var("EMAIL_ADDRESS").expect("EMAIL_ADDRESS must be set in .env")
}

pub fn email_app_password() -> String {
    std::env::var("EMAIL_APP_PASSWORD").expect("EMAIL_APP_PASSWORD must be set in .env")
}

pub fn smtp_server() -> String {
    std::env::var("SMTP_SERVER").expect("SMTP_SERVER must be set in .env")
}

pub const EMAIL_VERIFICATION_BODY: &str = std::include_str!("emails/email_verify.html");
pub const EMAIL_PASSWORD_RESET_BODY: &str = std::include_str!("emails/password_reset.html");

lazy_static! {
    pub static ref MAILER: SmtpTransport = SmtpTransport::relay(&smtp_server())
        .unwrap()
        .credentials(Credentials::new(
            email_address().to_string(),
            email_app_password().to_string(),
        ))
        .tls(Tls::Wrapper(
            TlsParameters::new(smtp_server().to_string()).unwrap()
        ))
        .pool_config(PoolConfig::new())
        .build();
    pub static ref CLIENT: Client = reqwest::Client::new();
}
