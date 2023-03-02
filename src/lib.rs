pub mod app;
pub mod app_config;
pub mod models;
pub mod schema;

use cfg_if::cfg_if;

cfg_if! {
if #[cfg(feature = "hydrate")] {

  use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    pub fn hydrate() {
      use app::*;
      use leptos::*;

      // initializes logging using the `log` crate
      _ = console_log::init_with_level(log::Level::Debug);
      console_error_panic_hook::set_once();

      leptos::mount_to_body(move |cx| {
          view! { cx, <App/> }
      });
    }
}
else if #[cfg(feature="ssr")]{
use dotenvy::dotenv;
use std::env;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use std::{sync::Mutex};
use r2d2::Pool;
use diesel::r2d2::ConnectionManager;


use lazy_static::lazy_static;

// Build a database connection pool for server functions
lazy_static! {
  pub static ref DB_CONNECTION: Pool<ConnectionManager<SqliteConnection>> =  {
      dotenv().ok();
      let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
      Pool::builder()
        .max_size(15)
        .build(ConnectionManager::<SqliteConnection>::new(database_url))
        .unwrap()
    };

}
// Get the session out of a scope. Useful for server functions
pub fn get_session(cx: leptos::Scope) -> Option<actix_session::Session> {
    use actix_session::SessionExt;
    let req = leptos::use_context::<actix_web::HttpRequest>(cx);
    if req.is_none() {
      None
    }
    else {
      Some(req.unwrap().get_session())
    }
}

pub fn get_azure_secret() -> String {
    use std::env;
    env::var("AZURE_SECRET").expect("AZURE_SECRET must be set in .env")
}

}

}
