use std::{env, fs, process::Command};

use maud::html;

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

struct Badge {
    name: String,
    url: String,
    paths: Vec<String>
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
                    converted_badges.push((badge.0, badge.1, vec![path_to_compress]));
                    continue;
                }
            }
            path_to_compress = cached_path;
        }

        // need to allow cross-conversion, webp doesnt like jxl and jxl doesnt like webp
        // TODO: support animated webp during this part of the build phase, currently webp must become gif
        if path_to_compress.ends_with("webp") || path_to_compress.ends_with("jxl") {
            let op = path_to_compress.clone();
            path_to_compress = format!("{}.png", path_to_compress);
            if file_violates_cache_rules(&path_to_compress) || cache_violated { // check cache_violated as the png would exist but be expired
                cache_violated = true;
                println!("{} is in a format that cannot be compressed easily! Converting to PNG to re-compress...", badge.0);
                let cmpr = run_command_nicely(Command::new("convert").arg(&op).arg(&path_to_compress));
                if cmpr.0 != 0 {
                    eprintln!("Could not convert webp image... serving as-is for {}", badge.0);
                    converted_badges.push((badge.0, badge.1, vec![op]));
                    continue;
                }
            }
        }
        let mut valid_save_paths: Vec<String> = Vec::new();


        // if gif, just serve the gif
        if path_to_compress.ends_with("gif") {
            println!("Serving GIF image for {}", badge.0);
            // attempt to compress

            let save_path = format!("{out_dir}/artifacts/88x31/published/{}.webp", badge.0);
            if file_violates_cache_rules(&save_path) || cache_violated {
                // https://github.com/gianni-rosato/minify/blob/129027ccc0ac134d05ad748b7938d255158750ac/minify.sh#L62
                let cjxl = run_command_nicely(Command::new("gif2webp").arg("-q").arg("100").arg("-m").arg("6").arg("-metadata").arg("icc").arg(&path_to_compress).arg("-o").arg(&save_path));
                if cjxl.0 == 0 {
                    println!("Cached and optimized {} for Animated WEBP", badge.0);
                    valid_save_paths.push(save_path);
                } else {
                    eprintln!("Animated WEBP compression failed for {}", cjxl.1);
                }
            } else {
                // already compressed!
                println!("Serving cached and optimized file for {}", badge.0);
                valid_save_paths.push(save_path);
            }

            valid_save_paths.push(path_to_compress);

            converted_badges.push((badge.0, badge.1, valid_save_paths));
            continue;
        }


        // now we can try to compress it
        let save_path: String = format!("{out_dir}/artifacts/88x31/published/{}.jxl", badge.0);
        if file_violates_cache_rules(&save_path) || cache_violated {
            let cjxl = run_command_nicely(Command::new("cjxl").arg("-d").arg("0").arg("-e").arg("10").arg(&path_to_compress).arg(&save_path));
            if cjxl.0 == 0 {
                println!("Cached and optimized {} for JXL", badge.0);
                valid_save_paths.push(save_path);
            } else {
                eprintln!("JXL compression failed for {}", cjxl.1);
            }
        } else {
            // already compressed!
            println!("Serving cached and optimized file for {}", badge.0);
            valid_save_paths.push(save_path);
        }


        // fallback webp
        let save_path = format!("{out_dir}/artifacts/88x31/published/{}.webp", badge.0);
        if file_violates_cache_rules(&save_path) || cache_violated {
            // https://github.com/gianni-rosato/minify/blob/129027ccc0ac134d05ad748b7938d255158750ac/minify.sh#L62
            let cjxl = run_command_nicely(Command::new("cwebp").arg("-mt").arg("-lossless").arg("-z").arg("9").arg("-alpha_filter").arg("best").arg("-metadata").arg("icc").arg(&path_to_compress).arg("-o").arg(&save_path));
            if cjxl.0 == 0 {
                println!("Cached and optimized {} for JXL", badge.0);
                valid_save_paths.push(save_path);
                //converted_badges.push((badge.0, badge.1, save_path));
            } else {
                eprintln!("JXL compression failed for {}", cjxl.1);
            }
            //converted_badges.push((badge.0, badge.1, path_to_compress));
        } else {
            // already compressed!
            println!("Serving cached and optimized file for {}", badge.0);
            //converted_badges.push((badge.0, badge.1, save_path));
            valid_save_paths.push(save_path);
        }

        // done, now we can see if anything was compressed
        if valid_save_paths.len() == 0 {
            converted_badges.push((badge.0, badge.1, vec![path_to_compress]));
        } else {
            converted_badges.push((badge.0, badge.1, valid_save_paths));
        }
    }

    println!("{:?}", converted_badges);

    let mut badges = Vec::new();
    let mut builder = phf_codegen::Map::new();

    for (name, url, images) in &converted_badges {
        let mut image_paths = Vec::new();
        for image in images {
            match fs::read(image) {
                Ok(contents) => {
                    let ext = image.split("/").last().unwrap().split(".").last().unwrap();
                    let im_path = format!("{name}.{ext}");
                    image_paths.push(im_path.clone());
                    builder.entry(im_path, format!("&{:?}", contents).as_str());
                },
                Err(e) => {
                    panic!("Failed to read badge file {}: {}", image, e);
                }
            }
        }

        badges.push(Badge {
            name: name.to_string(),
            url: url.to_string(),
            paths: image_paths
        });
    }

    let badge_build = html! {
        div."badges" {
            @for (_, badge) in badges.iter().enumerate() {
                a href=(badge.url) target="_blank" {
                    picture class="eightyeightthirtyone" {
                        @let urls = &badge.paths;
                        @for (i, url) in urls.iter().enumerate() {
                            @if i < urls.len() - 1 {
                                source alt=(badge.name) srcset=(format!("/88x31/{}", url)) type=(format!("image/{}", url.split_once(".").unwrap().1));
                            } @else {
                                img alt=(badge.name) src=(format!("/88x31/{}", url));
                            }
                        }
                    }
                }
            }
        }
    };
    let output = format!("// This file was auto generated, do not modify!

pub const BADGE_HTML: &str = \"{}\";

static BADGE_DATA: phf::Map<&'static str, &[u8]> = {};",
        badge_build.into_string().replace("\"", "\\\""),
        builder.build()
    );

    fs::write(format!("{out_dir}/badges.rs"), output).unwrap();
}
