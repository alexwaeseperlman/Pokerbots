use lazy_static::lazy_static;
use lettre::{
    transport::smtp::{authentication::Credentials, PoolConfig},
    SmtpTransport,
};
use reqwest::Client;
use std::fs;

use lettre::transport::smtp::client::{Tls, TlsParameters};

pub const TEAM_SIZE: usize = 5;

pub fn microsoft_client_id() -> String {
    std::env::var("APP_MICROSOFT_CLIENT_ID").expect("MICROSOFT_CLIENT_ID must be set in .env")
}

pub fn azure_secret() -> String {
    std::env::var("AZURE_SECRET").unwrap_or_else(|_| {
        fs::read_to_string("/run/secrets/azure-secret")
            .expect("AZURE_SECRET must be set in .env or /run/secrets/azure-secret")
    })
}

pub fn microsoft_redirect_uri() -> String {
    std::env::var("APP_MICROSOFT_REDIRECT_URI")
        .expect("APP_MICROSOFT_REDIRECT_URI must be set in .env")
}

pub fn google_client_id() -> String {
    std::env::var("APP_GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set in .env")
}

pub fn google_secret() -> String {
    std::env::var("GOOGLE_SECRET").unwrap_or_else(|_| {
        fs::read_to_string("/run/secrets/google-secret")
            .expect("GOOGLE_SECRET must be set in .env or /run/secrets/google-secret")
    })
}

pub fn google_redirect_uri() -> String {
    std::env::var("APP_GOOGLE_REDIRECT_URI").expect("APP_GOOGLE_REDIRECT_URI must be set in .env")
}

pub fn pfp_s3_bucket() -> String {
    std::env::var("PFP_S3_BUCKET").expect("PFP_S3_BUCKET must be set in .env")
}

pub fn bot_s3_bucket() -> String {
    std::env::var("BOT_S3_BUCKET").expect("BOT_S3_BUCKET must be set in .env")
}

pub fn build_logs_s3_bucket() -> String {
    std::env::var("BUILD_LOGS_S3_BUCKET").expect("BUILD_LOGS_S3_BUCKET must be set in .env")
}

pub fn game_logs_s3_bucket() -> String {
    std::env::var("GAME_LOGS_S3_BUCKET").expect("GAME_LOGS_S3_BUCKET must be set in .env")
}

pub fn bot_size() -> u64 {
    std::env::var("BOT_SIZE")
        .expect("BOT_SIZE must be set in .env")
        .parse()
        .expect("BOT_SIZE must be a number")
}

pub fn pepper() -> String {
    std::env::var("PEPPER").expect("PEPPER must be set in .env")
}

pub fn alias_email() -> String {
    std::env::var("ALIAS_EMAIL").expect("ALIAS_EMAIL must be set in .env")
}

pub fn underlying_email() -> String {
    std::env::var("UNDERLYING_EMAIL").expect("UNDERLYING_EMAIL must be set in .env")
}

pub fn email_app_password() -> String {
    std::env::var("EMAIL_APP_PASSWORD").expect("EMAIL_APP_PASSWORD must be set in .env")
}

pub fn email_verification_link_uri() -> String {
    std::env::var("EMAIL_VERIFICATION_LINK_URI")
        .expect("EMAIL_VERIFICATION_LINK_URI must be set in .env")
}

pub fn password_reset_link_uri() -> String {
    std::env::var("PASSWORD_RESET_LINK_URI").expect("PASSWORD_RESET_LINK_URI must be set in .env")
}

pub fn smtp_server() -> String {
    std::env::var("SMTP_SERVER").expect("SMTP_SERVER must be set in .env")
}

lazy_static! {
    pub static ref EMAIL_EMAIL_VERIFICATION_BODY: String = r#"
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">

<head>
  <meta http-equiv="Content-Type" content="text/html; charset=utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Please activate your account</title>
  <!--[if mso]><style type="text/css">body, table, td, a { font-family: Arial, Helvetica, sans-serif !important; }</style><![endif]-->
</head>

<body style="font-family: Helvetica, Arial, sans-serif; margin: 0px; padding: 0px; background-color: #ffffff;">
  <table role="presentation"
    style="width: 100%; border-collapse: collapse; border: 0px; border-spacing: 0px; font-family: Arial, Helvetica, sans-serif; background-color: rgb(239, 239, 239);">
    <tbody>
      <tr>
        <td align="center" style="padding: 1rem 2rem; vertical-align: top; width: 100%;">
          <table role="presentation" style="max-width: 600px; border-collapse: collapse; border: 0px; border-spacing: 0px; text-align: left;">
            <tbody>
              <tr>
                <td style="padding: 40px 0px 0px;">
                  <div style="padding: 20px; background-color: rgb(255, 255, 255);">
                    <div style="color: rgb(0, 0, 0); text-align: left;">
                      <h1 style="margin: 1rem 0">Email verification</h1>
                      <p style="padding-bottom: 16px">Follow this link to verify your email address.</p>
                      <p style="padding-bottom: 16px"><a href="{}" target="_blank"
                          style="padding: 12px 24px; border-radius: 4px; color: #FFF; background: #2B52F5;display: inline-block;margin: 0.5rem 0;">Verify
                          now</a></p>
                      <p style="padding-bottom: 16px">If you didn't expect this email, reach out to contact@upac.dev</p>
                      <p style="padding-bottom: 16px">Thanks,<br>The UPAC team</p>
                    </div>
                  </div>
                  <div style="padding-top: 20px; color: rgb(153, 153, 153); text-align: center;">
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </td>
      </tr>
    </tbody>
  </table>
</body>

</html>
"#.to_string();

    pub static ref EMAIL_PASSWORD_RESET_BODY: String = r#"
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">

<head>
  <meta http-equiv="Content-Type" content="text/html; charset=utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Please activate your account</title>
  <!--[if mso]><style type="text/css">body, table, td, a { font-family: Arial, Helvetica, sans-serif !important; }</style><![endif]-->
</head>

<body style="font-family: Helvetica, Arial, sans-serif; margin: 0px; padding: 0px; background-color: #ffffff;">
  <table role="presentation"
    style="width: 100%; border-collapse: collapse; border: 0px; border-spacing: 0px; font-family: Arial, Helvetica, sans-serif; background-color: rgb(239, 239, 239);">
    <tbody>
      <tr>
        <td align="center" style="padding: 1rem 2rem; vertical-align: top; width: 100%;">
          <table role="presentation" style="max-width: 600px; border-collapse: collapse; border: 0px; border-spacing: 0px; text-align: left;">
            <tbody>
              <tr>
                <td style="padding: 40px 0px 0px;">
                  <div style="padding: 20px; background-color: rgb(255, 255, 255);">
                    <div style="color: rgb(0, 0, 0); text-align: left;">
                      <h1 style="margin: 1rem 0">Email verification</h1>
                      <p style="padding-bottom: 16px">Follow this link to reset your password.</p>
                      <p style="padding-bottom: 16px"><a href="{}" target="_blank"
                          style="padding: 12px 24px; border-radius: 4px; color: #FFF; background: #2B52F5;display: inline-block;margin: 0.5rem 0;">Reset Password</a></p>
                      <p style="padding-bottom: 16px">If you didn't expect this email, reach out to contact@upac.dev</p>
                      <p style="padding-bottom: 16px">Thanks,<br>The UPAC team</p>
                    </div>
                  </div>
                  <div style="padding-top: 20px; color: rgb(153, 153, 153); text-align: center;">
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </td>
      </tr>
    </tbody>
  </table>
</body>

</html>
    "#.to_string();

     pub static ref MAILER: SmtpTransport = SmtpTransport::relay(&smtp_server()).unwrap()
        .credentials(Credentials::new(
            underlying_email().to_string(),
            email_app_password().to_string(),
        ))
        .tls(Tls::Wrapper(
                TlsParameters::new(smtp_server().to_string()).unwrap()
        ))
        .pool_config(PoolConfig::new())
        .build();

    pub static ref CLIENT: Client = reqwest::Client::new();
}
