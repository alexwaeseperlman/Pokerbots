use shared::db::models::UserProfile;
use uuid::Uuid;

use super::*;

#[derive(Deserialize)]
pub struct CreateTeamQuery {
    pub name: String,
}

pub fn validate_name(name: &str, message: &str) -> Result<(), ApiError> {
    if name.len() < 3 || name.len() > 20 {
        return Err(actix_web::error::ErrorNotAcceptable(message.to_string()).into());
    }
    for c in name.chars() {
        if !c.is_alphanumeric() && c != ' ' && c != '-' {
            return Err(actix_web::error::ErrorNotAcceptable(message.to_string()).into());
        }
    }
    if name.contains("  ") {
        return Err(actix_web::error::ErrorNotAcceptable(message.to_string()).into());
    }
    Ok(())
}

#[get("/create-team")]
pub async fn create_team(
    session: Session,
    web::Query::<CreateTeamQuery>(CreateTeamQuery { name }): web::Query<CreateTeamQuery>,
) -> ApiResult<()> {
    let user =
        auth::get_user(&session).ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    auth::get_profile(&session).ok_or(actix_web::error::ErrorUnauthorized(
        "Your profile must be completed to create a team.",
    ))?;
    // You can't create a team if you're already in one
    if auth::get_team(&session).is_some() {
        return Err(actix_web::error::ErrorConflict("You are already on a team.").into());
    }

    validate_name(&name, "Invalid team name. It must be at most 20 characters and cannot contain consecutive spaces.")?;

    let conn = &mut (*DB_CONNECTION).get()?;
    let new_id = diesel::insert_into(teams::dsl::teams)
        .values(NewTeam {
            name,
            owner: user.clone().id,
        })
        .returning(teams::dsl::id)
        .get_result::<i32>(conn)?;

    diesel::update(users::dsl::users)
        .filter(users::dsl::id.eq(user.id))
        .set(users::dsl::team.eq(new_id))
        .get_result::<User>(conn)?;

    Ok(web::Json(()))
}

#[derive(Deserialize)]
pub struct JoinTeamQuery {
    pub code: String,
}

#[get("/delete-team")]
pub async fn delete_team(session: Session) -> ApiResult<()> {
    let user =
        auth::get_user(&session).ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    let team =
        auth::get_team(&session).ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;
    // You can't delete a team if you're not in one
    if team.clone().owner != user.id {
        return Err(actix_web::error::ErrorNotFound(
            "You can't delete a team if you are not the owner.",
        )
        .into());
    }
    let conn = &mut (*DB_CONNECTION).get()?;

    conn.transaction(|conn| {
        // Set the team to deleted
        diesel::update(teams::dsl::teams)
            .filter(teams::dsl::id.eq(team.id))
            .set(teams::dsl::deleted_at.eq(Some(chrono::offset::Utc::now().timestamp())))
            .execute(conn)?;
        // Make everyone on the team leave the team
        diesel::update(users::dsl::users)
            .filter(users::dsl::team.eq(team.id))
            .set(users::dsl::team.eq::<Option<i32>>(None))
            .execute(conn)?;
        diesel::result::QueryResult::Ok(())
    })?;

    Ok(web::Json(()))
}

#[get("/leave-team")]
pub async fn leave_team(session: Session) -> ApiResult<()> {
    let user =
        auth::get_user(&session).ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    let team =
        auth::get_team(&session).ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;
    // You can't delete a team if you're not in one or you're the owner
    if user.clone().id == team.owner {
        return Err(actix_web::error::ErrorNotAcceptable(
            "You can't leave the team if you are the owner.",
        )
        .into());
    }
    let conn = &mut (*DB_CONNECTION).get()?;

    // Set the current user's team to null
    diesel::update(users::dsl::users)
        .filter(users::dsl::id.eq(user.id))
        .set(users::dsl::team.eq::<Option<i32>>(None))
        .execute(conn)?;

    Ok(web::Json(()))
}
#[get("/create-invite")]
pub async fn create_invite(session: Session) -> ApiResult<()> {
    let user =
        auth::get_user(&session).ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    let team =
        auth::get_team(&session).ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;
    // You can't join a team if you are already on one or if you aren't logged in
    // Also only the owner can create a team

    // if the number of invites plus the number of users is at the limit, then don't create an invite
    if team.invites.clone().unwrap().len() + team.members.len() >= crate::config::TEAM_SIZE as usize
    {
        return Err(actix_web::error::ErrorNotAcceptable("Team is full.").into());
    }

    /*if user.email != team.clone().owner {
        return Err(actix_web::error::ErrorNotAcceptable("Not able to make invite.").into());
    }*/
    // Insert an invite with expiry date 100 years from now
    // We are not using expiry dates for invites
    let day: i64 = 24 * 3600 * 1000 * 365 * 100;
    let now: i64 = chrono::offset::Utc::now().timestamp();
    let conn = &mut (*DB_CONNECTION).get()?;
    let out = diesel::insert_into(team_invites::dsl::team_invites)
        .values(NewInvite {
            expires: now + day,
            code: format!("{:02x}", rand::thread_rng().gen::<u128>()),
            team: team.clone().id,
        })
        .returning(team_invites::dsl::code)
        .get_result::<String>(conn)?;
    Ok(web::Json(()))
}

#[derive(Deserialize)]
pub struct CancelTeamQuery {
    pub code: String,
}

#[get("/cancel-invite")]
pub async fn cancel_invite(
    session: Session,
    web::Query(CancelTeamQuery { code }): web::Query<CancelTeamQuery>,
) -> ApiResult<()> {
    let user =
        auth::get_user(&session).ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    let team =
        auth::get_team(&session).ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;

    // Insert an invite with expiry date 24 hours from now
    let conn = &mut (*DB_CONNECTION).get()?;
    let out = diesel::delete(team_invites::dsl::team_invites)
        .filter(team_invites::dsl::code.eq(&code))
        .filter(team_invites::dsl::team.eq(team.id))
        .returning(team_invites::dsl::code)
        .get_result::<String>(conn)?;
    Ok(web::Json(()))
}

#[get("/join-team")]
pub async fn join_team(
    session: Session,
    web::Query::<JoinTeamQuery>(JoinTeamQuery { code }): web::Query<JoinTeamQuery>,
    req: actix_web::HttpRequest,
) -> ApiResult<()> {
    let user =
        auth::get_user(&session).ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    let team = auth::get_team(&session);
    auth::get_profile(&session).ok_or(actix_web::error::ErrorUnauthorized(
        "Your profile must be completed to create a team.",
    ))?;

    // You can't join a team if you are already on one or if you aren't logged in
    if team.is_some() {
        return Err(actix_web::error::ErrorNotAcceptable("You are already on a team.").into());
    } else {
        let conn = &mut (*DB_CONNECTION).get()?;
        // Check if there is an existing team invite with this code
        let codes: Vec<TeamInvite> = team_invites::dsl::team_invites
            .find(code.clone())
            .load::<TeamInvite>(conn)?;
        if let Some(code) = codes.first() {
            let now: i64 = chrono::offset::Utc::now().timestamp();
            let TeamInvite {
                code,
                expires,
                team,
            } = code;
            if expires < &now {
                return Err(actix_web::error::ErrorNotAcceptable("Invalid code.").into());
            } else {
                // Set the users team and set the code to used
                diesel::delete(team_invites::dsl::team_invites)
                    .filter(team_invites::dsl::code.eq(code))
                    .execute(conn)?;
                diesel::update(users::dsl::users)
                    .filter(users::dsl::id.eq(user.id))
                    .set(users::dsl::team.eq(team))
                    .execute(conn)?;
            }
        } else {
            return Err(actix_web::error::ErrorNotAcceptable("Invalid code.").into());
        }
    }
    Ok(web::Json(()))
}

#[put("/upload-pfp")]
pub async fn upload_pfp(
    s3_client: actix_web::web::Data<s3::Client>,
    session: Session,
    mut payload: web::Payload,
) -> ApiResult<()> {
    let user =
        auth::get_user(&session).ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    let team =
        auth::get_team(&session).ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;

    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > 500000 {
            return Err(actix_web::error::ErrorBadRequest("PFP too large").into());
        }
        body.extend_from_slice(&chunk);
    }
    s3_client
        .put_object()
        .bucket(pfp_s3_bucket())
        .key(format!("{}", team.id))
        .body(body.to_vec().into())
        .acl(s3::types::ObjectCannedAcl::PublicRead)
        .send()
        .await?;

    // TODO: Maybe run the image through a sanitizer/thumbnailer
    // TODO: Maybe check for inappropriate content using Rekognition

    Ok(web::Json(()))
}

#[derive(Deserialize)]
pub struct KickMemberQuery {
    pub user_id: Uuid,
}

#[get("/kick-member")]
pub async fn kick_member(
    session: Session,
    web::Query::<KickMemberQuery>(KickMemberQuery { user_id }): web::Query<KickMemberQuery>,
) -> ApiResult<()> {
    let user =
        auth::get_user(&session).ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    let team =
        auth::get_team(&session).ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;
    if team.clone().owner != user.clone().id {
        return Err(
            actix_web::error::ErrorUnauthorized("Only the team owner can kick members.").into(),
        );
    }

    let conn = &mut (*DB_CONNECTION).get()?;
    diesel::update(users::dsl::users)
        .filter(users::dsl::id.eq(user_id))
        .filter(users::dsl::team.eq(team.id))
        .set(users::dsl::team.eq::<Option<i32>>(None))
        .execute(conn)?;
    // TODO: Maybe some kind of message should show for the user next time they log in?
    Ok(web::Json(()))
}

#[derive(Deserialize)]
pub struct MakeOwnerQuery {
    pub user_id: Uuid,
}

#[get("/update-owner")]
pub async fn update_owner(
    session: Session,
    web::Query::<MakeOwnerQuery>(MakeOwnerQuery { user_id }): web::Query<MakeOwnerQuery>,
) -> ApiResult<()> {
    let user =
        auth::get_user(&session).ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    let team =
        auth::get_team(&session).ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;
    if team.clone().owner != user.clone().id {
        return Err(
            actix_web::error::ErrorUnauthorized("Only the team owner can kick members.").into(),
        );
    }

    // make sure the user is on the team
    let conn = &mut (*DB_CONNECTION).get()?;
    let count: i64 = users::table
        .find(&user_id)
        .count()
        .get_result::<i64>(conn)?;
    if count == 0 {
        return Err(actix_web::error::ErrorNotFound("User not found.").into());
    }

    let conn = &mut (*DB_CONNECTION).get()?;
    diesel::update(teams::table)
        .set(teams::dsl::owner.eq(user_id))
        .execute(conn)?;
    // TODO: Maybe some kind of message should show for the user next time they log in?
    Ok(web::Json(()))
}

#[derive(Deserialize)]
pub struct RenameTeamQuery {
    pub to: String,
}

#[get("/rename-team")]
pub async fn rename_team(
    session: Session,
    web::Query::<RenameTeamQuery>(RenameTeamQuery { to }): web::Query<RenameTeamQuery>,
) -> ApiResult<()> {
    let user =
        auth::get_user(&session).ok_or(actix_web::error::ErrorUnauthorized("Not logged in"))?;
    let team =
        auth::get_team(&session).ok_or(actix_web::error::ErrorUnauthorized("Not on a team"))?;

    validate_name(&to, "Invalid team name. It must be at most 20 characters and cannot contain consecutive spaces.")?;

    let conn = &mut (*DB_CONNECTION).get()?;
    diesel::update(teams::dsl::teams)
        .filter(teams::dsl::id.eq(team.clone().id))
        // Team members can change the team name
        //.filter(teams::dsl::owner.eq(user.clone().email))
        .set(teams::dsl::name.eq(to))
        .execute(conn)?;
    Ok(web::Json(()))
}
