use maud::{html, Markup};

use super::root::MergedPage;



pub async fn index() -> Markup {
    MergedPage::new_content_and_meta_main("About".to_string(), "A little information about myself".to_string(), html! {
        h1 { "Well hello 👋 guess you wanted to know more about me" }
        p { "I'm currently a third year at the Georgia Institute of Technology
        (I’m just going to call it Georgia Tech, but I guess that’s the official title it’s supposed to be called by)
        I am currently studying Computer Engineering, focusing in distributed systems and hardware. I was formerly
        Computer Science with Intelligence and Media (also then formerly Devices)" }
        p { "Much of my free time when I’m not attempting to be productive usually consists
        of listening to music (Spotify Discover Weekly and Apple Music Radio are a wonderful creations) in my car or at my desk.
        I also enjoy attending concerts or new movie releases whenever I can (AMC A-List!).
        Much of my time is spent working on various personal projects such as the homelab (post soon)
        that powers this website as well as all the other things I host."}
        p { "Whenever I’m not on campus (and at home), I manage (and have for many years)
        a 65 gallon saltwater aquarium complete with some fish, plenty of coral, some shrimp, and an anemone (well, 4, it keeps multiplying).
        Unfortunately my phone camera cannot take good pictures of the tank, otherwise I would put one here.
        When I am on vacation, I tend to scuba dive wrecks in the area, or coral reefs (posts soon, allegedly)." }
        img src="/assets/images/about/diving.jpg";
        p { "On campus, I am an active member of WREK Atlanta, the school’s public radio station (available at 91.1 FM in Georgia or at " a href="https://wrek.org" {"wrek.org"} "anywhere with internet),
        where I operate the station every Friday at 11am to 12pm. I am also on the engineering team as Assistant Chief Engineer where we are currently working on modernizing some of our older systems." }
        p { "If you're looking for my resume (since perhaps you're a part of my ECE1100 class, you can find it " a href="/assets/Holliger_Resume_1100_Privacy.pdf" { "here" }
        ". Do note, this resume is not entirely accurate and my personal resume can be obtained upon email request."}
        i { "This page was last updated November 12, 2024. It is likely not complete or entirely up to date."}
        br;
        br;
        a href="/" { "Return home"}
    }).render()
}
