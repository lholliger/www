use std::time::Duration;

use axum::{
     body::Body, extract::Request, handler::HandlerWithoutStateExt, http::{Response, StatusCode}, routing::get, Router
};
use maud::Markup;
use paths::{about::index, eighteightthirtyone::serve_88x31, posts::serve_post_page, root::{error_page, error_page_file}};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{info, Span};
use crate::paths::root::serve_generated_image;

mod paths;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let not_found = error_page(StatusCode::NOT_FOUND, "");
    let not_found_file_service = error_page_file(StatusCode::NOT_FOUND, "").into_service();
    
    let app = Router::new()
        .route("/", get(root))
        .route("/about", get(index))
        .route("/posts/:slug", get(serve_post_page))
        .route("/posts/:slug/", get(serve_post_page))
        .route("/88x31/:image", get(serve_88x31))
        .route("/generated/:image", get(serve_generated_image))
        .nest_service(
            "/assets/css",
            ServeDir::new("assets/css")
                .not_found_service(ServeDir::new("assets").fallback(not_found_file_service.clone()))
        )
        .nest_service(
            "/pub", 
            ServeDir::new("pub")
                .not_found_service(ServeDir::new("pub").fallback(not_found_file_service.clone()))
        )
        .fallback(not_found)
        .layer(TraceLayer::new_for_http()
            .make_span_with(|request: &Request<Body>| {
                tracing::info_span!(
                    "request",
                    method = %request.method(),
                    uri = %request.uri(),
                    version = ?request.version(),
                )
            })
            .on_response(|response: &Response<Body>, latency: Duration, _span: &Span| {
                tracing::info!(
                    status = response.status().as_u16(),
                    latency = ?latency,
                    "response"
                );
            }));

    info!("Loaded! Listening...");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Markup {
    paths::root::index()
}