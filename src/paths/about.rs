use axum::extract::State;
use maud::{html, Markup, PreEscaped};

use crate::util::state::SiteState;

use super::root::{MergedPage};



pub async fn index(State(state): State<SiteState>) -> Markup {
    MergedPage::new_content_and_meta_main("About", "A little information about myself", html! {
        h1 { "Well hello 👋 guess you wanted to know more about me" }
        p { "I'm currently a fourth year at the Georgia Institute of Technology
        (I’m just going to call it Georgia Tech, but I guess that’s the official title it’s supposed to be called by)
        I am currently studying Computer Engineering, focusing in distributed systems and hardware. I was formerly
        Computer Science with Intelligence and Media (also then formerly Devices)" }
        p { "Much of my free time when I’m not attempting to be productive usually consists
        of listening to music on Apple Music Radio in my car or at my desk.
        I tend to see one to two movies a week (or try to, now slipping to 2-3 a month) or attending concerts in my area.
        Much of my time is spent working on various personal projects such as the homelab (post soon)
        or watching the many movie recommendations that I get from friends.
        that powers this website as well as all the other things I host."}
        p { "Whenever I’m not on campus (and at home), I manage (and have for many years)
        a 65 gallon saltwater aquarium complete with some fish, plenty of coral, some shrimp, and an anemone (well, 4, it keeps multiplying).
        Unfortunately my phone camera cannot take good pictures of the tank, otherwise I would put one here.
        When I am on vacation, I tend to scuba dive wrecks in the area, or coral reefs (posts soon, allegedly)." }
        (PreEscaped(state.map_internal_image("assets/images/about/diving.jpg")))
        p { "On campus, I am an active member of WREK Atlanta, the school’s public radio station (available at 91.1 FM in Georgia or at " a href="https://wrek.org" {"wrek.org"} "anywhere with internet),
        where I operate the station every Friday at 11am to 12pm. I am also on the engineering team as the Chief Engineer where we are currently working on modernizing some of our older systems.
        I am also on the exec board of the Amateur Radio Club (W4AQL). I'm not too active over the airwaves, but I can be found as KQ4QME!"}
        p{ "If you would like my personal resume, can be obtained upon email request. My LinkedIn should be mostly up to date however."}
        i { "This page was last updated June 27, 2025 It is likely not complete or entirely up to date."}
        br;
        br;
        a href="/" { "Return home"}
    }, state).render()
}
