use std::{env, fs, process::Command};
mod eightyeightthirtyone;
mod posts;
mod image;

use eightyeightthirtyone::{compress_badges, generate_badge_file};
use posts::generate_post_map;
use crate::image::{process_internal_images_to_file, zip_images_and_paths_to_file, Image, ImageCompressor};
use crate::posts::{apply_compression_to_post_photos, generate_posts_file};

fn main() {
    // note: add error checking yourself.
    let git_hash = String::from_utf8(Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output().unwrap().stdout).unwrap();
    let git_time = String::from_utf8(Command::new("git").args(&["show", "--no-patch", "--format=%ct", "HEAD"]).output().unwrap().stdout).unwrap();
    let git_message = String::from_utf8(Command::new("git").args(&["show", "--no-patch", "--format=%B", "HEAD"]).output().unwrap().stdout).unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    println!("cargo:rustc-env=GIT_TIME={}", git_time);
    println!("cargo:rustc-env=GIT_MESSAGE={}", git_message.trim());

    // now we can get into 88x31 caching
    println!("cargo::rerun-if-changed=content/88x31.csv");

    println!("BUILD MODE IS NOT RELEASE: {}", Ok("release".to_owned()) != env::var("PROFILE"));

    let compressor = ImageCompressor::new(out_dir.as_str(), Ok("release".to_owned()) != env::var("PROFILE"));

    let converted_badges = compress_badges("content/88x31.csv", &compressor);
    fs::write(format!("{out_dir}/badges.rs"),  generate_badge_file(converted_badges)).unwrap();


    // process the random images that have changed
    let mut generated_images: Vec<(Image, String)> = Vec::new();

    let converted_images = process_internal_images_to_file("content/internal_images.csv", &compressor);
    generated_images.extend(converted_images.0);

    fs::write(format!("{out_dir}/internal_images.rs"), converted_images.1).unwrap();

    println!("cargo::rerun-if-changed=posts");
    println!("{out_dir}/badges.rs");

    let mut posts = generate_post_map("content/posts");
    for post in &mut posts {
        generated_images.extend(apply_compression_to_post_photos(post, &compressor));
    }

    fs::write(format!("{out_dir}/posts.rs"), generate_posts_file(posts)).unwrap();

    // now we need to save the images!

    fs::write(format!("{out_dir}/generated_images.rs"), zip_images_and_paths_to_file(generated_images)).unwrap();
}
