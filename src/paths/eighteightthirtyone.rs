use phf;
use axum::{extract::Path, http::StatusCode, response::Response};
use maud::Markup;

use super::root::error_page;

include!(concat!(env!("OUT_DIR"), "/badges.rs"));


pub async fn serve_88x31(Path(image): Path<String>) -> Result<Response, (StatusCode, Markup)> {
    let image_name: String = image.to_string();
    let ext = image_name.split_once(".").ok_or_else(|| {
        error_page(StatusCode::BAD_REQUEST, "Invalid image name format")
    })?;

    if !BADGE_DATA.contains_key(&image) {
        return Err(error_page(StatusCode::NOT_FOUND, "88x31 not found :("))
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", format!("image/{}", ext.1))
        .body(BADGE_DATA[&image].to_vec().into())
        .unwrap())
}