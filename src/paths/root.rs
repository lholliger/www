use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use chrono::{NaiveDateTime, Utc};
use maud::{html, Markup, PreEscaped};

use crate::util::state::SiteState;
pub struct MergedPage {
    title: Option<String>,
    meta_description: Option<String>,
    meta_image: Option<String>,
    body: PreEscaped<String>,
    main_page: bool,
    state: Option<SiteState>
}

impl MergedPage {
    // may want to generate a description based off of the content
    pub fn new(title: Option<&str>, description: Option<&str>, image: Option<&str>, body: PreEscaped<String>, main_page: bool, state: SiteState) -> MergedPage {
        MergedPage {
            title: title.map(|s| s.to_string()),
            meta_description: description.map(|s| s.to_string()),
            meta_image: image.map(|s| s.to_string()),
            body,
            main_page,
            state: Some(state)
        }
    }

    pub fn new_content_and_meta(title: &str, description: &str, body: PreEscaped<String>, state: SiteState) -> MergedPage {
        MergedPage::new(Some(title), Some(description), None, body, false, state)
    }

    pub fn new_content_and_meta_main(title: &str, description: &str, body: PreEscaped<String>, state: SiteState) -> MergedPage {
        MergedPage::new(Some(title), Some(description), None, body, true, state)
    }
    pub fn render(&self) -> Markup {
        html! {
            (maud::DOCTYPE);
            html {
                head {
                    meta charset="utf-8";
                    meta name="viewport" content="width=device-width, initial-scale=1.0";
                    meta name="author" content="Lukas Holliger";
                    @if self.title.is_none() {
                        title { "Lukas Holliger" }
                        meta property="og:title" content="Lukas Holliger";
                    } @else {
                        title { "Lukas Holliger | " (self.title.as_ref().unwrap()) }
                        meta property="og:title" content=(format!("Lukas Holliger | {}", self.title.as_ref().unwrap()));
                    }
                    @if !self.meta_description.is_none() {
                        meta property="og:description" content=(self.meta_description.as_ref().unwrap());
                        meta name="description" content=(self.meta_description.as_ref().unwrap());
                    }

                    @if !self.meta_image.is_none() {
                        meta property="og:image" content=(self.meta_image.as_ref().unwrap());
                    }

                    link rel="stylesheet" href=(format!("/assets/css/style.css?revision={}", env!("GIT_HASH")));
                }
                body {
                    header {
                        div."header-top" {
                            a href="/" { h1 { "Lukas Holliger" } }
                            span alt="profile picture" class="profile-image" {
                                 @if let Some(state) = &self.state {
                                    (PreEscaped(state.map_internal_image("assets/images/me.jpeg").replace("type", "sizes=\"267px\" type"))) // this feels like a bad hack
                                }
                            }
                        }
                        @if self.main_page {
                            ul {
                                li { a href="/about" {"About"} };
                            }
                        }
                    }
                    main {
                        (self.body)
                    }
                    footer {
                        @if self.main_page {
                            div."badges" {
                                @if let Some(state) = &self.state {
                                    (maud::PreEscaped(state.get_cached_html_element("badge_html")))
                                }
                            }
                            /*div."fah" { // its summer!
                                i { "Stay warm this winter, do some folding!" }
                                br;
                                a href="https://folding.extremeoverclocking.com/user_summary.php?s=&u=1353754" { img src="https://folding.extremeoverclocking.com/sigs/sigimage.php?u=1353754"; }
                            }*/
                        } @else {
                            a href="/" { "Return home"}
                            br;
                            br;
                        }
                        p { "Source code " a href="https://github.com/lholliger/www" { "available here" } " released under the " a href="https://github.com/lholliger/www/blob/main/COPYING" {"GNU AGPLv3 license"} }
                        p { "Updated " (NaiveDateTime::parse_from_str(env!("GIT_TIME"), "%s").unwrap().and_local_timezone(Utc).unwrap().to_rfc2822()) " (" (env!("GIT_MESSAGE")) ") [" (env!("GIT_HASH")) "]"}
                        p { "All opinions here are my own and do not reflect the views of my employers or university: future, past, and present." }
                    }
                }
            }
        }
    }
}

pub fn index(state: SiteState) -> Markup {
    MergedPage::new(None, Some("Hello 👋 I'm Lukas"), Some("/assets/images/me.jpeg"), html! {
                    section class="hero" {
                        h2 { "Hello 👋 I'm Lukas" }
                        p { "I'm a software engineer from Atlanta, Georgia currently studying Computer Engineering at Georgia Tech. I am currently an intern at Apple working in Services Engineering where I have work on various video technologies in areas around streaming and metadata." }
                        p { "I have experience and enjoy working with large sums of data in Rust, JavaScript, and/or TypeScript. In the past I've analyzed millions of projects and accounts on Scratch, as well as made an API to query ranking information.
                        At Apple, I've worked on HLS technologies in areas of metadata, live streaming, and ad serving."}
                        p { "Currently I'm between a lot of different projects (a lot to do with video encoding and also QR codes, perhaps both together at the same time), and yet another class requires me to make a blog/website, so this website is existing from that class and also pressure from many friends. Hopefully this one I can keep up to date!" }
                        p { "If you want to stay in touch, feel free to add me on " a href="https://linkedin.com/in/lukasholliger" {"LinkedIn"} ", contact me on Signal at " tt {"@lukash.01"} ", or email me at " tt {"[anything]@holliger.me"} ". "}
                    }
                    section class="content" {
                        h3 { "Posts" }
                        (PreEscaped(state.get_cached_html_element("post_index_html")))
                        a href="/posts" {"View all posts"}
                    }
    }, true, state).render()
}

pub fn error_page(code: StatusCode, message: &str, state: SiteState) -> (StatusCode, Markup) {
    (code, MergedPage::new_content_and_meta("404","Page not found :(", html! {
        div {
            h1 { "Error " (code.as_u16()) }
            @if message.len() > 0 {
                p { (message) }
            } else {
                p { "Looks like that page can't be found :(" }
                a href="/" {"Return home"}
            }
        }
    }, state).render())
}

pub fn error_page_file(code: StatusCode, message: &str, state: SiteState) -> (StatusCode, Markup) {
    (code, MergedPage::new_content_and_meta("404","File not found :(", html! {
        div {
            h1 { "Error " (code.as_u16()) }
            @if message.len() > 0 {
                p { (message) }
            } else {
                p { "Looks like that file can't be found :(" }
                p { "Have you made sure every part of the URL is correct? Capitalization matters!" }
                a href="/" {"Return home"}
            }
        }
    }, state).render())
}

pub async fn serve_generated_image(State(state): State<SiteState>, Path(image): Path<String>) -> Result<Response, (StatusCode, Markup)> {
    let image_name: String = image.to_string();
    let ext = image_name.split_once(".").ok_or_else(|| {
        error_page(StatusCode::BAD_REQUEST, "Invalid image name format", state.clone())
    })?;

    let corrected_image_path = format!("/generated/{}", image);

    let image_data = state.get_image(&corrected_image_path).ok_or_else(|| {
        error_page(StatusCode::NOT_FOUND, "Image not found :(", state)
    })?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", format!("image/{}", ext.1))
        .body(image_data.to_vec().into())
        .unwrap())
}


// we need to build the list of posts n things