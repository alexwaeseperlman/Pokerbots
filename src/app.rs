use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use super::app_config::*;
pub mod pages {
    pub mod homepage;
    pub mod team;
}

pub mod login;
use login::*;
pub mod components {
    pub mod nav;
}
use pages::homepage::*;
use pages::team::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    view! {
        cx,
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/pokerbots.css"/>

        // sets the document title
        <Title text="Pokerbots McGill"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=|cx| view! { cx, <HomePage/> }/>
                    <Route path="/team" view=|cx| view! { cx, <Team/> }/>
                </Routes>
            </main>
        </Router>
    }
}
