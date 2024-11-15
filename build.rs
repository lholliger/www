use core::num;
use std::{env, fs, process::Command};

fn run_command_nicely(command: &mut Command) -> (i32, String) {
    let output = command.output();
    match output {
        Ok(output) => match output.status.code().unwrap_or(256) {
            0 => {
                (0, String::from_utf8_lossy(&output.stdout).to_string())
            },
            code => {
                (code, String::from_utf8_lossy(&output.stderr).to_string())
            }
            
        },
        Err(e)  => (
            e.raw_os_error().unwrap(),
            format!("{:?}", e),
        ),
    }
}

// makes sure the file is "expired" or does not exist
fn file_violates_cache_rules(path: &String) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        if let Ok(modified) = metadata.modified() {
            let age = modified.elapsed().unwrap_or_default();
            if age.as_secs() < 60*60*24*7 {
                return false;
            }
        }
    }
    return true;
}

fn main() {
    // note: add error checking yourself.
    let git_hash = String::from_utf8(Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output().unwrap().stdout).unwrap();
    let git_time = String::from_utf8(Command::new("git").args(&["show", "--no-patch", "--format=%ct", "HEAD"]).output().unwrap().stdout).unwrap();
    let git_message = String::from_utf8(Command::new("git").args(&["show", "--no-patch", "--format=%B", "HEAD"]).output().unwrap().stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    println!("cargo:rustc-env=GIT_TIME={}", git_time);
    println!("cargo:rustc-env=GIT_MESSAGE={}", git_message.trim());

    // now we can get into 88x31 caching
    println!("cargo::rerun-if-changed=content/88x31.csv");
    let file = std::fs::read_to_string("content/88x31.csv")
    .unwrap_or(String::new());
    let lines = file.lines();
    let mut badges = Vec::new();
    for line in lines {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() >= 3 {
            let name = fields[0].to_string();
            let url = fields[1].to_string();
            let image = fields[2].to_string();
            badges.push((name, url, image));
        }
    }
    println!("{:?}", badges);
    let out_dir = env::var("OUT_DIR").unwrap();
    fs::create_dir_all(format!("{out_dir}/artifacts/88x31/published")).unwrap();
    fs::create_dir_all(format!("{out_dir}/artifacts/88x31/cached")).unwrap();
    println!("{:?}", out_dir);
    // now lets not do the whole hotlinking thing
    let mut converted_badges = Vec::new();
    for badge in badges {
        let mut cache_violated = false;
        let sum = format!("{:x}", crc32fast::hash(badge.1.as_bytes()));
        let mut path_to_compress = badge.2;
        if path_to_compress.starts_with("http") {
            let ext = path_to_compress.split('.').last().unwrap(); // TODO: dont unwrap but make it work
            let cached_path = format!("{out_dir}/artifacts/88x31/cached/{sum}.{ext}");
            
            if file_violates_cache_rules(&cached_path) {
                cache_violated = true;
                println!("{} is not already cached! Downloading...", badge.0);
                let curl = run_command_nicely(Command::new("curl").arg(&path_to_compress).arg("-o").arg(&cached_path));
                if curl.0 != 0 { // the command failed
                    eprintln!("Failed to download the badge! Will ned to serve as-is for now! (Error: {:?}", curl.1);
                    converted_badges.push((badge.0, badge.1, path_to_compress));
                    continue;
                }
            }
            path_to_compress = cached_path;
        }

        // webp does badly for some reason
        if path_to_compress.ends_with("webp") {
            let op = path_to_compress.clone();
            path_to_compress = format!("{}.png", path_to_compress);
            if file_violates_cache_rules(&path_to_compress) || cache_violated { // check cache_violated as the png would exist but be expired
                cache_violated = true;
                println!("{} is not webp-converted! Converting...", badge.0);
                let cmpr = run_command_nicely(Command::new("convert").arg(&op).arg(&path_to_compress));
                if cmpr.0 != 0 {
                    eprintln!("Could not convert webp image... serving as-is for {}", badge.0);
                    converted_badges.push((badge.0, badge.1, op));
                    continue;
                }
            }
        }

        // if gif, just serve the gif
        if path_to_compress.ends_with("gif") {
            println!("Serving GIF image for {}", badge.0);
            converted_badges.push((badge.0, badge.1, path_to_compress));
            continue;
        }

        // now we can try to compress it
        let save_path = format!("{out_dir}/artifacts/88x31/published/{}.jxl", badge.0);
        // compress!
        if file_violates_cache_rules(&save_path) || cache_violated {
            let cjxl = run_command_nicely(Command::new("cjxl").arg("-d").arg("0").arg("-e").arg("10").arg(&path_to_compress).arg(&save_path));
            if cjxl.0 == 0 {
                println!("Cached and optimized {}", badge.0);
                converted_badges.push((badge.0, badge.1, save_path));
                continue;
            }
            eprintln!("JXL compression failed. Serving the uncompressed version: {}", cjxl.1);
            converted_badges.push((badge.0, badge.1, path_to_compress));
        } else {
            // already compressed!
            println!("Serving cached and optimized file for {}", badge.0);
            converted_badges.push((badge.0, badge.1, save_path));
        }
    }

    println!("{:?}", converted_badges);

    // now we need to figure out how to cache this
    let mut badge_strings = Vec::new();
    let mut number_mappings = Vec::new();
    let mut count = 0;
    for (name, url, image) in &converted_badges {
        let ext = image.split("/").last().unwrap().split(".").last().unwrap();
        let im_path = format!("{name}.{ext}");
        badge_strings.push(format!("(\"{}\", \"{}\", \"{}\")", name, url, im_path));
        number_mappings.push((im_path, count));
        count += 1;
    }
    // BADGE_MAPPING exists for the lazy_static HashMap to be made... there should be be a better solution than this
    let output = format!("const BUILD_BADGES: [(&str, &str, &str); {}] = [{}];\nconst BADGE_MAPPING: [(&str, i32); {}] = [{}];", 
        badge_strings.len(),
        badge_strings.join(", "),
        number_mappings.len(),
        number_mappings.iter()
            .map(|(s, n)| format!("(\"{}\", {})", s, n))
            .collect::<Vec<String>>()
            .join(", ")
    );

    fs::write(format!("{out_dir}/badges.rs"), output).unwrap();

    let mut badge_contents = Vec::new();
    for (_, _, image) in &converted_badges {
        match fs::read(image) {
            Ok(contents) => {
                badge_contents.push(format!("&{:?}", contents));
            },
            Err(e) => {
                panic!("Failed to read badge file {}: {}", image, e);
            }
        }
    }
    
    // Create the array declaration with the contents
    let output = format!("const BADGE_CONTENTS: &[&[u8]; {}] = &[{}];",
        badge_contents.len(),
        badge_contents.join(", ")
    );

    fs::write(format!("{out_dir}/badge_content.rs"), output).unwrap(); 
}
