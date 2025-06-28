use axum::{extract::{Path, State}, http::StatusCode};
use chrono::{DateTime, Utc};
use maud::{html, Markup};
use serde::{Deserialize, Serialize};

use crate::util::state::SiteState;

use super::root::{error_page, MergedPage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub slug: String,
    pub title: String,
    pub date: DateTime<Utc>,
    pub description: String,
    pub tags: Vec<String>,
    pub text: String,
}

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

pub async fn post_full_list(State(state): State<SiteState>) -> Markup {
    MergedPage::new_content_and_meta("Posts", "All the things I've written", html! {
        (maud::PreEscaped(state.get_cached_html_element("post_list_html")))
    }, state).render()
}

pub async fn serve_post_page(State(state): State<SiteState>, Path(slug): Path<String>) -> Result<Markup, (StatusCode, Markup)> {
    let post = state.get_post(&slug);
    match post {
        Ok(post) => {
            Ok(MergedPage::new_content_and_meta(&post.title, &post.description, html! {
                h1 { (&post.title) }
                hr;
                (maud::PreEscaped(&post.text))
            }, state).render())
    },
        Err(_) => return Err(error_page(StatusCode::NOT_FOUND, "Post not found :(", state))
    }
}