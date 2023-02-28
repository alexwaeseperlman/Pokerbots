use crate::app::login::*;
use leptos::{ev::MouseEvent, *};

#[component]
pub(crate) fn HomePage(cx: Scope) -> impl IntoView {
    view! {
        cx,
        <div class="px-64 bg-red-600 py-12 w-full h-72 flex content-baseline">
            <h1 class="text-white mt-auto">"Pokerbots McGill"</h1>
        </div>
        <div class="px-64 py-12 w-full flex content-baseline gap-4">
            <a href={microsoft_login_url()}>"Log in with Microsoft"</a>
        </div>
    }
}
