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
pub const CLIENT_ID: &str = "cc6185f1-7e94-4314-a79e-7d72d8fd68fc";
pub const REDIRECT_URI: &str = "http%3A%2F%2Flocalhost:3000%2Fapi%2Flogin";
pub const TENANT_ID: &str = "f8cdef31-a31e-4b4a-93e4-5f571e91255a";
