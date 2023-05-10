use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use dotenvy::dotenv;
use lazy_static::lazy_static;
use std::env;

// Build a database connection pool for server functions
lazy_static! {
    pub static ref DB_CONNECTION: Pool<ConnectionManager<PgConnection>> = {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        Pool::builder()
            .max_size(15)
            .build(ConnectionManager::<PgConnection>::new(database_url))
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
    pub static ref AZURE_SECRET: String =
        std::env::var("AZURE_SECRET").expect("APP_PFP_S3_BUCKET must be set in .env");
    pub static ref FRONTEND_URL: String =
        std::env::var("APP_FRONTEND_URL").expect("APP_PFP_S3_BUCKET must be set in .env");
}
