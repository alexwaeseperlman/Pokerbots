use actix_service::Service;
use actix_session::{storage::CookieSessionStore, SessionExt, SessionMiddleware};
use actix_web::{cookie, middleware::Logger, web, App, HttpMessage, HttpServer};
use futures_util::future::FutureExt;
use std::sync::Mutex;

use pokerbots::app::{
    api::{self, upload_bot::BotsList},
    login, pages,
};

fn get_secret_key() -> cookie::Key {
    let key = std::env::var("SECRET_KEY").expect("SECRET_KEY must be set in .env");
    cookie::Key::from(key.as_bytes())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    dotenvy::dotenv().ok();
    let bots_list = web::Data::new(BotsList {
        bots: Mutex::new(Vec::new()),
    });

    // Generate the list of routes in your App
    HttpServer::new(move || {
        let session_middleware =
            SessionMiddleware::builder(CookieSessionStore::default(), get_secret_key())
                .cookie_secure(true)
                .build();
        let mut hbars = handlebars::Handlebars::new();
        hbars.set_strict_mode(true);
        hbars
            .register_templates_directory(".hbs", "templates")
            .expect("Failed to load templates");

        App::new()
            .wrap_fn(|req, srv| {
                let user_data = login::get_user_data(&req.get_session());
                let team_data = login::get_team_data(&req.get_session());
                req.extensions_mut().insert(user_data);
                req.extensions_mut().insert(team_data);
                log::debug!("{}", req.uri());
                srv.call(req).map(|res| res)
            })
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(session_middleware)
            .app_data(web::Data::new(hbars))
            .route("/api/login", web::get().to(login::handle_login))
            .service(actix_files::Files::new("/static", "static/"))
            .service(pages::home::home)
            .service(pages::team::team)
            .service(pages::manage_team::manage_team)
            .service(api::manage_team::create_team)
            .service(api::manage_team::delete_team)
            .service(api::manage_team::leave_team)
            .service(api::manage_team::make_invite)
            .service(api::manage_team::join_team)
            .service(api::signout::signout)
            .service(api::upload_bot::upload_bot)
            .app_data(bots_list.clone())

        //.wrap(middleware::Compress::default())
    })
    .workers(8)
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}
