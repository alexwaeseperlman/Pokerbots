use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use dotenvy::dotenv;
use lazy_static::lazy_static;
use std::{env, fs};

// Build a database connection pool for server functions
lazy_static! {
    pub static ref DB_CONNECTION: Pool<ConnectionManager<PgConnection>> = {
        dotenv().ok();
        let db_url = env::var("DB_URL").expect("DB_URL must be set");
        let db_password = env::var("DB_PASSWORD").expect("DB_PASSWORD must be set");
        let db_user = env::var("DB_USER").expect("DB_USER must be set");

        Pool::builder()
            .max_size(15)
            .build(ConnectionManager::<PgConnection>::new(format!(
                "postgres://{}:{}@{}",
                db_user, db_password, db_url
            )))
            .unwrap()
    };
}

pub const TEAM_SIZE: i32 = 5;

lazy_static! {
    pub static ref CLIENT_ID: String =
        std::env::var("MICROSOFT_CLIENT_ID").expect("MICROSOFT_CLIENT_ID must be set in .env");
    pub static ref REDIRECT_URI: String =
        std::env::var("REDIRECT_URI").expect("REDIRECT_URI must be set in .env");
    pub static ref TENANT_ID: String =
        std::env::var("MICROSOFT_TENANT_ID").expect("MICROSOFT_TENANT_ID must be set in .env");
    pub static ref PFP_S3_BUCKET: String =
        std::env::var("APP_PFP_S3_BUCKET").expect("APP_PFP_S3_BUCKET must be set in .env");
    pub static ref AZURE_SECRET: String = std::env::var("AZURE_SECRET")
        .unwrap_or_else(|_| fs::read_to_string("/run/secrets/AZURE_SECRET")
            .expect("AZURE_SECRET must be set in .env or /run/secrets/azure_secret"));
}
