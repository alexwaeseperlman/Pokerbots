use actix_session::Session;
use actix_web::{get, web, HttpResponse};
use diesel::*;
use serde::Deserialize;
use serde_json::json;

use crate::app::{api::ApiResult, login};
use crate::config::APP_PFP_ENDPOINT;
use shared::db::conn::DB_CONNECTION;
use shared::db::{
    models::{Bot, Team, TeamInvite, TeamWithMembers, User},
    schema,
};

use super::ServerMessage;

#[get("/my-account")]
pub async fn my_account(session: Session) -> ApiResult {
    Ok(HttpResponse::Ok().json(login::get_user_data(&session)))
}

#[get("/my-team")]
pub async fn my_team(session: Session) -> ApiResult {
    log::debug!("my-team");
    Ok(HttpResponse::Ok().json(login::get_team_data(&session)))
}

#[derive(Deserialize)]
pub struct ServerMessageQuery {
    pub clear: Option<bool>,
}

#[get("/server-message")]
pub async fn server_message(
    session: Session,
    web::Query::<ServerMessageQuery>(ServerMessageQuery { clear }): web::Query<ServerMessageQuery>,
) -> ApiResult {
    let msg = HttpResponse::Ok().json(session.get::<ServerMessage>("message")?);
    if clear.unwrap_or(false) {
        session.remove("message");
    }
    Ok(msg)
}
#[derive(Deserialize)]
pub struct TeamQuery {
    pub ids: Option<String>,
    pub page_size: Option<i32>,
    pub page: Option<i32>,
    pub fill_members: Option<bool>,
}

#[get("/teams")]
pub async fn teams(
    session: Session,
    web::Query::<TeamQuery>(TeamQuery {
        ids,
        page_size,
        page,
        fill_members,
    }): web::Query<TeamQuery>,
) -> ApiResult {
    let team = login::get_team_data(&session);
    let conn = &mut (*DB_CONNECTION).get()?;
    let mut base = schema::teams::dsl::teams
        .order_by(schema::teams::dsl::score.desc())
        .into_boxed();
    if let Some(ids) = ids {
        let ids: Result<Vec<i32>, _> = ids.split(",").map(|i| i.parse()).collect();
        let ids = ids?;
        base = base.filter(schema::teams::dsl::id.eq_any(ids));
    }
    let page_size = page_size.unwrap_or(10).min(100);
    let page = page.unwrap_or(0);
    base = base
        .limit((page_size).into())
        .offset((page * page_size).into());
    let result: Vec<Team> = base.load::<Team>(conn)?.into_iter().collect();
    if fill_members.unwrap_or(false) {
        let users = schema::users::dsl::users
            .filter(
                schema::users::dsl::team_id
                    .eq_any(result.iter().map(|t| t.id).collect::<Vec<i32>>()),
            )
            .load::<User>(conn)?;
        let invites = schema::team_invites::dsl::team_invites
            .filter(schema::team_invites::dsl::teamid.eq(team.clone().map(|u| u.id).unwrap_or(-1)))
            .load::<TeamInvite>(conn)?;
        return Ok(HttpResponse::Ok().json(
            result
                .into_iter()
                .map(|t| TeamWithMembers {
                    members: users
                        .clone()
                        .into_iter()
                        .filter(|u| u.team_id == Some(t.id))
                        .collect(),
                    // only show invites if the user is on the team
                    invites: if Some(t.id) == team.clone().map(|t| t.id) {
                        Some(
                            invites
                                .clone()
                                .into_iter()
                                .filter(|u| u.teamid == t.id)
                                .collect(),
                        )
                    } else {
                        None
                    },
                    active_bot: t.active_bot,
                    id: t.id,
                    owner: t.owner,
                    score: t.score,
                    team_name: t.team_name,
                })
                .collect::<Vec<TeamWithMembers>>(),
        ));
    }
    Ok(HttpResponse::Ok().json(result))
}

#[derive(Deserialize)]
pub struct BotQuery {
    pub ids: Option<String>,
    pub team: Option<i32>,
    pub page_size: Option<i32>,
    pub page: Option<i32>,
    pub count: Option<bool>,
}

#[get("/bots")]
pub async fn bots(
    session: Session,
    web::Query::<BotQuery>(BotQuery {
        ids,
        team,
        page_size,
        page,
        count,
    }): web::Query<BotQuery>,
) -> ApiResult {
    let conn = &mut (*DB_CONNECTION).get()?;
    let mut base = schema::bots::dsl::bots.into_boxed();
    if let Some(ids) = ids {
        let ids: Result<Vec<i32>, _> = ids.split(",").map(|i| i.parse()).collect();
        let ids = ids?;
        base = base.filter(schema::bots::dsl::id.eq_any(ids));
    }
    if let Some(team) = team {
        // get bots belonging to the team
        base = base.filter(schema::bots::dsl::team.eq(team))
    }
    let count = count.unwrap_or(false);
    let page_size = page_size.unwrap_or(10).min(100);
    let page = page.unwrap_or(0);
    if count {
        let count = base.count().get_result::<i64>(conn)?;
        return Ok(HttpResponse::Ok().json(json!({ "count": count })));
    }
    base = base
        .order_by(schema::bots::dsl::created.desc())
        .limit((page_size).into())
        .offset((page * page_size).into());
    let result: Vec<Bot> = base.load::<Bot>(conn)?.into_iter().collect();
    Ok(HttpResponse::Ok().json(result))
}

#[derive(Deserialize)]
pub struct InviteCodeQuery {
    pub code: String,
}
#[get("/invite-code")]
pub async fn invite_codes(
    web::Query::<InviteCodeQuery>(InviteCodeQuery { code }): web::Query<InviteCodeQuery>,
) -> ApiResult {
    let conn = &mut (*DB_CONNECTION).get()?;
    let invite = schema::team_invites::dsl::team_invites
        .inner_join(schema::teams::dsl::teams)
        .filter(schema::team_invites::dsl::invite_code.eq(code))
        .first::<(TeamInvite, Team)>(conn)?;
    Ok(HttpResponse::Ok().json(invite))
}

#[get("/pfp-endpoint")]
pub async fn pfp_endpoint() -> ApiResult {
    Ok(HttpResponse::Ok().json(&*APP_PFP_ENDPOINT))
}
