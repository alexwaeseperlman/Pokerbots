use std::io::Write;

use actix_multipart::Multipart;
use actix_web::{web, App, Error, HttpResponse, HttpServer};
use futures::{StreamExt, TryStreamExt};

pub async fn save_file(mut payload: Multipart, file_path: &'static str) -> Option<bool> {
    // iterate over multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition();
        //let filename = content_type.get_filename().unwrap();
        // let filepath = format!(".{}", file_path);

        // File::create is blocking operation, use threadpool
        let mut f = web::block(move || std::fs::File::create(file_path))
            .await
            .expect("file create failed")
            .expect("blocking execution of file creation failed");


        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&data).map(|_| f))
                .await
                .expect("file write failed")
                .expect("blocking execution of file write filed");
        }
    }

    Some(true)
}
