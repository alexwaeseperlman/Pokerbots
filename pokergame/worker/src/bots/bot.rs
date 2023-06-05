pub mod languages;

pub struct Bot {
    path: PathBuf,
    language: Box<dyn languages::Language>,
}

impl Bot {
    pub fn new(path: PathBuf) -> Result<Bot, Box<dyn std::error::Error>> {
        std::env::set_current_dir(&path)?;
        // We leave the bot.zip in the directory cause why not
        Command::new("unzip")
            .arg("-o")
            .arg("bot.zip")
            .spawn()?
            .wait()?;
        // Read language from bot.json
        let language = match fs::read_to_string(path.join("bot.json")) {
            Ok(s) => {
                let json: serde_json::Value = serde_json::from_str(&s).unwrap();
                let language = json["build"].as_str().unwrap();
                detect_language(language)
            }
            Err(e) => panic!("Failed to read bot.json: {}", e),
        };
        Ok(Self { path, language })
    }

    /// builds files necesary for bot in subprocess
    pub fn build(&self) -> languages::BuildResult {
        if std::env::set_current_dir(&self.path).is_err() {
            log::error!("Unable to set current directory to {}", self.path.display());
            return languages::BuildResult::Failure;
        };
        self.language.build()
    }

    /// runs the bot in subprocess
    pub fn run(&self) -> Result<Child, RunResult> {
        if std::env::set_current_dir(&self.path).is_err() {
            log::error!("Unable to set current directory to {}", self.path.display());
            return Err(RunResult::Failure);
        };
        self.language
            .run(|command| command.stdin(Stdio::piped()).stdout(Stdio::piped()))
            .map_err(|e| {
                log::error!("Unable to run bot: {}", e);
                RunResult::Failure
            })
    }
}

use std::{
    fs, os,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use shared::GameError;

use self::languages::{detect_language, RunResult};

pub async fn download_bot(
    key: &str,
    path: &PathBuf,
    bot_bucket: &str,
    client: aws_sdk_s3::Client,
) -> Result<(), GameError> {
    //TODO: download this in a better way
    if let Ok(res) = client.get_object().bucket(bot_bucket).key(key).send().await {
        if let Ok(body) = res.body.collect().await {
            let bytes = body.into_bytes();
            return fs::write(path.join("bot.zip"), bytes)
                .map_err(|e| GameError::InternalError(format!("Unable to write bot.zip: {}", e)));
        }
    }
    Err(shared::GameError::InternalError(
        "Unable to get bot from s3".to_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use std::{
        cell::LazyCell,
        fs,
        sync::{Mutex, OnceLock},
    };

    static TEST_BUCKET: &str = "pokerbots-test-bucket";
    pub async fn setup() {
        let aws_config = aws_config::load_from_env().await;
        let client = aws_sdk_s3::Client::new(&aws_config);
        client
            .create_bucket()
            .bucket(TEST_BUCKET)
            .send()
            .await
            .unwrap();
        client
            .put_object()
            .bucket(TEST_BUCKET)
            .key("error_bot.zip")
            .body(
                std::fs::read("../../example-bots/error_bot.zip")
                    .unwrap()
                    .into(),
            )
            .send()
            .await
            .unwrap();
        let path = std::path::Path::new("/tmp/pokerbots_test").to_path_buf();
        fs::create_dir(&path).unwrap_or(());
        fs::remove_file(path.join("bot.zip")).unwrap_or(());
    }
    pub async fn teardown() {
        let aws_config = aws_config::load_from_env().await;
        let client = aws_sdk_s3::Client::new(&aws_config);
        client
            .delete_bucket()
            .bucket(TEST_BUCKET)
            .send()
            .await
            .unwrap();
    }

    /*#[tokio::test]
    async fn download() {
        setup().await;
        let aws_config = aws_config::load_from_env().await;
        let client = aws_sdk_s3::Client::new(&aws_config);
        let key = "error_bot.zip".to_owned();
        let path = std::path::Path::new("/tmp/pokerbots_test").to_path_buf();
        fs::create_dir(&path).unwrap_or(());
        fs::remove_file(path.join("bot.zip")).unwrap_or(());
        let res = super::download_bot(&key, path.clone(), TEST_BUCKET, client).await;
        res.unwrap();
        assert!(path.join("bot.zip").exists());
        teardown().await;
    }*/

    #[tokio::test]
    async fn make_bot() {
        let path = std::path::Path::new("/tmp/pokerbots_test").to_path_buf();
        fs::remove_dir_all(&path).unwrap_or(());
        fs::create_dir(&path).unwrap_or(());
        fs::remove_file(path.join("bot.zip")).unwrap_or(());
        fs::copy("../../example-bots/error_bot.zip", path.join("bot.zip")).unwrap();
        let bot = super::Bot::new(path.clone()).unwrap();
        bot.build();
        bot.run().unwrap().wait().unwrap();
        assert!(path.join("bot.zip").exists());
    }
}
