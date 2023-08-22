use super::*;

pub fn homepage() -> Markup {
    html! {
        link rel="stylesheet" href="/static/css/homepage.css" {};
        div class="background-graphic poker-bot-background-graphic" {}
        div class="page-container" {
            div class="logo-big" {
                img src="/static/logo.svg" alt="Pokerbots logo" class="logo" {};
            };
            h1 class="headline" { "Poker Bot League" }
            a class="discord-button" href="https://discord.gg/2Yb8XpG" {
                img src="/static/discord.svg" alt="Discord logo" class="discord-logo" {};
                "JOIN OUR DISCORD"
            };
            p class="home-page-text" {
                "The competition will start in 2024. For sponsorship inquiries, please contact alex.waese-perlman@mcgill.ca"
            }
        }
    }
}

#[routes]
#[get("/homepage")]
#[get("/")]
pub async fn homepage_route(session: Session) -> Result<impl Responder, actix_web::Error> {
    let user = login::get_user_data(&session);
    Ok(common::base_page(
        homepage(),
        user,
        common::nav::NavLocation::HomePage,
    ))
}
