use shared::db::{
    models::{Auth, NewUser},
    schema::auth,
};

use crate::app::api;

use super::*;

#[derive(Deserialize)]
pub struct MicrosoftLoginCode {
    code: Option<String>,
}
#[get("/oauth/microsoft/login")]
pub async fn microsoft_login(
    web::Query::<MicrosoftLoginCode>(MicrosoftLoginCode { code }): web::Query<MicrosoftLoginCode>,
    session: Session,
) -> ApiResult<()> {
    // retrieve access token
    #[derive(Deserialize)]
    struct AzureAuthTokenResponse {
        access_token: Option<String>,
    }
    let body = format!(
        "code={}&client_id={}&redirect_uri={}&grant_type=authorization_code&client_secret={}",
        code.ok_or(ApiError {
            status_code: StatusCode::BAD_REQUEST,
            message: "No code provided".to_string(),
        })?,
        config::microsoft_client_id(),
        url::form_urlencoded::byte_serialize(config::microsoft_redirect_uri().as_bytes())
            .collect::<String>(),
        config::azure_secret()
    );
    let response = config::CLIENT
        .post("https://login.microsoftonline.com/common/oauth2/v2.0/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?;

    let response: AzureAuthTokenResponse = response.json().await?;

    if response.access_token.is_none() {
        return Err(ApiError {
            status_code: StatusCode::UNAUTHORIZED,
            message: "No access token received".to_string(),
        });
    }

    // exchange access token for user info
    #[derive(Deserialize, Debug)]
    pub struct AzureMeResponse {
        pub displayName: Option<String>,
        pub givenName: Option<String>,
        pub mail: Option<String>,
        pub userPrincipalName: Option<String>,
        pub id: Option<String>,
    }
    let me: AzureMeResponse = config::CLIENT
        .get("https://graph.microsoft.com/v1.0/me")
        .header("Content-Type", "application/json")
        .header("Authorization", response.access_token.unwrap())
        .send()
        .await?
        .json()
        .await?;

    // TODO: use mail or UPN?
    // TODO: When you sign in with some microsoft accounts, there is no email
    // but there is a userPrincipalName. We should confirm that it is ok
    // to use userPrincipalName when email doesn't exist (this is mainly
    // used to verify that a user is from an allowed organization)

    log::info!("{:?}", me);
    if me.userPrincipalName.is_none() {
        return Err(ApiError {
            status_code: StatusCode::UNAUTHORIZED,
            message: "No UPN on selected account".to_string(),
        });
    }

    // TODO: make transaction
    diesel::insert_into(auth::dsl::auth)
        .values(&Auth {
            email: me.userPrincipalName.clone().unwrap(),
            mangled_password: None,
            email_verification_link: None,
            email_verification_link_expiration: None,
            password_reset_link: None,
            password_reset_link_expiration: None,
            email_confirmed: false,
            is_admin: false,
        })
        .on_conflict(auth::dsl::email)
        .do_nothing()
        .execute(&mut (*DB_CONNECTION).get().unwrap())?;

    diesel::insert_into(users::dsl::users)
        .values(NewUser {
            email: me.userPrincipalName.clone().unwrap(),
            display_name: me
                .displayName
                .unwrap_or(me.userPrincipalName.clone().unwrap()),
            email_hash: api::auth::mangle(&me.userPrincipalName.clone().unwrap())?,
        })
        .on_conflict(users::dsl::email)
        .do_nothing()
        .execute(&mut (*DB_CONNECTION).get().unwrap())?;

    session.insert("email", me.userPrincipalName.unwrap())?;

    Ok(web::Json(()))
}

#[derive(Deserialize)]
pub struct GoogleLoginCode {
    code: Option<String>,
}
#[get("/oauth/google/login")]
async fn google_login(
    web::Query::<GoogleLoginCode>(GoogleLoginCode { code }): web::Query<GoogleLoginCode>,
    session: Session,
) -> ApiResult<()> {
    // retrieve access token
    #[derive(Deserialize, Debug)]
    struct GoogleAuthTokenResponse {
        access_token: Option<String>,
    }
    let response: GoogleAuthTokenResponse = config::CLIENT
        .post("https://oauth2.googleapis.com/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            (
                "code",
                code.ok_or(ApiError {
                    status_code: StatusCode::BAD_REQUEST,
                    message: "No code provided".to_string(),
                })?,
            ),
            ("client_id", config::google_client_id().to_string()),
            ("redirect_uri", config::google_redirect_uri().to_string()),
            ("grant_type", "authorization_code".to_string()),
            ("client_secret", config::google_secret().to_string()),
        ])
        .send()
        .await?
        .json()
        .await?;

    if response.access_token.is_none() {
        return Err(ApiError {
            status_code: StatusCode::UNAUTHORIZED,
            message: "Invalid response from Google OAuth".to_string(),
        });
    }

    // exchange access token for user info
    #[derive(Deserialize, Debug)]
    pub struct GoogleUserInfoResponse {
        pub name: Option<String>,
        pub email: Option<String>,
        pub email_verified: Option<bool>,
    }
    let user_info: GoogleUserInfoResponse = config::CLIENT
        .get(format!(
            "https://www.googleapis.com/oauth2/v3/userinfo?access_token={}",
            response.access_token.as_ref().unwrap()
        ))
        .header("Content-Type", "application/json")
        .send()
        .await?
        .json()
        .await?;

    println!("{:?}", user_info);

    if user_info.email.is_none() || user_info.email_verified.unwrap_or(false) == false {
        return Err(ApiError {
            status_code: StatusCode::UNAUTHORIZED,
            message: "Invalid response from Google OAuth".to_string(),
        });
    }

    // TODO: make transaction
    diesel::insert_into(auth::dsl::auth)
        .values(&Auth {
            email: user_info.email.clone().unwrap(),
            mangled_password: None,
            email_verification_link: None,
            email_verification_link_expiration: None,
            password_reset_link: None,
            password_reset_link_expiration: None,
            email_confirmed: false,
            is_admin: false,
        })
        .on_conflict(auth::dsl::email)
        .do_nothing()
        .execute(&mut (*DB_CONNECTION).get().unwrap())?;

    diesel::insert_into(users::dsl::users)
        .values(NewUser {
            email: user_info.email.clone().unwrap(),
            display_name: user_info.name.unwrap_or(user_info.email.clone().unwrap()),
            email_hash: api::auth::mangle(&user_info.email.clone().unwrap())?,
        })
        .on_conflict(users::dsl::email)
        .do_nothing()
        .execute(&mut (*DB_CONNECTION).get().unwrap())?;

    session.insert("email", user_info.email.unwrap())?;

    Ok(web::Json(()))
}
