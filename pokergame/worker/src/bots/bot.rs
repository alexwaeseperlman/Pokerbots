pub mod languages;

use tokio::{
    io,
    process::{Child, ChildStdin, ChildStdout, Command},
    sync::Mutex,
    task::JoinHandle,
};

use std::{
    fs,
    os::{self, unix::process::ExitStatusExt},
    path::PathBuf,
    process::{ExitStatus, Stdio},
    sync::Arc,
};

use shared::{GameError, WhichBot};

use crate::bots::bot;

use self::languages::detect_language;

#[derive(Debug)]
pub struct Bot {
    pub status: Arc<Mutex<Option<ExitStatus>>>,
    pub output: tokio::io::BufReader<ChildStdout>,
    pub input: tokio::io::BufWriter<ChildStdin>,
    proc: Arc<Mutex<Child>>,
    listener: JoinHandle<()>,
}

impl Bot {
    pub async fn new(
        bot_folder: PathBuf,
        configure: fn(command: &mut Command) -> &mut Command,
        which_bot: WhichBot,
    ) -> Result<Bot, GameError> {
        let cwd = std::env::current_dir()?;
        std::env::set_current_dir(&bot_folder)?;
        log::debug!("Constructing bot in {:?}", std::env::current_dir()?);
        //TODO: run this in a cgroup this to protect against zip bombs
        // We leave the bot.zip in the directory cause why not
        tokio::process::Command::new("unzip")
            .arg("-o")
            .arg("bot.zip")
            .spawn()?
            .wait()
            .await?;

        // This fails if the bot folder is not named bot.
        // This should be guaranteed by the server.
        std::env::set_current_dir("./bot")?;

        // Read language from bot.json
        let language = match fs::read_to_string("bot.json") {
            Ok(s) => {
                let json: serde_json::Value = serde_json::from_str(&s).map_err(|e| {
                    GameError::CompileError(format!("Failed to parse bot.json: {}", e), which_bot)
                })?;
                let language = json["build"].as_str().ok_or(GameError::CompileError(
                    "Failed to read language from bot.json.".into(),
                    which_bot,
                ))?;
                detect_language(language).ok_or(GameError::CompileError(
                    format!("{} is not a supported language.", json["build"]),
                    which_bot,
                ))?
            }
            Err(e) => panic!("Failed to read bot.json: {}", e),
        };
        let build_result = language
            .build()
            .await
            .map_err(|e| GameError::CompileError(e.to_string(), which_bot))?;

        if !build_result.success() {
            return Err(GameError::CompileError(
                "Failed to build bot.".into(),
                which_bot,
            ));
        }

        let proc = Arc::new(Mutex::new(language.run(configure)?));
        let p = proc.clone();
        let mut p = p.lock().await;
        let status: Arc<Mutex<Option<ExitStatus>>> = Arc::new(Mutex::new(None));
        let bot = Bot {
            status: status.clone(),
            output: tokio::io::BufReader::new(
                p.stdout
                    .take()
                    .ok_or(GameError::InternalError("Unable to get stdout".into()))?,
            ),
            input: tokio::io::BufWriter::new(
                p.stdin
                    .take()
                    .ok_or(GameError::InternalError("Unable to get stdin".into()))?,
            ),
            proc: proc.clone(),
            listener: tokio::spawn(async move {
                let proc = proc.clone();
                let mut proc = proc.lock().await;
                let status = status.clone();
                let mut status = status.lock().await;
                *status = Some(proc.wait().await.unwrap_or(ExitStatus::from_raw(1)));
            }),
        };
        Ok(bot)
    }
    pub async fn kill(&mut self) -> io::Result<()> {
        self.proc.lock().await.kill().await
    }
}

impl Drop for Bot {
    // Kill the bot if it is still running
    fn drop(&mut self) {
        let proc = self.proc.clone();
        tokio::spawn(async move {
            let mut proc = proc.lock().await;
            if let Err(e) = proc.kill().await {
                log::error!("Failed to kill bot: {}", e);
            }
        });
    }
}

pub async fn download_bot(
    key: &str,
    path: &PathBuf,
    bot_bucket: &str,
    client: aws_sdk_s3::Client,
) -> Result<(), GameError> {
    //TODO: download this in a better way
    log::debug!("Downloading bot {} from s3", key);
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
    use std::{assert_matches::assert_matches, process::Stdio};

    use rand::Rng;
    use shared::{GameError, WhichBot};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

    use crate::bots::bot::Bot;

    #[tokio::test]
    // Unzips the bot.zip file and runs the bot in a tmp dir
    pub async fn python_ping() {
        let test_id = format!("{:x}", rand::thread_rng().gen::<u32>());
        println!("Test id: {}", test_id);
        tokio::fs::create_dir(format!("/tmp/{}", test_id))
            .await
            .unwrap();
        tokio::fs::copy(
            "./runner_tests/python_ping/bot.zip",
            format!("/tmp/{}/bot.zip", test_id),
        )
        .await
        .unwrap();
        let mut bot = Bot::new(
            format!("/tmp/{}", test_id).into(),
            |command| command.stdin(Stdio::piped()).stdout(Stdio::piped()),
            WhichBot::BotA,
        )
        .await
        .expect("Failed to create bot");
        bot.input.write(b"ping\n").await.unwrap();
        bot.input.flush().await.unwrap();
        let mut str: String = String::new();
        bot.output.read_line(&mut str).await.unwrap();
        assert_eq!(str, "pong\n");
    }

    #[tokio::test]
    // Unzips the bot.zip file and runs the bot in a tmp dir
    pub async fn cpp_compile_error() {
        let test_id = format!("{:x}", rand::thread_rng().gen::<u32>());
        println!("Test id: {}", test_id);
        tokio::fs::create_dir(format!("/tmp/{}", test_id))
            .await
            .unwrap();
        tokio::fs::copy(
            "./runner_tests/cpp_compile_error/bot.zip",
            format!("/tmp/{}/bot.zip", test_id),
        )
        .await
        .unwrap();
        let mut bot = Bot::new(
            format!("/tmp/{}", test_id).into(),
            |command| command.stdin(Stdio::piped()).stdout(Stdio::piped()),
            WhichBot::BotA,
        )
        .await;
        match bot {
            Ok(mut b) => {
                panic!("Expected compile error, got {:?}", b)
            }
            Err(e) => assert_matches!(e, GameError::CompileError(_, WhichBot::BotA)),
        }
    }
}
