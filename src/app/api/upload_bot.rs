use actix_multipart::Multipart;
use actix_session::Session;
use actix_web::{post, web, HttpResponse};
use futures::{StreamExt, TryStreamExt};

use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

use crate::app::{bots::Bot, login};

pub struct BotsList {
    pub bots: std::sync::Mutex<Vec<Bot>>,
}
#[post("/api/upload-bot")]
pub async fn upload_bot(
    session: Session,
    data: web::Data<BotsList>,
    mut payload: Multipart,
) -> actix_web::Result<HttpResponse> {
    let team_name = {
        let team = login::get_team_data(&session);
        team.unwrap().team_name
    };
    let team_path = PathBuf::from(format!("/tmp/pokerzero/{}", team_name));

    let mut zip_file = {
        if !team_path.exists() {
            fs::create_dir_all(&team_path)?;
        }
        fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(team_path.join("bot.zip"))?
    };

    while let Ok(Some(mut field)) = payload.try_next().await {
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            zip_file = zip_file.write_all(&data).map(|_| zip_file)?;
        }
    }

    // TODO: SHOULD WE web::block these?
    let vals: serde_yaml::Value = {
        let mut archive = zip::ZipArchive::new(zip_file).map_err(io::Error::from)?;
        archive.extract(&team_path).map_err(io::Error::from)?;
        let yaml_file = match std::fs::File::open(team_path.join("bot").join("cmd.yaml")) {
            Ok(f) => f,
            Err(e) => match e.kind() {
                io::ErrorKind::NotFound => fs::File::open(team_path.join("bot").join("cmd.yml"))?,
                _ => return Err(e).map_err(actix_web::Error::from),
            },
        };
        serde_yaml::from_reader(yaml_file).map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Yaml parsing error: {}", e))
        })?
    };

    let bot = Bot::new(
        team_name,
        // team.unwrap().team_name,
        team_path,
        vals.get("build")
            .map_or(None, |v| v.as_str())
            .map(String::from),
        vals.get("run")
            .map_or(None, |v| v.as_str())
            .map(String::from),
    );
    let bot_ = bot.clone();
    // TODO: MAKE IT REPLACE OLD BOT RATHER THAN PUSH
    data.bots.lock().unwrap().push(bot_);
    bot.play(&data.bots.lock().unwrap()).await?;

    Ok(HttpResponse::Ok().body("Successfully uploaded bot"))
}
