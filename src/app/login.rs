use actix_session::Session;
use actix_web::{middleware::Logger, web, HttpResponse};
use diesel::prelude::*;
use log::error;
use serde::{Deserialize, Serialize};
use std::env;

use crate::{
    config::{CLIENT_ID, DB_CONNECTION, REDIRECT_URI},
    models::{NewUser, Team, User},
    schema::{teams, users},
};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UserData {
    pub email: String,
    pub display_name: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TeamData {
    pub id: i32,
    pub team_name: String,
    pub members: Vec<UserData>,
    pub owner: String,
}

pub fn microsoft_login_url() -> String {
    format!("https://login.microsoftonline.com/{}/oauth2/v2.0/authorize?client_id={}&response_type=code&redirect_uri={}&response_mode=query&scope=User.Read", "common", CLIENT_ID, REDIRECT_URI)
}

pub fn get_azure_secret() -> String {
    env::var("AZURE_SECRET").expect("AZURE_SECRET must be set in .env")
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AzureMeResponse {
    pub displayName: Option<String>,
    pub givenName: Option<String>,
    pub mail: Option<String>,
    pub userPrincipalName: Option<String>,
    pub id: Option<String>,
}

#[derive(Deserialize)]
pub struct MicrosoftLoginCode {
    code: Option<String>,
}

// https://learn.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow
#[derive(Deserialize)]
struct AzureAuthTokenResopnse {
    access_token: Option<String>,
}

pub fn get_user_data(session: &Session) -> Option<UserData> {
    let me: UserData = session.get("me").ok()??;
    // If the user is logged in and we don't have an entry for them in the database, add them
    if let Err(err) = diesel::insert_into(users::dsl::users)
        .values(NewUser {
            email: me.email.clone(),
            display_name: me.display_name.clone(),
        })
        .on_conflict_do_nothing()
        .execute(&mut (*DB_CONNECTION).get().unwrap())
    {
        error!("{:?}", err);
        return None;
    };
    Some(me)
}

pub fn get_team_data(session: &Session) -> Option<TeamData> {
    let user_data = get_user_data(session);
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    let us = users::dsl::users
        .filter(users::dsl::email.eq(user_data?.email))
        .limit(1)
        .load::<User>(conn)
        .expect("Unable to load user");
    let u = us.get(0)?;

    let ts: Vec<Team> = teams::dsl::teams
        .find(u.team_id?)
        .load::<Team>(conn)
        .expect("Unable to load team");
    let t = ts.get(0)?;

    let members: Vec<UserData> = users::dsl::users
        .filter(users::dsl::team_id.eq(t.id))
        .load::<User>(conn)
        .expect("Unable to load users in team")
        .into_iter()
        .map(|user| UserData {
            display_name: user.display_name,
            email: user.email,
        })
        .collect();

    Some(TeamData {
        id: t.id,
        team_name: t.team_name.clone(),
        members,
        owner: t.owner.clone(),
    })
}
//
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

    if let Some(token) = response.access_token {
        let me: AzureMeResponse = client
            .get("https://graph.microsoft.com/v1.0/me")
            .header("Content-Type", "application/json")
            .header("Authorization", token)
            .send()
            .await?
            .json()
            .await?;
        // TODO: When you sign in with some microsoft accounts, there is no email
        // but there is a userPrincipalName. We should confirm that it is ok
        // to use userPrincipalName when email doesn't exist (this is mainly
        // used to verify that a user is from an allowed organization)
        if let Some(mail) = me.userPrincipalName.clone() {
            session.insert(
                "me",
                UserData {
                    email: mail.clone(),
                    display_name: me
                        .displayName
                        .clone()
                        .unwrap_or_else(|| me.givenName.unwrap_or_else(|| mail.clone())),
                },
            )?;
        } else {
            session.insert("message", "There was an issue logging you in.")?;
        }
        Ok(HttpResponse::Found()
            .append_header(("Location", "/manage-team"))
            .finish())
    } else {
        Ok(HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish())
    }
}
