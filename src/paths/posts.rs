use std::collections::HashMap;

use axum::{extract::Path, http::StatusCode};
use chrono::{DateTime, Utc};
use gray_matter::{engine::YAML, Matter};
use lazy_static::lazy_static;
use maud::{html, Markup};
use markdown::{to_html_with_options, CompileOptions, Options};
use tracing::{error, info};

use super::root::{error_page, MergedPage};

#[derive(Debug, Clone)]
pub struct Post {
    title: String,
    date: DateTime<Utc>,
    description: String,
    tags: Vec<String>,
    text: String,
}


pub fn md_to_html(md: String) -> String {
    let result = to_html_with_options(md.as_str(), &Options {
        compile: CompileOptions {
        allow_dangerous_html: true,
        allow_dangerous_protocol: true,
        ..CompileOptions::default()
        },
        ..Options::default()
    });
    result.unwrap()
}

// TODO: dont unwrap so much!
lazy_static! {
    static ref POSTS: HashMap<String, Post> = {
        info!("Generating posts map!");
        let mut posts: HashMap<String, Post> = HashMap::new();
        for entry in std::fs::read_dir("content/posts").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().unwrap_or_default() == "md" {
                // some good help from https://github.com/haylinmoore/www/blob/main/src/words.rs
                let content = std::fs::read_to_string(&path).unwrap();

                let matter = Matter::<YAML>::new();
                let result = matter.parse(&content);

                let Some(result_map) = result.data.as_ref() else {
                    error!("Error parsing YAML");
                    continue;
                };
                let Ok(result_map) = result_map.as_hashmap() else {
                    error!("Error getting hashmap from Pod");
                    continue;
                };

                let title = result_map["title"].as_string().unwrap();
                let description = result_map["description"].as_string().unwrap();

                let mut tags: Vec<String> = Vec::new();
                if result_map.contains_key("tags") {
                    let taglist = result_map["tags"].as_vec().unwrap();
                    for tag in taglist {
                        tags.push(tag.as_string().unwrap());
                    }
                }

                let date_str = result_map["date"].as_string().unwrap();
                let date = if date_str.contains('T') {
                    // If the date string already includes time
                    DateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%SZ")
                        .unwrap_or_else(|_| panic!("Invalid date format: {}", date_str))
                        .with_timezone(&Utc)
                } else {
                    // If the date string is just a date (YYYY-MM-DD)
                    DateTime::parse_from_str(&format!("{}T00:00:00+00:00", date_str), "%Y-%m-%dT%H:%M:%S%:z")
                        .unwrap_or_else(|_| panic!("Invalid date format: {}", date_str))
                        .with_timezone(&Utc)
                };

                let slug = path.file_stem().unwrap().to_string_lossy().to_string();
                
                info!("Loaded post \"{}\"", title);
                posts.insert(slug, Post {
                    title,
                    description,
                    tags,
                    date,
                    text: md_to_html(result.content)
                });
            }
        }
        posts

    };
}

pub fn get_posts_html(count: usize) -> Markup {
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
}

pub async fn serve_post_page(Path(slug): Path<String>) -> Result<Markup, (StatusCode, Markup)> {
    let post_num = slug.to_string();
    if !POSTS.contains_key(&slug) {
        return Err(error_page(StatusCode::NOT_FOUND, "Post not found :("))
    }

    let post = POSTS.get(&post_num).unwrap();
    Ok(MergedPage::new_content_and_meta(post.title.clone(), post.description.clone(), html! {
        h1 { (post.title) }
        p."post-date" { (post.date.format("%Y-%m-%d")) }
        p."description" { (post.description) }
        div."post-content" {
            (maud::PreEscaped(&post.text))
        }
    }).render())
}