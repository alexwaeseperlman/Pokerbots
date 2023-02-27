use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[path = "./app_config.rs"]
mod app_config;

mod pages {
    pub mod homepage;
    pub mod signup;
}

mod components {
    pub mod nav;
}
use pages::homepage::*;
use pages::signup::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    view! {
        cx,

        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/leptos_start.css"/>

        // sets the document title
        <Title text="Pokerbots McGill"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=|cx| view! { cx, <HomePage/> }/>
                    <Route path="/signup" view=|cx| view! { cx, <SignUp/> }/>
                </Routes>
            </main>
        </Router>
    }
}
