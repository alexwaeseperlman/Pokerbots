use super::super::app_config::*;
use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::models::{Team, User};
use crate::schema::users;
use crate::{app_config::TENANT_ID, UserData};

pub fn microsoft_login_url() -> String {
    format!("https://login.microsoftonline.com/{}/oauth2/v2.0/authorize?client_id={}&response_type=code&redirect_uri={}&response_mode=query&scope=User.Read", "common", CLIENT_ID, REDIRECT_URI)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AzureMeResponse {
    pub displayName: Option<String>,
    pub givenName: Option<String>,
    pub mail: Option<String>,
    pub userPrincipalName: Option<String>,
    pub id: Option<String>,
}

use crate::{get_azure_secret, TeamData};
use actix_web::{get, web, HttpRequest, HttpResponse};

use actix_session::Session;

#[derive(Deserialize)]
pub struct MicrosoftLoginCode {
    code: Option<String>,
}

// https://learn.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow
#[derive(Deserialize)]
struct AzureAuthTokenResopnse {
    access_token: Option<String>,
    error: Option<String>,
}

pub fn get_user_data(session: Option<Session>) -> Option<UserData> {
    if session.is_some()
        && session.clone().unwrap().get::<UserData>("me").is_ok()
        && session
            .clone()
            .unwrap()
            .get::<UserData>("me")
            .unwrap()
            .is_some()
    {
        let me: UserData = session.unwrap().get("me").unwrap().unwrap();
        return Some(me);
    }
    None
}

pub fn get_team_data(session: Option<Session>) -> Option<TeamData> {
    use crate::schema::teams;
    use crate::schema::users;
    use crate::DB_CONNECTION;
    use diesel::*;
    use std::iter;
    let data = get_user_data(session.clone());
    if data.is_none() {
        return None;
    }
    let connection = &mut (*DB_CONNECTION).get().unwrap();
    let us = users::dsl::users
        .filter(users::dsl::email.eq(data.clone().unwrap().email))
        .limit(1)
        .load::<User>(connection)
        .expect("Unable to load user");

    if us.len() == 0 {
        return None;
    }

    let u = us.get(0).unwrap();

    if u.team.is_none() {
        return None;
    }

    let ts: Vec<Team> = teams::dsl::teams
        .find(u.team.unwrap())
        .load::<Team>(connection)
        .expect("Unable to load team");
    if ts.len() == 0 {
        return None;
    }
    let t = ts.get(0).unwrap();

    let members: Vec<UserData> = users::dsl::users
        .filter(users::dsl::teamid.eq(t.id))
        .load::<User>(connection)
        .expect("Unable to load users in team")
        .into_iter()
        .map(|user| UserData {
            displayName: user.displayName,
            email: user.email,
        })
        .collect();
    Some(TeamData {
        name: t.teamname.clone(),
        members: members,
    })
}
// By the end of this method, if given a valid authorization code, the email address field in the session should be set
pub async fn handle_login(
    req: web::Query<MicrosoftLoginCode>,
    session: Session,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let code = req.code.clone().unwrap_or_default();
    // TODO: Is it bad to make a new client for every login?
    let client = reqwest::Client::new();
    let secret = get_azure_secret();

    let response: AzureAuthTokenResopnse = client
        .post(format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            "common"
        ))
        .body(format!(
            "code={}&client_id={}&redirect_uri={}&grant_type=authorization_code&client_secret={}",
            code, CLIENT_ID, REDIRECT_URI, secret
        ))
        .send()
        .await?
        .json()
        .await?;
    if response.access_token.is_some() {
        let me: AzureMeResponse = client
            .get("https://graph.microsoft.com/v1.0/me")
            .header("Content-Type", "application/json")
            .header("Authorization", response.access_token.unwrap())
            .send()
            .await?
            .json()
            .await?;
        if me.mail.is_some() {
            session.insert(
                "me",
                UserData {
                    email: me.mail.clone().unwrap(),
                    displayName: me
                        .displayName
                        .unwrap_or(me.givenName.unwrap_or(me.mail.unwrap())),
                },
            )?;
        }
        Ok(HttpResponse::Found()
            .append_header(("Location", "/team"))
            .finish())
    } else {
        Ok(HttpResponse::Found()
            .append_header(("Location", "/login"))
            .finish())
    }
}
