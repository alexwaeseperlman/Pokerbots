use cfg_if::cfg_if;

cfg_if! {
if #[cfg(feature = "ssr")] {
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};
use dotenvy::dotenv;
use std::env;
use actix_web::*;
use pokerbots::app::login::handle_login;

fn get_secret_key() -> cookie::Key {
    let key = env::var("SECRET_KEY").expect("SECRET_KEY must be set in .env");
    cookie::Key::from(key.as_bytes())
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    //use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
    use actix_session::*;
    use leptos::*;
    use leptos_actix::{generate_route_list, LeptosRoutes};
    use pokerbots::app::*;

    dotenv().ok();

    let conf = get_configuration(None).await.unwrap();
    let addr = conf.leptos_options.site_addr;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(|cx| view! { cx, <App/> });

    register_server_functions();

    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;

        App::new()
            .route("/api/login", web::get().to(handle_login))
            .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
            .wrap(
                SessionMiddleware::new(CookieSessionStore::default(), get_secret_key())
            )
            .leptos_routes(
                leptos_options.to_owned(),
                routes.to_owned(),
                |cx| view! { cx, <App/> },
            )
            .service(Files::new("/", site_root))
        //.wrap(middleware::Compress::default())
    })
    .bind(&addr)?
    .run()
    .await
}
}
else {
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
}
}