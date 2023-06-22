use std::path::PathBuf;

use tokio::{fs, io};

pub async fn download_file<T: Into<String>, U: Into<PathBuf>, V: Into<String>>(
    key: T,
    path: U,
    bucket: V,
    client: &aws_sdk_s3::Client,
) -> Result<(), io::Error> {
    //TODO: download this in a better way
    let key: String = key.into();
    if let Ok(res) = client.get_object().bucket(bucket).key(key).send().await {
        if let Ok(body) = res.body.collect().await {
            let bytes = body.into_bytes();
            fs::write(path.into(), bytes).await?;
            return Ok(());
        }
    }
    Err(io::Error::new(
        io::ErrorKind::Other,
        "Failed to download bot",
    ))
}
