use std::time::Duration;

use axum::{
     body::Body, extract::{Request, State}, handler::HandlerWithoutStateExt, http::{Response, StatusCode}, routing::get, Router
};
use maud::Markup;
use paths::{about::index, posts::{post_full_list, serve_post_page}, root::{error_page, error_page_file}};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{info, Level, Span};
use crate::{paths::{root::serve_generated_image}, util::state::SiteState};
use tracing_subscriber::EnvFilter;
use clap::Parser;

mod paths;
mod util;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = false)]
    build: bool
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber_level = match std::env::var("LOGLEVEL")
        .unwrap_or_else(|_| "INFO".to_string())
        .to_ascii_uppercase()
        .as_str() 
    {
        "TRACE" => Level::TRACE,
        "DEBUG" => Level::DEBUG,
        "INFO" => Level::INFO,
        "WARN" => Level::WARN,
        "ERROR" => Level::ERROR,
        _ => Level::INFO, // default if the environment variable is not set or invalid
    };

    tracing_subscriber::fmt()
        .with_max_level(subscriber_level)
        .with_env_filter(EnvFilter::new("holligerme"))
        .init();


    let args = Cli::parse();



    let state = SiteState::new("database", args.build)?;

    if args.build {
        info!("Build complete. Exiting");
        return Ok(())
    }

    let not_found = error_page(StatusCode::NOT_FOUND, "", state.clone());
    let not_found_file_service = error_page_file(StatusCode::NOT_FOUND, "", state.clone()).into_service();
    
    let app = Router::new()
        .route("/", get(root))
        .route("/about", get(index))
        .route("/posts/:slug", get(serve_post_page))
        .route("/posts/:slug/", get(serve_post_page))
        .route("/posts", get(post_full_list))
        .route("/posts/", get(post_full_list))
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
            }))
            .with_state(state);

    info!("Loaded! Listening...");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn root(State(state): State<SiteState>) -> Markup {
    paths::root::index(state)
}