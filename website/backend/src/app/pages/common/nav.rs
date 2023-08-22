use super::*;

#[derive(PartialEq)]
pub enum NavLocation {
    HomePage,
    ManageTeam,
    Leaderboard,
}

pub fn top_nav(user: Option<UserData>, location: NavLocation) -> Markup {
    html! {
        nav hx-boost="true" class="nav" {
            a class="nav-item" href="/" {
                img src="/static/logo.svg" alt="Pokerbots logo" class="logo" {};
            };
            a href="/manage-team" class=(format!("nav-item selectable {}", if location == NavLocation::ManageTeam { "selected" } else { "" })) {
                "MANAGE TEAM"
            };

            a href="/leaderboard" class="nav-item selectable" {
                "LEADERBOARD"
            };

            div style="flex-grow: 1;" {};

            a href="https://github.com/alexwaeseperlman/Pokerbots/wiki" class="nav-item selectable" {
                "DOCUMENTATION"
            };
            @if user.is_some() {
                a href="/signout" class="nav-item selectable" {
                    "SIGN OUT"
                };
            }
        };
    }
}

pub fn bottom_nav() -> Markup {
    html! {
        nav class="nav" {
            a class="nav-item" {
                "© Poker Bot League 2023"
            };
            div style="flex-grow: 1;" {};
            a class="nav-item selectable" href="https://github.com/alexwaeseperlman/Pokerbots/issues" {
                "REPORT AN ISSUE"
            };
        };
    }
}
