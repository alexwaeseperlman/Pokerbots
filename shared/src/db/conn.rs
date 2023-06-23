use lazy_static::lazy_static;
use std::{env, fs};

use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};

lazy_static! {
    // Build a database connection pool
    pub static ref DB_CONNECTION: Pool<ConnectionManager<PgConnection>> = {
        let db_url = env::var("DB_URL").expect("DB_URL must be set");
        let db_password = env::var("DB_PASSWORD").unwrap_or_else(|_| {
            fs::read_to_string("/run/secrets/db-password")
                .expect("db-password must be set in .env or /run/secrets/db-password")
        });
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
