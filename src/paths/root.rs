use axum::http::StatusCode;
use maud::{html, Markup};

use crate::paths::{eighteightthirtyone::badges, posts::get_posts_html};

pub fn merge_page(body: Markup, is_main_page: bool) -> Markup {
    html! {
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "Lukas Holliger" }
                link rel="stylesheet" href="/assets/css/style.css";
            }
            body {
                header {
                    div."header-top" {
                        a href="/" { h1 { "Lukas Holliger" } }
                        img src="/assets/images/me.jpeg" alt="profile picture" class="profile-image";
                    }
                    @if is_main_page {
                        ul {
                            li { a href="/about" {"About"} };
                        }
                    }
                }
                main {
                    (body)
                }
                footer {
                    @if is_main_page {
                        (badges())
                    } @else {
                        a href="/" { "Return home"}
                    }
                    p { "All opinions here are my own and do not reflect the views of my employers or university: future, past, and present." }
                }
            }
        }
    }
}

pub fn index() -> Markup {
    merge_page(html! {
                    section class="hero" {
                        h2 { "Hello 👋 I'm Lukas" }
                        p { "I'm a software engineer in Atlanta, Georgia currently studying Computer Engineering at Georgia Tech. I was previously (and will be in 2025) an intern at Apple, working in Services Engineering where I have worked on various HLS and metadata technologies." }
                        p { "I have experience and enjoy working with large sums of data in Rust, JavaScript, and/or TypeScript. In the past I've analyzed millions of projects and accounts on Scratch, as well as made an API to query ranking information.
                        At Apple, I've worked on HLS technologies in areas of metadata, live streaming, and ad serving."}
                        p { "Currently I'm between a lot of different projects (a lot to do with video encoding and also QR codes, perhaps both together at the same time), and yet another class requires me to make a blog/website, so this website is existing from that class and also pressure from many friends. Hopefully this one I can keep up to date!" }
                        p { "If you want to stay in touch, feel free to add me on " a href="https://linkedin.com/in/lukasholliger" {"LinkedIn"} ", contact me on Signal at " tt {"@lukash.01"} ", or email me at " tt {"[anything]@holliger.me"} ". "}
                    }
                    section class="content" {
                        h3 { "Posts" }
                        (get_posts_html(5))
                    }
    }, true)
}

pub fn error_page(code: StatusCode, message: &str) -> (StatusCode, Markup) {
    (code, merge_page(html! {
        div {
            h1 { "Error " (code.as_u16()) }
            @if message.len() > 0 {
                p { (message) }
            } else {
                p { "Looks like that page can't be found :(" }
                a href="/" {"Return home"}
            }
        }
    }, true))
}


// we need to build the list of posts n things