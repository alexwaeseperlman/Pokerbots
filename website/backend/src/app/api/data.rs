use shared::db::models::{BotWithTeam, Team, TeamWithMembers};

use crate::{
    app::login::{TeamData, UserData},
    config::APP_PFP_ENDPOINT,
};

use super::*;

#[get("/my-account")]
pub async fn my_account(session: Session) -> ApiResult<Option<UserData>> {
    Ok(web::Json(login::get_user_data(&session)))
}

#[get("/my-team")]
pub async fn my_team(session: Session) -> ApiResult<Option<TeamData>> {
    Ok(web::Json(login::get_team_data(&session)))
}

#[derive(Deserialize)]
pub enum TeamsQuerySort {
    Score,
    Created,
}

#[derive(Deserialize)]
pub enum TeamsQuerySortDirection {
    Asc,
    Desc,
}

#[derive(Deserialize)]
pub struct TeamQuery {
    pub ids: Option<String>,
    pub page_size: Option<i32>,
    pub page: Option<i32>,
    pub fill_members: Option<bool>,
    pub sort: Option<TeamsQuerySort>,
    pub sort_direction: Option<TeamsQuerySortDirection>,
    pub count: Option<bool>,
}

#[derive(Serialize, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
pub enum TeamsResponse {
    Count(i64),
    Teams(Vec<Team>),
    TeamsWithMembers(Vec<TeamWithMembers>),
}

#[get("/teams")]
pub async fn teams(
    session: Session,
    web::Query::<TeamQuery>(TeamQuery {
        ids,
        page_size,
        page,
        fill_members,
        sort,
        sort_direction,
        count,
    }): web::Query<TeamQuery>,
) -> ApiResult<TeamsResponse> {
    let team = login::get_team_data(&session);
    let conn = &mut (*DB_CONNECTION).get()?;
    let mut base = schema::teams::dsl::teams.into_boxed();
    // <cringe>
    if !count.unwrap_or(false) {
        match sort {
            Some(TeamsQuerySort::Score) | None => match sort_direction {
                Some(TeamsQuerySortDirection::Asc) => {
                    base = base.order_by(schema::teams::dsl::score.asc());
                }
                Some(TeamsQuerySortDirection::Desc) | None => {
                    base = base.order_by(schema::teams::dsl::score.desc());
                }
            },
            Some(TeamsQuerySort::Created) => match sort_direction {
                Some(TeamsQuerySortDirection::Asc) => {
                    base = base.order_by(schema::teams::dsl::id.asc());
                }
                Some(TeamsQuerySortDirection::Desc) | None => {
                    base = base.order_by(schema::teams::dsl::id.desc());
                }
            },
        }
    }
    // </cringe>

    if let Some(ids) = ids {
        let ids: Result<Vec<i32>, _> = ids.split(",").map(|i| i.parse()).collect();
        let ids = ids?;
        base = base.filter(schema::teams::dsl::id.eq_any(ids));
    }
    let page_size = page_size.unwrap_or(10).min(100);
    let page = page.unwrap_or(0);
    if count.unwrap_or(false) {
        return Ok(web::Json(TeamsResponse::Count(
            base.count().get_result::<i64>(conn)?,
        )));
    }
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
        return Ok(web::Json(TeamsResponse::TeamsWithMembers(
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
        )));
    }
    Ok(web::Json(TeamsResponse::Teams(result)))
}

#[derive(Deserialize)]
pub struct BotQuery {
    pub ids: Option<String>,
    pub team: Option<i32>,
    pub page_size: Option<i32>,
    pub page: Option<i32>,
    pub count: Option<bool>,
    pub join_team: Option<bool>,
}

#[derive(Serialize, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
pub enum BotsResponse {
    Count(i64),
    Bots(Vec<Bot>),
    BotsWithTeam(Vec<BotWithTeam<Team>>),
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
        join_team,
    }): web::Query<BotQuery>,
) -> ApiResult<BotsResponse> {
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
        return Ok(web::Json(BotsResponse::Count(count)));
    }
    base = base
        .order_by(schema::bots::dsl::created.desc())
        .limit((page_size).into())
        .offset((page * page_size).into());
    if join_team.unwrap_or(false) {
        let result: Vec<BotWithTeam<Team>> = base
            .inner_join(
                schema::teams::dsl::teams.on(schema::bots::dsl::team.eq(schema::teams::dsl::id)),
            )
            .select((Bot::as_select(), Team::as_select()))
            .load::<(Bot, Team)>(conn)?
            .into_iter()
            .map(|(b, t)| BotWithTeam {
                build_status: b.build_status,
                created: b.created,
                id: b.id,
                team: t,
                description: b.description,
                name: b.name,
                uploaded_by: b.uploaded_by,
                score: b.score,
            })
            .collect();
        return Ok(web::Json(BotsResponse::BotsWithTeam(result)));
    }
    let result: Vec<Bot> = base.load::<Bot>(conn)?.into_iter().collect();
    Ok(web::Json(BotsResponse::Bots(result)))
}

#[derive(Deserialize)]
pub struct InviteCodeQuery {
    pub code: String,
}

#[derive(Serialize, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
pub struct InviteCodeResponse {
    pub invite_code: String,
    pub team: Team,
    pub expires: i64,
}

#[get("/invite-code")]
pub async fn invite_code(
    web::Query::<InviteCodeQuery>(InviteCodeQuery { code }): web::Query<InviteCodeQuery>,
) -> ApiResult<InviteCodeResponse> {
    let conn = &mut (*DB_CONNECTION).get()?;
    let (invite, team) = schema::team_invites::dsl::team_invites
        .inner_join(schema::teams::dsl::teams)
        .filter(schema::team_invites::dsl::invite_code.eq(&code))
        .first::<(TeamInvite, Team)>(conn)?;
    Ok(web::Json(InviteCodeResponse {
        invite_code: code,
        expires: invite.expires,
        team,
    }))
}

#[derive(Deserialize)]
pub struct PfpQuery {
    pub id: i32,
}

#[get("/pfp")]
pub async fn pfp(
    web::Query::<PfpQuery>(PfpQuery { id }): web::Query<PfpQuery>,
    s3_client: web::Data<aws_sdk_s3::Client>,
) -> Result<HttpResponse, ApiError> {
    let response = s3_client
        .get_object()
        .bucket(&*PFP_S3_BUCKET)
        .key(id.to_string())
        .send()
        .await?;

    Ok(HttpResponse::Ok().streaming(response.body))
}
