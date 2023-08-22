use crate::app::login::UserData;

use super::*;
pub mod nav;

pub fn base_page(body: Markup, user: Option<UserData>, location: nav::NavLocation) -> Markup {
    html! {
        (DOCTYPE)
        meta charset = "utf-8" {};
        meta name = "viewport" content = "width=device-width, initial-scale=1.0" {};
        link rel = "icon" type = "image/png" sizes = "32x32" href = "/static/favicon-32x32.png" {};
        link rel = "icon" type = "image/png" sizes = "16x16" href = "/static/favicon-16x16.png" {};
        link rel = "stylesheet" href="/static/css/common.css" {};

        title { "Pokerbots" }

        body class="background" style = "padding: 0; margin: 0; min-height: 100vh; display: flex; flex-direction: column;" {
            (nav::top_nav(user, location))
            div id = "root" style="display:flex;flex-grow: 1;" {
                (body)
            }
            (nav::bottom_nav())
        }

        script type = "module" src = "/static/js/common.js" {};
        script src="https://unpkg.com/htmx.org@1.9.3" {}
    }
}
