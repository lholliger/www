use maud::html;
use crate::util::build::image::ImageCompressor;

#[derive(Debug, Clone)]
pub struct Badge {
    pub name: String,
    url: String,
    pub paths: Vec<String>
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


pub fn generate_badge_file(converted_badges:  Vec<(String, String, Vec<String>)>) -> (String, Vec<Badge>, Vec<(String, Vec<u8>)>) {
    let mut badges = Vec::new();
    let mut image_data = Vec::new();
    for (name, url, images) in &converted_badges {
        let mut image_paths = Vec::new();
        for image in images {
                let ext = image.split("/").last().unwrap().split(".").last().unwrap();
                let im_path = format!("{name}.{ext}");
                image_paths.push(im_path.clone());
                let data = std::fs::read(image).expect(&format!("Failed to read image: {}", image));
                image_data.push((format!("/generated/88x31_{}", im_path), data));
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
                                source alt=(badge.name) srcset=(format!("/generated/88x31_{}", url)) type=(format!("image/{}", url.split_once(".").unwrap().1));
                            } @else {
                                img alt=(badge.name) width="88" height="31" src=(format!("/generated/88x31_{}", url));
                            }
                        }
                    }
                }
            }
    };

    return (badge_build.into_string(), badges, image_data)
}
