use axum::{extract::Path, http::StatusCode};
use chrono::{DateTime, Utc};
use maud::{html, Markup};

use super::root::{error_page, MergedPage};

#[derive(Debug, Clone)]
pub struct Post {
    title: String,
    date: DateTime<Utc>,
    description: String,
    tags: Vec<String>,
    text: String,
}

include!(concat!(env!("OUT_DIR"), "/posts.rs"));

// TODO: add pagination
/*pub fn get_posts_html(count: usize) -> Markup {
    let mut posts: Vec<_> = POSTS.iter().collect();
    posts.sort_by(|a, b| b.1.date.cmp(&a.1.date));
    html! {
        div."posts" {
            ul {
                @for (slug, post) in posts.iter().take(count) {
                    li {
                        span {
                            (post.date.format("%Y-%m-%d")) ": " a href=(format!("/posts/{}", slug)) {
                                (post.title)
                            }
                            p."description" { (post.description) }
                        }
                    }
                }
            }
        }
    }
}*/

pub async fn post_full_list() -> Markup {
    MergedPage::new_content_and_meta("Posts".to_string(), "All the things I've written".to_string(), html! {
        (maud::PreEscaped(POST_LIST_HTML))
    }).render()
}

pub async fn serve_post_page(Path(slug): Path<String>) -> Result<Markup, (StatusCode, Markup)> {
    let post_num = slug.to_string();
    if !POSTS.contains_key(&slug) {
        return Err(error_page(StatusCode::NOT_FOUND, "Post not found :("))
    }

    let post = POSTS.get(&post_num).unwrap();
    Ok(MergedPage::new_content_and_meta(post.0.to_string(), post.1.to_string(), html! {
            (maud::PreEscaped(&post.2))
    }).render())
}