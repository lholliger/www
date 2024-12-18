use std::collections::HashMap;
use std::fs;
use std::process::Command;

pub struct ImageCompressor {
    work_directory: String,
    speed_mode: bool
}

// this acts as an information center for the image, it may read files from the fs but it SHOULD NOT download anything!
#[derive(Debug, Clone)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub path: String,
    pub codec: String,
    animated: bool,
    cached: bool // this value is to be set to FALSE when the "parent" is changed... this may need to be done intelligently
}

impl Image {

    fn new_with_cache_set(path: String, cached: bool) -> Self {
        let ext = path.split('.').last().expect("No image extension").to_string();
        let mut img = Image {
            width: 0,
            height: 0,
            codec: ext,
            path: path.clone(),
            animated: false,
            cached
        };

        img.refresh();

        img
    }

    fn new(path: String) -> Self {
        Self::new_with_cache_set(path, true)
    }

    fn new_url_or_path(path: &String, working_directory: &String) -> Self {
        if path.starts_with("http") {
            let ext = path.split('.').last().expect("No image extension").to_string();
            Self::new(format!("{}/artifacts/cache/image-url-{:x}.{}", working_directory, crc32fast::hash(path.as_bytes()), ext))
        } else {
            Self::new(path.clone())
        }
    }

    fn refresh(&mut self) { // this should perhaps see if the current image changed and force cache to false
        let image_res_details = run_command_nicely(Command::new("identify").arg(&self.path));
        if image_res_details.0 != 0 { // the command failed
            // eprintln!("Identify failed for {}", self.path);
        } else {
            self.animated = false;
            for line in image_res_details.1.lines() {
                if line.contains("[0]") {
                    self.animated = true;
                }
                let text = line
                        .split(" ").nth(3).expect("Malformed entry line")
                        .split_once("+").expect("Malformed entry line (+)").0
                        .split_once("x").expect("Malformed entry line (x)");
                    self.width = text.0.split(".").next().expect("no width defined?").parse().expect("Number not found for width");
                    self.height = text.1.split(".").next().expect("no height defined?").parse().expect("Number not found for height");
            }
        }
    }

    fn update_path(&mut self, path: String) {
        let ext = path.split('.').last().expect("No image extension").to_string();
        self.codec = ext;
        self.path = path;
        self.refresh();
    }
    fn file_violates_cache_rules(&self) -> bool {
        if !self.cached {
            return true;
        }
        if let Ok(metadata) = fs::metadata(self.path.clone()) {
            if let Ok(modified) = metadata.modified() {
                let age = modified.elapsed().unwrap_or_default();
                if age.as_secs() < 60*60*24*7 { // dont hardcode?
                    return false;
                }
            }
        }
        return true;
    }
}

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

impl ImageCompressor { // speed mode is effectively just whenever running in debug, can be disabled by flag
    pub fn new(working_directory: &str, speed_mode: bool) -> Self {
        fs::create_dir_all(format!("{working_directory}/artifacts/cache")).unwrap();
        fs::create_dir_all(format!("{working_directory}/artifacts/publish")).unwrap();
        ImageCompressor {
            work_directory: working_directory.to_string(),
            speed_mode
        }
    }

    // TODO: overhaul to skip checking when caches are all good
    pub fn compress_with_encoding_options(&self, image_path: &str, webp_lossless: bool, webp_quality: u32, jxl_compression: f32, webp_animation_effort: u8, webp_effort: u8, jxl_effort: u8, override_resolutions: Option<Vec<usize>>) -> Result<Vec<Image>, &str> {
        let mut images: Vec<Image> = Vec::new();

        let mut working_image = Image::new_url_or_path(&image_path.to_string(), &self.work_directory);
        println!("{:?}", working_image);
        // we must download the image first, or verify it is already cached and not expired
        if image_path.starts_with("http") {
            if working_image.file_violates_cache_rules() { // may not exist
                println!("{} is not already cached! Downloading...", &image_path);
                let curl = run_command_nicely(Command::new("curl").arg(&image_path).arg("-o").arg(&working_image.path));
                if curl.0 != 0 { // the command failed
                    // could not serve the image!
                    return Err("Curl failed");
                }

                //working_image.update_path(hold_path);
                working_image.refresh();
                working_image.cached = false;
            }
        }

        let cache_image_output = format!("{}/artifacts/cache/{}", self.work_directory, crc32fast::hash(working_image.path.as_bytes()));
        let publish_image_output = format!("{}/artifacts/publish/{}", self.work_directory, crc32fast::hash(working_image.path.as_bytes()));

        // the conversion tools do not like each other, so in these cases it should be made into PNG
        // this should likely utilize "codec"
        if (working_image.path.ends_with("webp") || working_image.path.ends_with("jxl")) && !working_image.animated {
            let png_image = Image::new(format!("{}.png", cache_image_output));
            if png_image.file_violates_cache_rules() { // we dont need to do PNG conversion if the file has never changed and doenst need to be re-encoded
                println!("{} is in a format that cannot be compressed easily! Converting to PNG to re-compress...", working_image.path);
                let cmpr = run_command_nicely(Command::new("convert").arg(&working_image.path).arg(&png_image.path));
                if cmpr.0 != 0 {
                    eprintln!("Could not convert webp image... must serve AS-IS!");
                    return Ok(vec![working_image]);
                }
                working_image.cached = false;
            }
            working_image.update_path(png_image.path);
        }

        // now the working file should be in a... workable format, assuming its not animated

        let run_array = match override_resolutions {
            Some(resolutions) => resolutions,
            None => if !self.speed_mode {
                vec![1800, 1530, 1200, 792, 600, 300, working_image.width]
            } else {
                if working_image.width < 600 {
                    vec![working_image.width]
                } else {
                    vec![1200, 600]
                }
            }
        };

        // linter says 553×353, 792×506, 1200×766, 1530×977, 1800×1149 or something close, also encode default
        for res in run_array {
            if working_image.width < res {
                continue;
            }

            if res > 1800*2 {
                // don't deal with massive images
                continue;
            }
            // we can now "publish"

            if working_image.animated {
                if working_image.width == res {
                    let mut webp_conversion = Image::new_with_cache_set(format!("{}_ani_{res}.webp", publish_image_output), working_image.cached);
                    if webp_conversion.file_violates_cache_rules() {
                        let webpa = run_command_nicely(
                            Command::new("gif2webp")
                                .arg("-q").arg(webp_quality.to_string()).arg("-m")
                                .arg(webp_animation_effort.to_string()).arg("-metadata").arg("icc")
                                .arg(&working_image.path).arg("-o").arg(&webp_conversion.path)
                        );
                        if webpa.0 == 0 {
                            println!("Cached and optimized {} for Animated WEBP", working_image.path);
                            webp_conversion.refresh();
                        } else {
                            eprintln!("Animated WEBP compression failed for {}", working_image.path);
                        }
                    }
                    if webp_conversion.animated == true {
                        images.push(webp_conversion);
                    }
                    images.push(working_image.clone());
                }
                continue;
            }

            // so we need to resize first...
            let mut resized_version = Image::new_with_cache_set(format!("{}_resize_pre_{res}.png", cache_image_output), working_image.cached);
            if resized_version.file_violates_cache_rules() {
                println!("Resizing {} to {}px width", working_image.path, res);
                let resize = run_command_nicely(Command::new("convert").arg(&working_image.path).arg("-resize").arg(format!("{}x", res)).arg(&resized_version.path));
                if resize.0 != 0 {
                    eprintln!("Could not resize image! Cancelling");
                    return Ok(vec![working_image]);
                }
                resized_version.refresh();
                resized_version.cached = false
            }

            // the image can be compressed down to this size
            // three encoders to use, JXL, WEBP, AVIF(TODO)

            let mut jxl_version = Image::new_with_cache_set(format!("{}_{res}.jxl", publish_image_output), resized_version.cached);
            if jxl_version.file_violates_cache_rules() {
                let cjxl = run_command_nicely(Command::new("cjxl").arg("-d").arg(jxl_compression.to_string()).arg("-e").arg(jxl_effort.to_string()).arg(&resized_version.path).arg(&jxl_version.path));
                if cjxl.0 == 0 {
                    println!("V2: Cached and optimized {} for JXL", jxl_version.path);
                    jxl_version.refresh();
                    images.push(jxl_version);
                } else {
                    eprintln!("V2: JXL compression failed for {}", cjxl.1);
                }
            } else {
                println!("V2: Using optimized {} for JXL", jxl_version.path);
                images.push(jxl_version);
            }

            let mut webp_version = Image::new_with_cache_set(format!("{}_{res}.webp", publish_image_output), resized_version.cached);
            if webp_version.file_violates_cache_rules() {
                let cwebp = if webp_lossless {
                    run_command_nicely(
                        Command::new("cwebp").arg("-mt")
                            .arg("-lossless").arg("-z").arg(webp_effort.to_string())
                            .arg("-alpha_filter").arg("best")
                            .arg("-metadata").arg("icc")
                            .arg(&resized_version.path).arg("-o").arg(&webp_version.path)
                    )
                } else {
                    run_command_nicely(
                        Command::new("cwebp").arg("-mt")
                            .arg("-q").arg(webp_quality.to_string()).arg("-z").arg(webp_effort.to_string())
                            .arg("-alpha_filter").arg("best")
                            .arg("-metadata").arg("icc")
                            .arg(&resized_version.path).arg("-o").arg(&webp_version.path)
                    )
                };
                if cwebp.0 == 0 {
                    println!("V2: Cached and optimized {} for WEBP", webp_version.path);
                    webp_version.refresh();
                    images.push(webp_version);
                } else {
                    eprintln!("V2: WEBP compression failed for {}", cwebp.1);
                }
            } else {
                println!("V2: Using optimized {} for WEBP", webp_version.path);
                images.push(webp_version);
            }
        }
        Ok(images)
    }

    pub fn compress_lossy(&self, image_path: &str) -> Result<Vec<Image>, &str> {
        if self.speed_mode {
            return self.compress_speed(image_path)
        }
        self.compress_with_encoding_options(image_path, false, 100,  1.0, 6, 9, 10, None)
    }

    pub fn compress_lossless(&self, image_path: &str) -> Result<Vec<Image>, &str> {
        if self.speed_mode {
            return self.compress_speed(image_path)
        }
        self.compress_with_encoding_options(image_path, true, 100,  0.0, 6, 9, 10, None)
    }

    pub fn compress_speed(&self, image_path: &str) -> Result<Vec<Image>, &str> {
        self.compress_with_encoding_options(image_path, false, 75, 3.0, 1, 1, 5, None)
    }

    pub fn compress_lossy_with_custom_resolutions(&self, image_path: &str, overrides: Vec<usize>) -> Result<Vec<Image>, &str> {
        self.compress_with_encoding_options(image_path, false, 100, 1.0, 6, 9, 10, Some(overrides))
    }
}

pub fn zip_images_and_paths_to_file(images: Vec<(Image, String)>) -> String {
    let mut builder = phf_codegen::Map::new();
    for image in images {
        builder.entry(image.1, format!("include_bytes!(\"{}\")", image.0.path).as_str()); // MASSIVE speedup on building, and much less space used!
    }
    let output = format!("// This file was auto generated, do not modify!

static IMAGES: phf::Map<&'static str, &[u8]> = {};",
                         builder.build()
    );
    return output;
}

pub fn process_internal_images_to_file(csv_path: &str, compressor: &ImageCompressor) -> (Vec<(Image, String)>, String) {
    let csv = std::fs::read_to_string(csv_path)
        .unwrap_or(String::new());
    let lines = csv.lines();
    let mut images = Vec::new();
    for line in lines {
        let mut fields = line.split(',');
        let name = fields.next().unwrap();
        let resolutions = fields.map(|x| x.parse::<usize>().expect("value was not a valid integer")).collect::<Vec<_>>();
        images.push((name, resolutions));
    }

    let mut images_output: Vec<(Image, String)> = Vec::new();

    let mut builder = phf_codegen::Map::new();

    for image in images {
        let compressor_outputs = compressor.compress_lossy_with_custom_resolutions(image.0, image.1).expect("Could not compress image!");
        let output = convert_image_list_to_html_element_and_map(compressor_outputs, None);
        images_output.extend(output.0);
        builder.entry(image.0.to_string(), format!("\"{}\"", output.1.replace("\"", "\\\"")).as_str());
    }

    let output = format!("// This file was auto generated, do not modify!

pub static INTERNAL_IMAGES: phf::Map<&'static str, &str> = {};",
                         builder.build()
    );

    return (images_output, output);
}


pub fn convert_image_list_to_html_element_and_map(compressed_images: Vec<Image>, sizes: Option<&str>) -> (Vec<(Image, String)>, String) {
    let mut image_map = HashMap::new();
    let mut post_images = Vec::new();
    for image in &compressed_images {
        if !image_map.contains_key(&image.codec) {
            image_map.insert(image.codec.clone(), vec![]);
        }
        let image_path = format!("/generated/{}", image.path.split("/").last().expect("Image doesn't exist?"));
        let asset = image_map.get_mut(&image.codec).unwrap();
        asset.push(format!("{} {}w", image_path, image.width));
        post_images.push((image.clone(), image_path));
    }
    // TODO: this can look prettier
    let output = match sizes {
        Some(size) => {
        format!("<picture><source type=\"image/jxl\" sizes=\"{}\" srcset=\"{}\"><img sizes=\"{}\" type=\"image/webp\" srcset=\"{}\" src=\"{}\"></picture>\n",
                      size,
                      image_map.get("jxl").expect("No JXL!").join(", "),
                      size,
                      image_map.get("webp").expect("No WEBP!").join(", "),
                      image_map.get("webp").expect("No WEBP!").get(0)
                          .expect("No webp 0?").split_once(" ").expect("Malformed compression").0)
        },
        None => {
            format!("<picture><source srcset=\"{}\" type=\"image/jxl\"><img srcset=\"{}\" type=\"image/webp\" src=\"{}\"></picture>\n",
                image_map.get("jxl").expect("No JXL!").join(", "),
                image_map.get("webp").expect("No WEBP!").join(", "),
                image_map.get("webp").expect("No WEBP!").get(0)
                .expect("No webp 0?").split_once(" ").expect("Malformed compression").0)
            }
        };

    return (post_images, output);
}