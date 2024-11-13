use lazy_static::lazy_static;
use maud::{html, Markup};
use std::fs;
use std::process::Command;

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

lazy_static! {
    static ref BADGES: Markup = {
        println!("Generating 88x31 content!");
        fs::create_dir_all("artifacts/88x31/published").unwrap();
        fs::create_dir_all("artifacts/88x31/cached").unwrap();
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
        // now lets not do the whole hotlinking thing
        let mut converted_badges = Vec::new();
        for badge in badges {
            let sum = format!("{:x}", crc32fast::hash(badge.1.as_bytes()));
            let mut path_to_compress = badge.2;
            if path_to_compress.starts_with("http") {
                let ext = path_to_compress.split('.').last().unwrap(); // TODO: dont unwrap but make it work
                let cached_path = format!("artifacts/88x31/cached/{sum}.{ext}");
                let curl = run_command_nicely(Command::new("curl").arg(&path_to_compress).arg("-o").arg(&cached_path));
                if curl.0 != 0 { // the command failed
                    eprintln!("Failed to download the badge! Will ned to serve as-is for now! (Error: {:?}", curl.1);
                    converted_badges.push((badge.0, badge.1, path_to_compress));
                    continue;
                }
                path_to_compress = cached_path;
            }

            // webp does badly for some reason
            if path_to_compress.ends_with("webp") {
                let op = path_to_compress.clone();
                path_to_compress = format!("{}.png", path_to_compress);
                let cmpr = run_command_nicely(Command::new("convert").arg(&op).arg(&path_to_compress));
                if cmpr.0 != 0 {
                    eprintln!("Could not convert webp image... serving as-is");
                    converted_badges.push((badge.0, badge.1, op));
                    continue;
                }
            }

            // now we can try to compress it
            let save_path = format!("artifacts/88x31/published/{}.jxl", badge.0);
            // compress!
            let cjxl = run_command_nicely(Command::new("cjxl").arg("-d").arg("0").arg("-e").arg("10").arg(&path_to_compress).arg(&save_path));
            if cjxl.0 == 0 {
                println!("Cached and optimized {}", badge.0);
                converted_badges.push((badge.0, badge.1, format!("/{save_path}")));
                continue;
            }
            eprintln!("JXL compression failed. Serving the uncompressed version: {}", cjxl.1);
            converted_badges.push((badge.0, badge.1, format!("/{path_to_compress}")));
        }
        // now we can generate the HTML
        html! {
            div."badges" {
                @for (index, badge) in converted_badges.iter().enumerate() {
                    a href=(badge.1) target="_blank" {
                        img alt=(badge.0) src=(badge.2) class="eightyeightthirtyone";
                    }
                    @if (index + 1) % 5 == 0 {
                        br;
                    }
                }
            }
        }
    };
}

pub fn badges() -> Markup {
    BADGES.clone()
}