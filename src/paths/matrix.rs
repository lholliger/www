use axum::{extract::State, routing::get, Json, Router};
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct MatrixConfig {
    pub homeserver_url: String,
}

pub fn matrix_router(config: MatrixConfig) -> Router {
    Router::new()
        .route("/.well-known/matrix/client", get(well_known_client))
        .route("/.well-known/matrix/server", get(well_known_server))
        .with_state(config)
        .layer(CorsLayer::permissive())
}

async fn well_known_client(State(config): State<MatrixConfig>) -> Json<Value> {
    Json(json!({
        "m.homeserver": { "base_url": config.homeserver_url }
    }))
}

async fn well_known_server(State(config): State<MatrixConfig>) -> Json<Value> {
    let federation_host = config
        .homeserver_url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    Json(json!({
        "m.server": format!("{}:443", federation_host)
    }))
}
