use tokio::{fs, io};

use std::path::{Path, PathBuf};

use shared::Bot;

pub async fn build_bot<T: AsRef<Path>>(bot_folder: T) -> Result<(), io::Error> {
    //TODO: run this in a cgroup this to protect against zip bombs
    // We leave the bot.zip in the directory cause why not
    let path = bot_folder.as_ref();
    log::info!("Constructing bot in {:?}", path);

    shared::process::Process::sh_configured("unzip -o bot.zip", move |command| {
        command.current_dir(path)
    })
    .await?
    .wait()
    .await?;

    // This fails if the bot folder is not named bot.
    // This should be guaranteed by the server.

    // Read build command from bot.json
    let bot_json: Bot = {
        let json = fs::read_to_string(path.join("bot/bot.json")).await?;
        serde_json::from_str::<Bot>(&json)?
    };
    let build_result = shared::process::Process::sh_configured(
        bot_json.build.unwrap_or_default(),
        move |command| command.current_dir(path.join("bot")),
    )
    .await?
    .wait()
    .await?;

    if !build_result.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Build failed: {:?}", build_result),
        ));
    }

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::build_bot;
    use rand::Rng;
    use tokio::fs;

    #[tokio::test]
    pub async fn create_file() {
        let test_id = format!("{:x}", rand::thread_rng().gen::<u32>());
        fs::create_dir(format!("/tmp/{}", test_id)).await.unwrap();
        fs::copy(
            "../../example_bots/tests/create_file_at_build/bot.zip",
            format!("/tmp/{}/bot.zip", test_id),
        )
        .await
        .unwrap();
        build_bot(format!("/tmp/{}", test_id)).await.unwrap();
        // Check that the file was created
        let mut file = fs::read_to_string(format!("/tmp/{}/bot/file.txt", test_id))
            .await
            .unwrap();
        assert_eq!(file, "Hello, World!\n");
    }

    #[tokio::test]
    pub async fn cpp_compile_error() {
        let test_id = format!("{:x}", rand::thread_rng().gen::<u32>());
        fs::create_dir(format!("/tmp/{}", test_id)).await.unwrap();
        fs::copy(
            "../../example_bots/tests/cpp_compile_error/bot.zip",
            format!("/tmp/{}/bot.zip", test_id),
        )
        .await
        .unwrap();
        build_bot(format!("/tmp/{}", test_id))
            .await
            .expect_err("Build should fail");
    }

    #[tokio::test]
    pub async fn cpp_compile_success() {
        let test_id = format!("{:x}", rand::thread_rng().gen::<u32>());
        fs::create_dir(format!("/tmp/{}", test_id)).await.unwrap();
        fs::copy(
            "../../example_bots/tests/cpp_compile_success/bot.zip",
            format!("/tmp/{}/bot.zip", test_id),
        )
        .await
        .unwrap();
        build_bot(format!("/tmp/{}", test_id))
            .await
            .expect("Build should succeed");

        shared::process::Process::sh_configured("./main", move |command| {
            command.current_dir(format!("/tmp/{}/bot", test_id))
        })
        .await
        .unwrap()
        .wait()
        .await
        .unwrap();
    }
}
