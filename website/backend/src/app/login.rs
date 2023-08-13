use actix_session::Session;
use actix_web::{get, web, HttpResponse};
use diesel::prelude::*;
use log::error;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::config::{CLIENT_ID, REDIRECT_URI};
use shared::db::conn::DB_CONNECTION;

use shared::db::{
    models::{NewUser, Team, TeamInvite, User},
    schema::{team_invites, teams, users},
};

#[derive(Deserialize, Serialize, Clone, Debug, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
pub struct UserData {
    pub email: String,
    pub display_name: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
pub struct TeamData {
    pub id: i32,
    pub team_name: String,
    pub members: Vec<UserData>,
    pub owner: String,
    pub score: Option<i32>,
    pub invites: Vec<String>,
    pub active_bot: Option<i32>,
}

// TODO: right now state is just the return address,
// but we should do more with it in the future
// e.g. store a code in the database with the
// return address and other relevant info
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct HandleLoginQuery {
    pub state: Option<String>,
}

pub fn url_encode(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect::<String>()
}

pub fn microsoft_login_url(return_to: &str) -> String {
    format!(
        "https://login.microsoftonline.com/{}/oauth2/\
    v2.0/authorize?client_id={}&response_type=code&redirect_uri={}\
    &response_mode=query&scope=User.Read&state={}&prompt=select_account",
        "common",
        CLIENT_ID.to_string(),
        url_encode(&REDIRECT_URI),
        url_encode(return_to)
    )
}
/*
These have camel case names so they can correspond to
the fields in the JSON response from the Microsoft Graph API
*/
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

    let invites: Vec<String> = team_invites::dsl::team_invites
        .filter(team_invites::dsl::teamid.eq(t.id))
        .load::<TeamInvite>(conn)
        .expect("Unable to load team invites")
        .into_iter()
        .map(|invite: TeamInvite| invite.invite_code)
        .collect();

    Some(TeamData {
        id: t.id,
        team_name: t.team_name.clone(),
        members,
        owner: t.owner.clone(),
        score: t.score,
        invites,
        active_bot: t.active_bot,
    })
}
// By the end of this method, if given a valid authorization code, the email address field in the session should be set
pub async fn handle_login(
    req: web::Query<MicrosoftLoginCode>,
    session: Session,
    web::Query::<HandleLoginQuery>(HandleLoginQuery { state }): web::Query<HandleLoginQuery>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let code = req.code.clone().unwrap_or_default();
    // TODO: Is it bad to make a new client for every login?
    // => Yes, it is.
    let client = reqwest::Client::new();

    let response: AzureAuthTokenResopnse = client
        .post(format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            "common"
        ))
        .body(format!(
            "code={}&client_id={}&redirect_uri={}&grant_type=authorization_code&client_secret={}",
            code,
            CLIENT_ID.to_string(),
            url_encode(&REDIRECT_URI),
            &*crate::config::AZURE_SECRET
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
        }
        Ok(HttpResponse::Found()
            .append_header(("Location", state.unwrap_or("/manage-team".to_string())))
            .finish())
    } else {
        Ok(HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish())
    }
}

#[derive(Deserialize)]
pub struct LoginProvider {
    provider: String,
    state: Option<String>,
}
//TODO: instead of state being a parameter here, we should have state in the session
// Should we?
#[get("/api/login-provider")]
pub async fn login_provider(
    web::Query::<LoginProvider>(LoginProvider { provider, state }): web::Query<LoginProvider>,
) -> actix_web::Result<HttpResponse> {
    match provider.as_str() {
        "microsoft" => {
            let url = microsoft_login_url(&state.unwrap_or("/manage-team".to_string()));
            Ok(HttpResponse::Found()
                .append_header(("Location", url))
                .finish())
        }
        _ => Ok(HttpResponse::NotFound().finish()),
    }
}
