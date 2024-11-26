use axum::{
    handler::HandlerWithoutStateExt, http::StatusCode, routing::get, Router
};
use maud::Markup;
use paths::{about::index, eighteightthirtyone::serve_88x31, posts::serve_post_page, root::{error_page, error_page_file}};
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
        .nest_service("/assets", tower_http::services::ServeDir::new("assets").not_found_service(not_found_file_service.clone())) // can i get away with not cloning?
        .nest_service("/artifacts", tower_http::services::ServeDir::new("artifacts").not_found_service(not_found_file_service.clone()))
        .nest_service("/pub", tower_http::services::ServeDir::new("pub").not_found_service(not_found_file_service.clone()))
        .fallback(not_found);

    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Markup {
    paths::root::index()
}