use cfg_if::cfg_if;

use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::*;
use dotenvy::dotenv;
use handlebars::template::*;
use handlebars::*;
use pokerbots::app::login::{get_user_data, handle_login};
use std::{borrow::Borrow, env};

fn get_secret_key() -> cookie::Key {
    let key = env::var("SECRET_KEY").expect("SECRET_KEY must be set in .env");
    cookie::Key::from(key.as_bytes())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    //use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
    use actix_service::apply;
    use actix_service::*;
    use actix_session::*;
    use actix_web::*;
    use futures_util::future::FutureExt;
    use pokerbots::app::login::{get_team_data, get_user_data};
    use pokerbots::app::*;

    use actix_web::{middleware::Logger, App};
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    dotenv().ok();

    // Generate the list of routes in your App
    HttpServer::new(move || {
        let session_middleware =
            SessionMiddleware::builder(CookieSessionStore::default(), get_secret_key())
                .cookie_secure(true)
                .build();
        let mut hbars = Handlebars::new();
        hbars.set_strict_mode(true);
        hbars.register_templates_directory(".hbs", "templates");
        println!("{:?}", hbars.get_templates());
        let hbars_ref = web::Data::new(hbars);

        let a = App::new()
            .wrap_fn(|req, srv| {
                let user_data = get_user_data(Some(req.get_session().clone()));
                let team_data = get_team_data(Some(req.get_session().clone()));
                req.extensions_mut().insert(user_data);
                req.extensions_mut().insert(team_data);
                println!("Add data stuff");
                srv.call(req).map(|res| res)
            })
            .app_data(hbars_ref.clone())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(session_middleware)
            .route("/api/login", web::get().to(handle_login))
            .service(Files::new("/static", "static/"))
            .service(pokerbots::app::home_page);
        a
        //.wrap(middleware::Compress::default())
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}
