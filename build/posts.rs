use chrono::{DateTime, Utc};
use gray_matter::{engine::YAML, Matter};
use markdown::{to_html_with_options, CompileOptions, Options};
use maud::{html, Markup};
use crate::image::{convert_image_list_to_html_element_and_map, Image, ImageCompressor};

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
            println!("Found post: {}", path.display());
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
            
            let html_text = md_to_html(result.content);
            let mut collected_html_text = String::new();
            let opts = katex::Opts::builder().output_type(katex::OutputType::Mathml).build().unwrap();
            println!("HTML Text: {}", html_text);
            for line in html_text.lines() {
                let mut built_line = String::new();
                let mut equations: Vec<(usize, usize)> = Vec::new(); // starting position, length
                // we need to go step by step to find all equations present
                let mut curr_equation_active = false;
                let mut starting_pos: usize = 0;
                let line_content = line.as_bytes();
                for char_pos in line.chars().enumerate() { // this will have issues when its the first or last character, although due to how things render this shouldnt happen
                    if line_content[char_pos.0] == b'$' {
                        if curr_equation_active && line_content[char_pos.0-1] != b' ' && line_content[char_pos.0-1] != b'\\' {
                            // end of an equation!
                            // we need to add the equation to our list of equations
                            equations.push((starting_pos, char_pos.0 - starting_pos + 1));
                            curr_equation_active = false;
                        } else if !curr_equation_active && line_content[char_pos.0+1] != b' ' && line_content[char_pos.0-1] != b'\\' {
                            // start of an equation!
                            curr_equation_active = true;
                            starting_pos = char_pos.0;
                        }
                    }
                }
                println!("{:?}", equations);
                let mut current_pos = 0;
                for equation in equations {
                    built_line += &String::from_utf8_lossy(&line_content[current_pos..equation.0]);
                    let equation_text = &line[equation.0+1..equation.0+equation.1-1];
                    let rendered = katex::render_with_opts(equation_text, &opts).unwrap();
                    built_line += &rendered;
                    current_pos = equation.0 + equation.1;
                }
                built_line += &String::from_utf8_lossy(&line_content[current_pos..]);
                collected_html_text += format!("{}\n", built_line).as_str();
                println!("Built line: {}", built_line);
            }

            posts.push(Post {
                slug,
                title,
                description,
                tags,
                date,
                text: collected_html_text
            });
        }
    }
    posts
}

pub fn apply_compression_to_post_photos(post: &mut Post, compressor: &ImageCompressor) -> Vec<(Image, String)> {
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
            let conv_output = convert_image_list_to_html_element_and_map(compressed_images, Some("(min-width: 740px) 600px"));
            post_images.extend(conv_output.0);
            output += conv_output.1.as_str();
        } else {
            output += format!("{}\n", line).as_str();
        }
    }
    post.text = output;

    post_images
}

pub fn get_posts_html(posts: &Vec<Post>, count: usize) -> Markup {
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
        builder.entry(post.slug.clone(), format!("(\"{}\", \"{}\", \"{}\")", post.title, post.description, html.into_string().replace("\\", "\\\\").replace("\"", "\\\"")).as_str());
    }

    let mut post_sorted = posts.clone();
    post_sorted.sort_by(|a, b| b.date.cmp(&a.date));

    let output = format!("// This file was auto generated, do not modify!

pub static POST_INDEX_HTML: &str = \"{}\";

pub static POST_LIST_HTML: &str = \"{}\";


// title, description, content

static POSTS: phf::Map<&'static str, (&str, &str, &str)> = {};",
                         get_posts_html(&post_sorted, 5).into_string().replace("\"", "\\\""),
                         get_posts_html(&post_sorted, 100).into_string().replace("\"", "\\\""), // this number would need to be increased
                         builder.build()
    );
    return output;
}