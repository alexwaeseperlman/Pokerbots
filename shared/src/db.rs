use diesel::backend::Backend;
use diesel_migrations::MigrationHarness;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
pub mod conn;
pub mod dao;
pub mod models;
pub mod schema;

pub fn run_pending_migrations<T: MigrationHarness<U>, U: Backend>(conn: &mut T) {
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}
