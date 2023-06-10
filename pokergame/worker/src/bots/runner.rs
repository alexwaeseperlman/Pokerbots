use std::{
    io::{self, Read, Write},
    path::PathBuf,
    process::{exit, Stdio},
};
use tokio::{
    net::UnixStream,
    process::{Child, Command},
};

use crate::bots::bot::languages;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn run_bot(
    bot_folder: PathBuf,
    configure: fn(command: &mut Command) -> &mut Command,
) -> io::Result<Child> {
    let cwd = std::env::current_dir()?;
    std::env::set_current_dir(&bot_folder)?;
    log::debug!("Running bot in {:?}", std::env::current_dir()?);
    //TODO: run this in a cgroup this to protect against zip bombs
    tokio::process::Command::new("unzip")
        .arg("bot.zip")
        .spawn()?
        .wait()
        .await?;

    std::env::set_current_dir("./bot")?;
    let bot = crate::bots::bot::Bot::new(std::path::PathBuf::from(".")).await?;
    // serialize errors
    // TODO: Sandbox all of this since it is untrusted code
    bot.language.build()?;
    let proc = bot.language.run(configure)?;

    std::env::set_current_dir(cwd)?;
    Ok(proc)
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;

    #[tokio::test]
    // Unzips the bot.zip file and runs the bot in a tmp dir
    pub async fn python_ping() {
        let cwd = std::env::current_dir().unwrap();
        let test_id = format!("{:x}", rand::thread_rng().gen::<u32>());
        tokio::fs::create_dir(format!("/tmp/{}", test_id))
            .await
            .unwrap();
        tokio::fs::copy(
            "./runner_tests/python_ping/bot.zip",
            format!("/tmp/{}/bot.zip", test_id),
        )
        .await
        .unwrap();
        let mut proc = run_bot(format!("/tmp/{}", test_id).into(), |command| {
            command.stdin(Stdio::piped()).stdout(Stdio::piped())
        })
        .await
        .unwrap();
        let mut stdout = proc.stdout.take().unwrap();
        let mut stdin = proc.stdin.take().unwrap();
        stdin.write(b"ping\n").await.unwrap();
        let mut resp: [u8; 4] = [0; 4];
        let target: [u8; 4] = b"pong".to_owned().into();
        stdout.read(&mut resp).await.unwrap();
        stdin.shutdown().await.unwrap();
        proc.kill().await.unwrap();
        for i in 0..4 {
            assert_eq!(resp[i], target[i]);
        }
        assert_eq!(cwd, std::env::current_dir().unwrap());
    }
}
