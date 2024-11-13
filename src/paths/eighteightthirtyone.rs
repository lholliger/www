use lazy_static::lazy_static;
use maud::{html, Markup};

lazy_static! {
    static ref BADGES: Markup = {
        println!("Generating 88x31 content!");
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
        html! {
            div."badges" {
                @for (index, badge) in badges.iter().enumerate() {
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