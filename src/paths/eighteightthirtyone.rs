use std::collections::HashMap;

use axum::{extract::Path, http::StatusCode, response::Response};
use lazy_static::lazy_static;
use maud::{html, Markup};

use super::root::error_page;

include!(concat!(env!("OUT_DIR"), "/badges.rs"));
include!(concat!(env!("OUT_DIR"), "/badge_content.rs"));


lazy_static! {
    static ref BADGES: Markup = {
        // now we can generate the HTML
        html! {
            div."badges" {
                @for (index, badge) in BUILD_BADGES.iter().enumerate() {
                    a href=(badge.1) target="_blank" {
                        img alt=(badge.0) src=(format!("/88x31/{}", badge.2)) class="eightyeightthirtyone";
                    }
                    @if (index + 1) % 5 == 0 {
                        br;
                    }
                }
            }
        }
    };

    static ref BADGE_HASH_MAPPING: HashMap<String, i32> = {
        let mut mapping = HashMap::new();
        for bmap in BADGE_MAPPING {
            mapping.insert(bmap.0.to_string(), bmap.1);
        }
        mapping
    };
}

pub fn badges() -> Markup {
    BADGES.clone()
}

pub async fn serve_88x31(Path(image): Path<String>) -> Result<Response, (StatusCode, Markup)> {
    let image_name: String = image.to_string();
    let ext = image_name.split_once(".").ok_or_else(|| {
        error_page(StatusCode::BAD_REQUEST, "Invalid image name format")
    })?;

    if !BADGE_HASH_MAPPING.contains_key(&image_name) {
        return Err(error_page(StatusCode::NOT_FOUND, "88x31 not found :("))
    }
    
    let mapping = BADGE_HASH_MAPPING.get(&image_name).unwrap();
    let content = BADGE_CONTENTS[*mapping as usize];

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", format!("image/{}", ext.1))
        .body(content.to_vec().into())
        .unwrap())
}