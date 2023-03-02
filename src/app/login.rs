use super::super::app_config::*;
use cfg_if::cfg_if;
use leptos::{ev::MouseEvent, *};
use leptos_meta::*;
use leptos_router::*;
use leptos_server::*;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::app_config::TENANT_ID;

pub fn microsoft_login_url() -> String {
    format!("https://login.microsoftonline.com/{}/oauth2/v2.0/authorize?client_id={}&response_type=code&redirect_uri={}&response_mode=query&scope=User.Read", "common", CLIENT_ID, REDIRECT_URI)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AzureMeResponse {
    displayName: String,
    givenName: String,
    mail: String,
    userPrincipalName: String,
    id: String,
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use actix_web::{get, HttpResponse, HttpRequest, web};
        use crate::get_azure_secret;

        use actix_session::Session;

        #[derive(Deserialize)]
        pub struct MicrosoftLoginCode {
            code: Option<String>,
        }

        // https://learn.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow
        #[derive(Deserialize)]
        pub struct AzureAuthTokenResopnse {
            access_token: Option<String>,
            error: Option<String>
        }

        // By the end of this method, if given a valid authorization code, the email address field in the session should be set
        pub async fn handle_login(req: web::Query<MicrosoftLoginCode>, session: Session) -> Result<HttpResponse, Box<dyn std::error::Error>> {
            let code = req.code.clone().unwrap_or_default();
            // TODO: Is it bad to make a new client for every login?
            let client = reqwest::Client::new();
            let secret = get_azure_secret();

            let response: AzureAuthTokenResopnse = client.post(format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", "common")).body(
                format!("code={}&client_id={}&redirect_uri={}&grant_type=authorization_code&client_secret={}",
                    code,
                    CLIENT_ID,
                    REDIRECT_URI,
                    secret
                )
            ).send().await?.json().await?;
            if response.access_token.is_some() {
                let me: AzureMeResponse = client.get("https://graph.microsoft.com/v1.0/me")
                    .header("Content-Type", "application/json")
                    .header("Authorization", response.access_token.unwrap())
                    .send().await?.json().await?;
                session.insert("me", me.clone())?;
                Ok(HttpResponse::Found().append_header(("Location", "/team")).finish())
            }
            else {
                Ok(HttpResponse::Found().append_header(("Location", "/login")).finish())
            }
            //Ok(HttpResponse::Ok().body(response.access_token.unwrap_or(response.error.unwrap_or("wtf".to_string()))))
        }
    }
}
