use actix_multipart::Multipart;
use actix_session::Session;
use actix_web::{post, web, HttpResponse};
use futures::{StreamExt, TryStreamExt};

use std::io::Write;

use crate::app::login;
#[post("/api/upload-bot")]
pub async fn upload_bot(
    session: Session,
    mut payload: Multipart,
) -> actix_web::Result<HttpResponse> {
    while let Ok(Some(mut field)) = payload.try_next().await {
        let team = login::get_team_data(&session);
        let file_string = format!("/tmp/{}.py", team.unwrap().team_name);
        let mut f =
            web::block(move || std::fs::File::create(std::path::Path::new(&file_string))).await??;

        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            f = web::block(move || f.write_all(&data).map(|_| f)).await??;
        }
    }
    Ok(HttpResponse::Ok().body("Successfully uploaded bot"))
}
