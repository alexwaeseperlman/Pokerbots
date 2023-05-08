use actix_files::NamedFile;
use actix_service::{fn_service, Service};
use actix_session::{storage::CookieSessionStore, SessionExt, SessionMiddleware};
use actix_web::{
    cookie,
    dev::{ServiceRequest, ServiceResponse},
    get,
    http::header::{ContentDisposition, DispositionType},
    middleware::Logger,
    web, App, HttpMessage, HttpRequest, HttpServer,
};
use futures_util::future::FutureExt;

use pokerbots::app::{api, login};

fn get_secret_key() -> cookie::Key {
    let key = std::env::var("SECRET_KEY").expect("SECRET_KEY must be set in .env");
    cookie::Key::from(key.as_bytes())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    dotenvy::dotenv().ok();

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
            .route("/api/login", web::get().to(login::handle_login))
            .service(login::login_provider)
            .service(api::manage_team::create_team)
            .service(api::manage_team::delete_team)
            .service(api::manage_team::leave_team)
            .service(api::manage_team::make_invite)
            .service(api::manage_team::join_team)
            .service(api::manage_team::cancel_invite)
            .service(api::data::my_account)
            .service(api::data::server_message)
            .service(api::data::my_team)
            .service(api::signout::signout)
            // All remaining paths go to /app/dist, and fallback to index.html for client side routing
            .service(
                actix_files::Files::new("/", "app/dist/")
                    .index_file("/index.html")
                    .default_handler(fn_service(|req: ServiceRequest| async {
                        let (req, _) = req.into_parts();

                        let f = NamedFile::open_async("app/dist/index.html")
                            .await?
                            .into_response(&req);
                        Ok(ServiceResponse::new(req, f))
                    })),
            )

        //.wrap(middleware::Compress::default())
    })
    .workers(8)
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}
