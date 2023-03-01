use cfg_if::cfg_if;
use leptos::{ev::MouseEvent, *};
use leptos_server::*;

#[server(GetTeam, "/api")]
pub async fn getTeam() -> Result<i32, ServerFnError> {
    log!("test");
    Ok(1)
}

#[cfg(feature = "ssr")]
pub fn register_server_functions() {
    GetTeam::register();
}

#[component]
pub(crate) fn Team(cx: Scope) -> impl IntoView {
    let (val, set_val) = create_signal(cx, 0);
    view! {
        cx,
        <div>
            "Worked! " {val}
        </div>
    }
}
