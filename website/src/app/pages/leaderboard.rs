use actix_web::{get, web, HttpResponse, error};
use serde_json::{json, Value};
use crate::app::login::UserData;
use crate::{config::DB_CONNECTION, schema::teams};
use diesel::prelude::*;

#[get("/leaderboard")]
pub async fn leaderboard(
    hb: web::Data<handlebars::Handlebars<'_>>,
    user: Option<web::ReqData<UserData>>,
) -> actix_web::Result<HttpResponse> {
    let u: Option<UserData> = user.map(|x| x.into_inner());

    let conn = &mut (*DB_CONNECTION).get().unwrap();
    

    // hardcoding elo values for testing purposes
    diesel::update(teams::table.filter(teams::team_name.eq("Skilled Quesadillas")))
    .set(teams::elo.eq(200))
    .execute(conn)
    .map_err(|err| error::ErrorInternalServerError(err))?;

    // also hardcoding elo values for testing purposes
    diesel::update(teams::table.filter(teams::team_name.eq("Merciless Alpacas")))
    .set(teams::elo.eq(100))
    .execute(conn)
    .map_err(|err| error::ErrorInternalServerError(err))?;

    // vector with all team names (ranked or not it doesnt matter)
    let teams_names = teams::table
        .order(teams::elo.desc())
        .select(teams::team_name)
        .load::<String>(conn)
        .map_err(|err| error::ErrorInternalServerError(err))?;

    let teams_names_json: Value = serde_json::to_value(&teams_names)
        .map_err(|err| error::ErrorInternalServerError(err))?;


    // vector of corresponding ranked ELOs
    let elos = teams::table
        .order(teams::elo.desc())
        .select(teams::elo)
        .load::<Option<i32>>(conn)
        .map_err(|err| error::ErrorInternalServerError(err))?;

    let elos_json: Value = serde_json::to_value(&elos)
        .map_err(|err| error::ErrorInternalServerError(err))?
        
    // data packaging
    let data = json!({
        "user": u,
        "teams": teams_names_json,
        "elos": elos_json,
    });

    // template rendering
    let body = hb.render("leaderboard", &data).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {}", e))
    })?;

    Ok(HttpResponse::Ok().body(body))
}

