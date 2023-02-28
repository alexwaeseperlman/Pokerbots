use leptos::{ev::MouseEvent, *};

struct User {
    email: &str;
}

#[component]
pub(crate) fn Team(cx: Scope) -> impl IntoView {
    view! {
        cx,
        <div>
            "Worked!"
        </div>
    }
}
