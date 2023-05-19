use std::{fs, sync::Arc};

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
use aws_config;
use aws_sdk_s3::types::{
    builders::CreateBucketConfigurationBuilder, CreateBucketConfiguration, ObjectOwnership,
    OwnershipControls, OwnershipControlsRule, PublicAccessBlockConfiguration,
};
use diesel_migrations::*;
use futures_util::{future::FutureExt, StreamExt};
use lapin::options::ConfirmSelectOptions;
use pokerbots::{
    app::{api, login},
    config::DB_CONNECTION,
};

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

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenvy::dotenv().ok();

    let conn = &mut (*DB_CONNECTION).get().unwrap();
    conn.run_pending_migrations(MIGRATIONS).unwrap();
    let aws_config = aws_config::load_from_env().await;
    let s3_client = web::Data::new(aws_sdk_s3::Client::new(&aws_config));

    let addr = std::env::var("AMQP_URL").expect("AMQP_URL must be set");
    let conn = lapin::Connection::connect(&addr, lapin::ConnectionProperties::default())
        .await
        .expect("Connection error");

    // listen for messages
    let channel = conn.create_channel().await.unwrap();
    let channel_b = conn.create_channel().await.unwrap();
    let mut queue = channel
        .queue_declare(
            "poker",
            lapin::options::QueueDeclareOptions::default(),
            lapin::types::FieldTable::default(),
        )
        .await
        .unwrap();

    let mut queue_b = channel_b
        .queue_declare(
            "game_results",
            lapin::options::QueueDeclareOptions::default(),
            lapin::types::FieldTable::default(),
        )
        .await
        .unwrap();
    /*
    s3_client
        .create_bucket()
        .bucket(&*pokerbots::config::PFP_S3_BUCKET)
        .send()
        .await
        .unwrap();

    s3_client
        .delete_public_access_block()
        .bucket(&*pokerbots::config::PFP_S3_BUCKET)
        .send()
        .await
        .unwrap();

    s3_client
        .put_bucket_ownership_controls()
        .bucket(&*pokerbots::config::PFP_S3_BUCKET)
        .ownership_controls(
            OwnershipControls::builder()
                .rules(
                    OwnershipControlsRule::builder()
                        .object_ownership(ObjectOwnership::BucketOwnerPreferred)
                        .build(),
                )
                .build(),
        )
        .send()
        .await
        .unwrap();

    s3_client
        .put_bucket_cors()
        .bucket(&*pokerbots::config::PFP_S3_BUCKET)
        .cors_configuration(
            aws_sdk_s3::types::CorsConfiguration::builder()
                .cors_rules(
                    aws_sdk_s3::types::CorsRule::builder()
                        .allowed_headers("*")
                        .allowed_methods("PUT")
                        .allowed_origins("*")
                        .build(),
                )
                .build(),
        )
        .send()
        .await
        .unwrap();
    */
    let amqp_channel = web::Data::new(Arc::new(channel));
    tokio::spawn(async { pokerbots::games::listen_for_game_results(channel_b).await });
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
            .app_data(amqp_channel.clone())
            .app_data(s3_client.clone())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(session_middleware)
            .route("/api/login", web::get().to(login::handle_login))
            .service(login::login_provider)
            .service(api::manage_team::create_team)
            .service(api::manage_team::delete_team)
            .service(api::manage_team::leave_team)
            .service(api::manage_team::make_invite)
            .service(api::manage_team::upload_pfp)
            .service(api::manage_team::upload_bot)
            .service(api::manage_team::join_team)
            .service(api::manage_team::cancel_invite)
            .service(api::manage_team::kick_member)
            //.service(api::manage_team::bot_upload_url)
            .service(api::data::my_account)
            .service(api::data::server_message)
            .service(api::data::my_team)
            .service(api::data::teams)
            .service(api::data::bots)
            .service(api::games::make_game)
            .service(api::games::games)
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
    .workers(2)
    .bind((
        "0.0.0.0",
        std::env::var("PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(3000),
    ))?
    .run()
    .await
}
