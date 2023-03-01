use super::super::app_config::*;
use leptos::{ev::MouseEvent, *};
use leptos_meta::*;
use leptos_router::*;
use leptos_server::*;
use std::fmt;

pub fn microsoft_login_url() -> String {
    format!("https://login.microsoftonline.com/common/oauth2/v2.0/authorize?client_id={}&response_type=code&redirect_uri=http%3A%2F%2Flocalhost:3000%2Fteam&response_mode=query&scope=https%3A%2F%2Fgraph.microsoft.com%2Femail", CLIENT_ID)
}
