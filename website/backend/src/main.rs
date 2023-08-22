use std::fs;

use actix_service::Service;
use actix_session::{storage::CookieSessionStore, SessionExt, SessionMiddleware};
use actix_web::{cookie, middleware::Logger, web, App, HttpMessage, HttpServer};
use futures_util::future::FutureExt;
use pokerbots::app::{api, login, pages};
use shared::db::conn::DB_CONNECTION;

fn get_secret_key() -> cookie::Key {
    let key = std::env::var("SECRET_KEY").unwrap_or_else(|_| {
        fs::read_to_string("/run/secrets/secret-key")
            .expect("SECRET_KEY must be set in .env or /run/secrets/secret-key")
    });
    assert!(
        key.len() >= 64,
        "SECRET_KEY must be at least 64 characters long"
    );
    cookie::Key::from(key.as_bytes())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenvy::dotenv().ok();

    let conn = &mut (*DB_CONNECTION).get().unwrap();
    shared::db::run_pending_migrations(conn);
    let aws_config = shared::aws_config().await;
    let s3_client = web::Data::new(shared::s3_client(&aws_config).await);
    let sqs_client = web::Data::new(shared::sqs_client(&aws_config).await);

    // Generate the list of routes in your App
    HttpServer::new(move || {
        let session_middleware =
            SessionMiddleware::builder(CookieSessionStore::default(), get_secret_key())
                .cookie_secure(true)
                .build();
        App::new()
            .wrap_fn(|req, srv| {
                let user_data = login::get_user_data(&req.get_session());
                let team_data = login::get_team_data(&req.get_session());
                req.extensions_mut().insert(user_data);
                req.extensions_mut().insert(team_data);
                log::debug!("{}", req.uri());
                srv.call(req).map(|res| res)
            })
            //.app_data(amqp_channel.clone())
            .app_data(s3_client.clone())
            .app_data(sqs_client.clone())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(session_middleware)
            //.route("/api/login", web::get().to(login::handle_login))
            .service(login::login_provider)
            .service(api::api_service())
            .service(actix_files::Files::new("/static", "./static"))
            .service(pages::service())

        //.wrap(middleware::Compress::default())
    })
    .workers(2)
    .bind((
        "0.0.0.0",
        std::env::var("PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(80),
    ))?
    .run()
    .await
}
