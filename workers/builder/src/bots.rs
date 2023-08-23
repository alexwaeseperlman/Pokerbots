use subprocess::PopenConfig;
use tokio::{fs, io};

use std::path::{Path, PathBuf};

use shared::BotJson;

pub async fn build_bot<T: AsRef<Path>>(bot_folder: T) -> Result<(), io::Error> {
    //TODO: run this in a cgroup this to protect against zip bombs
    let path = bot_folder.as_ref();
    log::info!("Constructing bot in {:?}", path);

    subprocess::Popen::create(
        &["unzip", "-o", "bot.zip"],
        PopenConfig {
            cwd: Some(path.into()),
            stderr: subprocess::Redirection::Merge,
            stdout: subprocess::Redirection::File(std::fs::File::create(path.join("logs"))?),
            ..Default::default()
        },
    )
    .and_then(|mut p| p.wait())
    .map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to unzip bot: {:?}", e),
        )
    })?;

    // This fails if the bot folder is not named bot.
    // This should be guaranteed by the server.

    // Read build command from bot.json
    let bot_json: BotJson = {
        let json = fs::read_to_string(path.join("bot/bot.json")).await?;
        serde_json::from_str::<BotJson>(&json)?
    };

    std::fs::write(
        path.join("bot/build.sh"),
        bot_json.build.unwrap_or_default(),
    )
    .expect("write to build.sh failed");
    let chown_result = subprocess::Popen::create(
        &["chown", "-R", "builder:builder", "."],
        PopenConfig {
            cwd: Some(path.join("bot").into()),
            stderr: subprocess::Redirection::Merge,
            ..Default::default()
        },
    )
    .and_then(|mut p| p.wait());

    let _ = match chown_result {
        Ok(status) if status.success() => Ok(()),
        _ => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Build failed: {:?}", chown_result),
        )),
    };
    let chmod_result = subprocess::Popen::create(
        &["chmod", "+x", "build.sh", "."],
        PopenConfig {
            cwd: Some(path.join("bot").into()),
            stderr: subprocess::Redirection::Merge,
            ..Default::default()
        },
    )
    .and_then(|mut p| p.wait());
    let _ = match chmod_result {
        Ok(status) if status.success() => Ok(()),
        _ => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Build failed: {:?}", chmod_result),
        )),
    };

    let build_result = subprocess::Popen::create(
        &["su", "builder", "-c","bwrap --unshare-all --die-with-parent --dir /tmp --ro-bind /usr /usr --proc /proc --dev /dev --ro-bind /lib /lib --ro-bind /usr/bin /usr/bin --ro-bind /bin /bin --bind . /home/builder --chdir /home/builder ./build.sh" ],
        PopenConfig {
            cwd: Some(path.join("bot").into()),
            stderr: subprocess::Redirection::Merge,
            stdout: subprocess::Redirection::File(std::fs::File::create(path.join("logs"))?),
            ..Default::default()
        },
    )
    .and_then(|mut p| p.wait());

    match build_result {
        Ok(status) if status.success() => Ok(()),
        _ => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Build failed: {:?}", build_result),
        )),
    }
}
pub async fn download_bot<T: Into<String>, U: Into<PathBuf>, V: Into<String>>(
    key: T,
    path: U,
    bot_bucket: V,
    client: &aws_sdk_s3::Client,
) -> Result<(), io::Error> {
    //TODO: download this in a better way
    let key: String = key.into();
    log::debug!("Downloading bot {:?} from s3", key.clone());
    if let Ok(res) = client.get_object().bucket(bot_bucket).key(key).send().await {
        if let Ok(body) = res.body.collect().await {
            let bytes = body.into_bytes();
            fs::write(path.into().join("bot.zip"), bytes).await?;
            return Ok(());
        }
    }
    Err(io::Error::new(
        io::ErrorKind::Other,
        "Failed to download bot",
    ))
}

/*
#[cfg(test)]
mod tests {
    use std::io;

    use super::build_bot;
    use rand::Rng;
    use tokio::fs;

    async fn build_example(bot_name: &str) -> Result<String, io::Error> {
        let test_id = format!("{:x}", rand::thread_rng().gen::<u32>());
        fs::create_dir_all(format!("/tmp/{}", test_id))
            .await
            .unwrap();
        fs::copy(
            format!("../../example_bots/tests/{}/bot.zip", bot_name),
            format!("/tmp/{}/bot.zip", test_id),
        )
        .await
        .unwrap();
        build_bot(format!("/tmp/{}", test_id))
            .await
            .map(move |_| test_id)
    }

    #[tokio::test]
    pub async fn create_file() {
        let test_id = build_example("create_file_at_build").await.unwrap();
        // Check that the file was created
        let file = fs::read_to_string(format!("/tmp/{}/bot/file.txt", test_id))
            .await
            .unwrap();
        assert_eq!(file, "Hello, World!\n");
    }

    #[tokio::test]
    pub async fn cpp_compile_error() {
        build_example("cpp_compile_error")
            .await
            .expect_err("Build should fail");
    }

    #[tokio::test]
    pub async fn cpp_compile_success() {
        let test_id = build_example("cpp_compile_success")
            .await
            .expect("Build should succeed");

        assert!(subprocess::Exec::shell("./main")
            .cwd(format!("/tmp/{}/bot", test_id))
            .join()
            .unwrap()
            .success());
    }

    #[tokio::test]
    pub async fn logs_output_correctly() {
        let test_id = build_example("create_file_at_build").await.unwrap();
        // Check that the file was created
        let logs = fs::read_to_string(format!("/tmp/{}/logs", test_id))
            .await
            .unwrap();
        assert_eq!(logs, "Success!\nOr not...\n");
    }
}
*/
