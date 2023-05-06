use actix_web::{get, web, HttpResponse, error};
use serde_json::{json, Value};
use crate::app::login::UserData;
use crate::{config::DB_CONNECTION, schema::teams};
use diesel::prelude::*;
use rand::Rng;

#[get("/leaderboard")]
pub async fn leaderboard(
    hb: web::Data<handlebars::Handlebars<'_>>,
    user: Option<web::ReqData<UserData>>,
) -> actix_web::Result<HttpResponse> {
    let u: Option<UserData> = user.map(|x| x.into_inner());

    let conn = &mut (*DB_CONNECTION).get().unwrap();
    
    // vector with all team names (ranked or not it doesnt matter)

    let teams_names = teams::table
        .select(teams::team_name)
        .load::<String>(conn)
        .map_err(|err| error::ErrorInternalServerError(err))?;

    let teams_names_json: Value = serde_json::to_value(&teams_names)
        .map_err(|err| error::ErrorInternalServerError(err))?;


    // vector of corresponding ELOs (ranked or not it doesnt matter)
    
    let elos = teams::table
        .select(teams::elo)
        .load::<Option<i32>>(conn)
        .map_err(|err| error::ErrorInternalServerError(err))?;

    let elos_json: Value = serde_json::to_value(&elos)
        .map_err(|err| error::ErrorInternalServerError(err))?;


    // vector of dummy elos just for the UI
    let rand_elos = teams_names
    .iter()
    .map(|_| rand::thread_rng().gen_range(1..=3000))
    .collect::<Vec<i32>>();

    let rand_elos_json: Value = serde_json::to_value(&rand_elos)
    .map_err(|err| error::ErrorInternalServerError(err))?;

    // data packaging
    let data = json!({
        "user": u,
        "teams": teams_names_json,
        "elos": rand_elos_json,
    });

    // template rendering
    let body = hb.render("leaderboard", &data).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;

    Ok(HttpResponse::Ok().body(body))
}
