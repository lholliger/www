use maud::html;
use crate::image::ImageCompressor;

struct Badge {
    name: String,
    url: String,
    paths: Vec<String>
}

pub fn compress_badges(csv_path: &str, compressor: &ImageCompressor) -> Vec<(String, String, Vec<String>)> {
    let csv = std::fs::read_to_string(csv_path)
        .unwrap_or(String::new());
    let lines = csv.lines();
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

    let mut converted_badges = Vec::new();
    for badge in badges {
        let compressed = compressor.compress_lossless(&badge.2)
            .expect("Could not handle 88x31!")
            .iter().map(|x| x.path.clone()).collect();
        converted_badges.push((badge.0, badge.1, compressed));
    }
    return converted_badges;
}


pub fn generate_badge_file(converted_badges:  Vec<(String, String, Vec<String>)>) -> String {
    let mut badges = Vec::new();
    let mut builder = phf_codegen::Map::new();

    for (name, url, images) in &converted_badges {
        let mut image_paths = Vec::new();
        for image in images {
                let ext = image.split("/").last().unwrap().split(".").last().unwrap();
                let im_path = format!("{name}.{ext}");
                image_paths.push(im_path.clone());
                builder.entry(im_path, format!("include_bytes!(\"{}\")", image).as_str());
        }
        badges.push(Badge {
            name: name.to_string(),
            url: url.to_string(),
            paths: image_paths
        });
    }

    let badge_build = html! {
            @for (_, badge) in badges.iter().enumerate() {
                a href=(badge.url) target="_blank" {
                    picture class="eightyeightthirtyone" {
                        @let urls = &badge.paths;
                        @for (i, url) in urls.iter().enumerate() {
                            @if i < urls.len() - 1 {
                                source alt=(badge.name) srcset=(format!("/88x31/{}", url)) type=(format!("image/{}", url.split_once(".").unwrap().1));
                            } @else {
                                img alt=(badge.name) width="88" height="31" src=(format!("/88x31/{}", url));
                            }
                        }
                    }
                }
            }
    };

    // now we can generate the output string
    let output = format!("// This file was auto generated, do not modify!

pub const BADGE_HTML: &str = \"{}\";

static BADGE_DATA: phf::Map<&'static str, &[u8]> = {};",
        badge_build.into_string().replace("\"", "\\\""),
        builder.build()
    );
    return output;
}