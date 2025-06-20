use std::env::{self, temp_dir};

use anyhow::Result;
use tracing::{info, warn};

use crate::util::build::{eightyeightthirtyone::{compress_badges, generate_badge_file}, image::{process_internal_images_to_file, Image, ImageCompressor}, posts::{apply_compression_to_post_photos, generate_post_map, get_posts_html}};

pub mod posts;
pub mod image;
pub mod eightyeightthirtyone;


pub fn build(db: &sled::Db) -> Result<()> {

    let out_dir = temp_dir();

    let compressor = ImageCompressor::new(&out_dir.to_string_lossy(), Ok("release".to_owned()) != env::var("PROFILE"));
    warn!("BUILD MODE IS NOT RELEASE: {}", Ok("release".to_owned()) != env::var("PROFILE"));
    
    let built_html: sled::Tree = db.open_tree(b"html")?;


    let converted_badges = compress_badges("content/88x31.csv", &compressor);
    let badge_data = generate_badge_file(converted_badges);

    // process the random images that have changed
    let mut generated_images: Vec<(Image, String)> = Vec::new();

    let converted_images = process_internal_images_to_file("content/internal_images.csv", &compressor);
    generated_images.extend(converted_images.0);

    let image_map_tree: sled::Tree = db.open_tree(b"image_map")?;
    for image in converted_images.1 {
        image_map_tree.insert(image.0.to_string(), image.1.as_bytes())?;
    }
    
    let posts_tree: sled::Tree = db.open_tree(b"posts")?;
    let mut posts = generate_post_map("./content/posts");

    for post in &mut posts {
        info!("Storing post \"{}\" to slug {}", post.title, post.slug);
        generated_images.extend(apply_compression_to_post_photos(post, &compressor));

        let serialized_post = bincode::serialize(&post)?;
        posts_tree.insert(post.slug.clone(), serialized_post)?;
    }

    let images_tree: sled::Tree = db.open_tree(b"images")?;
    for image in generated_images {
        let file_content = std::fs::read(&image.0.path)
            .unwrap_or_else(|e| panic!("Failed to read image file {}: {}", image.0.path, e));
        images_tree.insert(image.1.as_bytes(), file_content)?;
    }
    for badge in badge_data.2 {
        images_tree.insert(badge.0.as_bytes(), badge.1)?;
    }

    let mut post_sorted = posts.clone();
    post_sorted.sort_by(|a, b| b.date.cmp(&a.date));
    
    // now we can pre-cache some HTML
    built_html.insert(b"badge_html", badge_data.0.as_bytes())?;
    built_html.insert(b"post_index_html", get_posts_html(&post_sorted, 5).into_string().as_bytes())?;
    built_html.insert(b"post_list_html", get_posts_html(&post_sorted, post_sorted.len()).into_string().as_bytes())?;
    /*
    fs::write(format!("{out_dir}/internal_images.rs"), converted_images.1).unwrap();

    println!("cargo::rerun-if-changed=posts");
    println!("{out_dir}/badges.rs");

    let mut posts = generate_post_map("content/posts");
    for post in &mut posts {
        generated_images.extend(apply_compression_to_post_photos(post, &compressor));
    }

    fs::write(format!("{out_dir}/posts.rs"), generate_posts_file(posts)).unwrap();

    // now we need to save the images!

    fs::write(format!("{out_dir}/generated_images.rs"), zip_images_and_paths_to_file(generated_images)).unwrap();*/



    Ok(())
}