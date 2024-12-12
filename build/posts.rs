use std::collections::HashMap;

use chrono::{DateTime, Utc};
use gray_matter::{engine::YAML, Matter};
use markdown::{to_html_with_options, CompileOptions, Options};
use maud::{html, Markup};
use crate::image::{Image, ImageCompressor};

#[derive(Debug, Clone)]
pub struct Post {
    slug: String,
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

pub fn generate_post_map(directory: &str) -> Vec<Post> {
    let mut posts: Vec<Post> = Vec::new();
    for entry in std::fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().unwrap_or_default() == "md" {
            // some good help from https://github.com/haylinmoore/www/blob/main/src/words.rs
            let content = std::fs::read_to_string(&path).unwrap();

            let matter = Matter::<YAML>::new();
            let result = matter.parse(&content);

            let Some(result_map) = result.data.as_ref() else {
                panic!("Error parsing YAML");
            };
            let Ok(result_map) = result_map.as_hashmap() else {
                panic!("Error getting hashmap from Pod");
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
            
            posts.push(Post {
                slug,
                title,
                description,
                tags,
                date,
                text: md_to_html(result.content)
            });
        }
    }
    posts
}

pub fn apply_compression_to_post_photos(post: &mut Post, compressor: &ImageCompressor) -> (Vec<(Image, String)>) {
    let mut output = String::new();
    let mut post_images: Vec<(Image, String)> = Vec::new();
    for line in post.text.lines() {
        if line.starts_with("<p><img src=\"") {
            let mut img_path = line.split("\"").nth(1).unwrap().to_string();
            if !img_path.starts_with("http") {
                img_path = format!(".{}", img_path);
            }
            // TODO: just use the uncompressed if failed
            let compressed_images = compressor.compress_lossy(img_path.as_str()).expect("Could not encode images");
            let mut image_map = HashMap::new();
            for image in &compressed_images {
                if !image_map.contains_key(&image.codec) {
                    image_map.insert(image.codec.clone(), vec![]);
                }
                let image_path = format!("/generated/{}", image.path.split("/").last().expect("Image doesn't exist?"));
                let asset = image_map.get_mut(&image.codec).unwrap();
                asset.push(format!("{} {}wh", image_path, image.width));
                post_images.push((image.clone(), image_path));
            }
            output += format!("<picture><source media=\"(min-width: 600px)\" sizes=\"600px\" srcset=\"{}\"><img sizes=\"600px\" srcset=\"{}\" src=\"{}\"></picture>\n",
                              image_map.get("jxl").expect("No JXL!").join(", "),
                              image_map.get("webp").expect("No WEBP!").join(", "),
                              image_map.get("webp").expect("No WEBP!").get(0)
                                  .expect("No webp 0?").split_once(" ").expect("Malformed compression").0
            ).as_str();

        } else {
            output += format!("{}\n", line).as_str();
        }
    }
    post.text = output;

    post_images
}

pub fn get_posts_html(mut posts: Vec<Post>) -> Markup {
    let count = 5;
    posts.sort_by(|a, b| b.date.cmp(&a.date));
    html! {
        div."posts" {
            ul {
                @for post in posts.iter().take(count) {
                    li {
                        span {
                            (post.date.format("%Y-%m-%d")) ": " a href=(format!("/posts/{}", post.slug)) {
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


pub fn generate_posts_file(posts: Vec<Post>) -> String {
    let mut builder = phf_codegen::Map::new();

    for post in &posts {
        let html = html! {
            h1 { (post.title) }
            p."post-date" { (post.date.format("%Y-%m-%d")) }
            p."description" { (post.description) }
            div."post-content" {
                (maud::PreEscaped(&post.text))
            }
        };
        builder.entry(post.slug.clone(), format!("(\"{}\", \"{}\", \"{}\")", post.title, post.description, html.into_string().replace("\"", "\\\"")).as_str());
    }

    let output = format!("// This file was auto generated, do not modify!

pub static POST_HTML: &str = \"{}\";

// title, description, content

static POSTS: phf::Map<&'static str, (&str, &str, &str)> = {};",
                         get_posts_html(posts).into_string().replace("\"", "\\\""),
                         builder.build()
    );
    return output;
}