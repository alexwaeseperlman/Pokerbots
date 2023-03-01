use leptos::{ev::MouseEvent, *};
use std::vec;

use std::fmt;

#[component]
fn TeamMember(cx: Scope, id: i32) -> impl IntoView {
    view! {
        cx,
        <div class="w-full mt-2">"Email address: "<input required name={format!("member-{}", id)} class="w-full" type="email"/></div>
    }
}

#[component]
pub fn SignUp(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    let (member_in, set_member_in) = create_signal::<Vec<View>>(cx, vec![]);

    let add_member = move || {
        let mut a = member_in();
        a.push(
            view! { cx, <TeamMember id={member_in().len().try_into().unwrap()}/> }.into_view(cx),
        );
        set_member_in(a);
    };
    // TODO: Figure out why adding a member at the start causes the first member not to render after adding more
    // add_member();
    let rem_member = move || {
        let mut a = member_in();
        a.pop();
        set_member_in(a);
    };
    let few_members = move || member_in().len() <= 1;
    let many_members = move || member_in().len() >= TEAM_SIZE.try_into().unwrap();

    view! {
        cx,
        <form action="/maketeam">
            <div class="w-full flex content-center flex-col flex-wrap">
                <div class="w-fit text-center"><h1 class="text-center">"Create a team"</h1></div>
                "Team id: " <input name="teamid"/>
                <div class="w-fit text-center">
                    <button on:click={move |ev| {
                        ev.prevent_default();
                        add_member();
                    }} prop:disabled=many_members>"Add member"</button>
                    <button on:click={move |ev| {
                        ev.prevent_default();
                        rem_member();
                    }} prop:disabled=few_members>"Remove member"</button>
                </div>
                <div class="w-80">
                    { member_in }
                </div>
                <button class="primary mt-4">"Submit"</button>
            </div>
        </form>
    }
}
