use axum::{
    routing::get,
    http::StatusCode,
    Router,
};
use maud::Markup;
use paths::{posts::serve_post_page, root::error_page};
mod paths;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();
    
    let app = Router::new()
        .route("/", get(root))
        .route("/posts/:slug", get(serve_post_page))
        .route("/posts/:slug/", get(serve_post_page))
        .nest_service("/assets", tower_http::services::ServeDir::new("assets"))
        .nest_service("/artifacts", tower_http::services::ServeDir::new("artifacts"))
        .fallback(error_page(StatusCode::NOT_FOUND, ""));

    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Markup {
    paths::root::index()
}