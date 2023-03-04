use actix_session::Session;
use actix_web::web::ReqData;
use serde::Deserialize;
use serde_json::json;

use crate::app::login::*;
use crate::models::User;
use actix::*;
use actix_service::{IntoService, Service, ServiceFactory};
use actix_web::*;
use actix_web::{get, HttpResponse};

use crate::{TeamData, UserData, DB_CONNECTION};
#[derive(Deserialize)]
pub struct CreateTeam {
    pub team_name: String,
}

#[get("/api/create-team")]
pub async fn create_team(
    session: Session,
    web::Query::<CreateTeam>(CreateTeam { team_name }): web::Query<CreateTeam>,
) -> Result<HttpResponse> {
    use diesel::*;
    let user = get_user_data(Some(session.clone()));
    if user.is_none() {
        return Ok(HttpResponse::NotFound()
            .append_header(("Location", "/login"))
            .finish());
    }
    // You can't create a team if you're already in one
    if get_team_data(Some(session)).is_some() {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/manage-team"))
            .finish());
    }
    let conn = &mut (*DB_CONNECTION).get().unwrap();
    use crate::schema::teams::dsl::*;
    use crate::schema::users::dsl::*;
    let team_create_result = diesel::insert_into(teams)
        .values(crate::models::NewTeam {
            teamname: team_name,
            owner: user.clone().unwrap().email,
        })
        .returning(id)
        .get_result::<i32>(conn);

    if let Ok(new_id) = team_create_result {
        println!(
            "{:?}",
            diesel::update(users)
                .filter(email.eq(user.unwrap().email))
                .set(teamid.eq(new_id))
                .get_result::<User>(conn)
        );
        return Ok(HttpResponse::Ok().body(format!("Team created with id {}", new_id)));
    } else {
        println!("{:?}", team_create_result);
    }

    Ok(HttpResponse::Ok().body(format!("Could not create team")))
}

#[get("/manage-team")]
pub async fn manage_team(
    hb: web::Data<handlebars::Handlebars<'_>>,
    session: Session,
) -> Result<HttpResponse> {
    let user = get_user_data(Some(session.clone()));
    let team = get_team_data(Some(session));

    let data = json!({
        "user": user,
        "team": team,
        "microsoft_login": microsoft_login_url(),
        "isOwner": user.is_some() && team.is_some() && user.unwrap().email == team.unwrap().owner

    });
    println!("{:?}", data);
    let body = hb.render("manage-team", &data).unwrap();
    Ok(HttpResponse::Ok().body(body))
}
