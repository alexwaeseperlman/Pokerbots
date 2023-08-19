use lazy_static::lazy_static;
use std::fs;

pub const TEAM_SIZE: usize = 5;
pub fn client_id() -> String {
    std::env::var("MICROSOFT_CLIENT_ID").expect("MICROSOFT_CLIENT_ID must be set in .env")
}

pub fn redirect_uri() -> String {
    std::env::var("REDIRECT_URI").expect("REDIRECT_URI must be set in .env")
}

pub fn tenant_id() -> String {
    std::env::var("MICROSOFT_TENANT_ID").expect("MICROSOFT_TENANT_ID must be set in .env")
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

pub fn app_pfp_endpoint() -> String {
    std::env::var("APP_PFP_ENDPOINT").expect("APP_PFP_ENDPOINT must be set in .env")
}

pub fn azure_secret() -> String {
    std::env::var("AZURE_SECRET")
        .or_else(|_| fs::read_to_string("/run/secrets/azure-secret"))
        .expect("AZURE_SECRET must be set in .env or /run/secrets/azure-secret")
}

pub fn bot_size() -> u64 {
    std::env::var("BOT_SIZE")
        .expect("BOT_SIZE must be set in .env")
        .parse()
        .expect("BOT_SIZE must be a number")
}
