use crate::{app::login::AzureMeResponse, models::*};
use cfg_if::cfg_if;
/*
#[server(GetTeam, "/api")]
pub async fn get_team(cx: Scope) -> Result<Option<i32>, ServerFnError> {
    use actix_session::SessionExt;
    use actix_web::Error;

    use crate::schema::teams::dsl::teams;
    use crate::DB_CONNECTION;
    use diesel::*;
    let session = crate::get_session(cx);*/
/*teams
    .limit(5)
    .load::<Team>(&mut (*DB_CONNECTION).get().unwrap())
    .expect("Error loading teams");
    Ok(Some(1))
}*/
