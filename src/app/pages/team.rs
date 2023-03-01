use crate::models::*;
use cfg_if::cfg_if;
use leptos::{ev::MouseEvent, *};
use leptos_server::*;

#[server(GetTeam, "/api")]
pub async fn get_team(cx: Scope) -> Result<Option<i32>, ServerFnError> {
    use actix_session::SessionExt;
    use actix_web::Error;

    use crate::schema::teams::dsl::teams;
    use crate::DB_CONNECTION;
    use diesel::*;
    let session = crate::get_session(cx);
    teams
        .limit(5)
        .load::<Team>(&mut (*DB_CONNECTION).get().unwrap())
        .expect("Error loading teams");
    log!("Called");
    Ok(Some(1))
}

#[cfg(feature = "ssr")]
pub fn register_server_functions() {
    GetTeam::register().expect("GetTeam could not register");
}

#[component]
pub fn TeamPage(cx: Scope) -> impl IntoView {
    let (val, set_val) = create_signal(cx, 0);
    spawn_local(async move {
        get_team(cx).await;
    });
    view! {
        cx,
        <div>
            "Worked!"
        </div>
    }
}
