use std::{env, fs, process::Command};
mod eightyeightthirtyone;

use eightyeightthirtyone::{compress_badges, generate_badge_file};

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
    
    let badge_csv_content = std::fs::read_to_string("content/88x31.csv")
        .unwrap_or(String::new());

    let converted_badges = compress_badges(&out_dir, &badge_csv_content);
    fs::write(format!("{out_dir}/badges.rs"),  generate_badge_file(converted_badges)).unwrap();
}
