use actix_session::SessionExt;
use actix_web::HttpMessage;

use {
    crate::app::login,
    actix_session::Session,
    actix_web::{
        body::BoxBody,
        dev::ServiceResponse,
        get,
        http::header::ContentType,
        middleware::{ErrorHandlerResponse, ErrorHandlers},
        routes, web, HttpResponse, Responder,
    },
    maud::{html, Markup, DOCTYPE},
};

pub mod common;
pub mod homepage;
pub fn service() -> actix_web::Scope {
    actix_web::web::scope("")
        .service(homepage::homepage_route)
        .service(web::scope("").wrap(error_handlers()))
}

// Custom error handlers, to return HTML responses when an error occurs.
fn error_handlers() -> ErrorHandlers<BoxBody> {
    ErrorHandlers::new().handler(actix_web::http::StatusCode::NOT_FOUND, not_found)
}

// Error handler for a 404 Page not found error.
fn not_found<B>(res: ServiceResponse<B>) -> actix_web::Result<ErrorHandlerResponse<BoxBody>> {
    let response = get_error_response(&res, "Page not found");
    Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
        res.into_parts().0,
        response.map_into_left_body(),
    )))
}

// Generic error handler.
fn get_error_response<B>(res: &ServiceResponse<B>, error: &str) -> HttpResponse {
    let request = res.request();
    let session: Session = request.get_session();

    common::base_page(
        html! {
            div class="background-graphic not-found-graphic" {};
            div class="page-container" {
                h1 class="headline" { "Error" }
                p { (error) }
            }
        },
        login::get_user_data(&session),
        common::nav::NavLocation::HomePage,
    )
    .respond_to(request)
    .map_into_boxed_body()
}
