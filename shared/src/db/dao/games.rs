use async_trait::async_trait;
use diesel::{
    pg::Pg,
    query_builder::{BoxedSqlQuery, Query, QueryBuilder, QueryFragment, SelectQuery},
};
use futures_util::future::try_join3;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use aws_sdk_s3::presigning::PresigningConfig;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl};
use rand::Rng;

use super::*;
use crate::{db::models, GameTask, PresignedRequest, WhichBot};

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
pub struct GameQueryOptions {
    pub id: Option<String>,
    pub team: Option<i32>,
    pub running: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[cfg_attr(feature = "ts-bindings", ts(export))]
pub struct PageOptions {
    pub page_size: i32,
    pub page: i32,
}

#[async_trait]
pub trait GamesDao {
    async fn count_games(
        &mut self,
        options: GameQueryOptions,
    ) -> Result<i64, diesel::result::Error>;
    async fn get_games(
        &mut self,
        games_options: GameQueryOptions,
        page_options: PageOptions,
    ) -> Result<Vec<GameWithBotsWithResult<BotWithTeam<Team>>>, diesel::result::Error>;
    async fn create_game(
        &mut self,
        defender: &Team,
        challenger: &Team,
        rated: bool,
        game_logs_s3_bucket: &str,
        new_games_sqs_queue: &str,
        sqs_client: &aws_sdk_sqs::Client,
        s3_client: &aws_sdk_s3::Client,
    ) -> Result<String, Box<dyn std::error::Error>>;
}

#[async_trait]
impl GamesDao for PgConnection {
    async fn count_games(
        &mut self,
        GameQueryOptions { id, team, running }: GameQueryOptions,
    ) -> Result<i64, diesel::result::Error> {
        use schema::*;
        let mut base = games::table.into_boxed();

        if let Some(id) = id {
            base = base.filter(schema::games::dsl::id.eq(id));
        }
        if let Some(team) = team {
            // get bots belonging to the team
            let bots = schema::bots::dsl::bots
                .filter(schema::bots::dsl::team.eq(team))
                .select(schema::bots::dsl::id);
            base = base.filter(
                schema::games::dsl::defender
                    .eq_any(bots.clone())
                    .or(schema::games::dsl::challenger.eq_any(bots)),
            );
        }
        Ok(base.count().get_result::<i64>(self)?)
    }

    async fn get_games(
        &mut self,
        GameQueryOptions { id, team, running }: GameQueryOptions,
        page_options: PageOptions,
    ) -> Result<Vec<GameWithBotsWithResult<BotWithTeam<Team>>>, diesel::result::Error> {
        // TODO: move this into one place once I know what type it should be
        use schema::*;
        use schema_aliases::*;

        let mut base = games::table
            .order_by(games::dsl::created.desc())
            .inner_join(
                defender_bots.on(games::dsl::defender.eq(defender_bots.field(bots::dsl::id))),
            )
            .inner_join(
                challenger_bots.on(games::dsl::challenger.eq(challenger_bots.field(bots::dsl::id))),
            )
            .inner_join(
                defender_teams.on(defender_bots
                    .field(bots::dsl::team)
                    .eq(defender_teams.field(teams::dsl::id))),
            )
            .inner_join(
                challenger_teams.on(challenger_bots
                    .field(bots::dsl::team)
                    .eq(challenger_teams.field(teams::dsl::id))),
            )
            .inner_join(
                defender_users.on(defender_bots
                    .field(bots::dsl::uploaded_by)
                    .eq(defender_users.field(users::dsl::id))),
            )
            .inner_join(
                challenger_users.on(challenger_bots
                    .field(bots::dsl::uploaded_by)
                    .eq(challenger_users.field(users::dsl::id))),
            )
            .left_join(game_results::dsl::game_results.on(games::dsl::id.eq(game_results::dsl::id)))
            .into_boxed();

        if let Some(id) = id {
            base = base.filter(schema::games::dsl::id.eq(id));
        }
        if let Some(team) = team {
            // get bots belonging to the team
            let bots = schema::bots::dsl::bots
                .filter(schema::bots::dsl::team.eq(team))
                .select(schema::bots::dsl::id);
            base = base.filter(
                schema::games::dsl::defender
                    .eq_any(bots.clone())
                    .or(schema::games::dsl::challenger.eq_any(bots)),
            );
        }
        base = base
            .limit(page_options.page_size.into())
            .offset((page_options.page * page_options.page_size).into());

        let result: Vec<(Game, Bot, Bot, Team, Team, User, User, Option<GameResult>)> =
            base.load(self)?;
        Ok(result
            .into_iter()
            .map(
                |(
                    game,
                    defender,
                    challenger,
                    defender_team,
                    challenger_team,
                    defender_user,
                    challenger_user,
                    game_result,
                )| {
                    GameWithBotsWithResult {
                        id: game.id,
                        defender: BotWithTeam::from_bot_team_user(
                            defender,
                            defender_team,
                            defender_user,
                        ),
                        challenger: BotWithTeam::from_bot_team_user(
                            challenger,
                            challenger_team,
                            challenger_user,
                        ),
                        created: game.created,
                        defender_rating: game.defender_rating,
                        challenger_rating: game.challenger_rating,
                        result: game_result,
                        rated: game.rated,
                    }
                },
            )
            .collect())
    }

    async fn create_game(
        &mut self,
        defender_team: &Team,
        challenger_team: &Team,
        rated: bool,
        game_logs_s3_bucket: &str,
        new_games_sqs_queue: &str,
        sqs_client: &aws_sdk_sqs::Client,
        s3_client: &aws_sdk_s3::Client,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // generate a random code and insert it into the database
        // also push a batch job to the queue
        let id = format!("{:02x}", rand::thread_rng().gen::<u128>());
        let local_id = id.clone();
        if defender_team.active_bot.is_none() {
            return Err("Defender team has no active bot".into());
        }
        if challenger_team.active_bot.is_none() {
            return Err("Challenger team has no active bot".into());
        }
        //self.transaction::<_, anyhow::Error, _>(move |self| {
        log::info!(
            "Creating game {} with defender {} and challenger {}. Current defender rating: {}, current challenger rating: {}, rated: {}",
            id,
            defender_team.active_bot.unwrap(),
            challenger_team.active_bot.unwrap(),
            defender_team.rating,
            challenger_team.rating,
            rated,
        );
        diesel::insert_into(schema::games::dsl::games)
            .values(models::NewGame {
                defender: defender_team.active_bot.unwrap(),
                challenger: challenger_team.active_bot.unwrap(),
                id: id.clone(),
                challenger_rating: challenger_team.rating,
                defender_rating: defender_team.rating,
                rated: rated,
            })
            .execute(self)?;

        log::info!("Game created {}", id);
        // push a batch job to the queue
        match {
            let presign_config =
                PresigningConfig::expires_in(std::time::Duration::from_secs(60 * 60 * 24 * 7))?;

            let (public_logs, defender_logs, challenger_logs) = try_join3(
                s3_client
                    .put_object()
                    .bucket(game_logs_s3_bucket)
                    .key(format!("public/{}", id.clone()))
                    .presigned(presign_config.clone()),
                s3_client
                    .put_object()
                    .bucket(game_logs_s3_bucket)
                    .key(format!("{}/{}", WhichBot::Defender.to_string(), id))
                    .presigned(presign_config.clone()),
                s3_client
                    .put_object()
                    .bucket(game_logs_s3_bucket)
                    .key(format!("{}/{}", WhichBot::Challenger.to_string(), id))
                    .presigned(presign_config.clone()),
            )
            .await?;
            log::debug!(
                "Log presigned keys created {}, {}, {}",
                public_logs.uri(),
                defender_logs.uri(),
                challenger_logs.uri()
            );
            let (public_logs_presigned, defender_logs_presigned, challenger_logs_presigned) = (
                PresignedRequest {
                    url: public_logs.uri().to_string(),
                    headers: public_logs.headers().into(),
                },
                PresignedRequest {
                    url: defender_logs.uri().to_string(),
                    headers: defender_logs.headers().into(),
                },
                PresignedRequest {
                    url: challenger_logs.uri().to_string(),
                    headers: challenger_logs.headers().into(),
                },
            );
            let job = sqs_client
                .send_message()
                .queue_url(new_games_sqs_queue)
                .message_body(&serde_json::to_string(&GameTask::Game {
                    defender: defender_team.active_bot.unwrap(),
                    challenger: challenger_team.active_bot.unwrap(),
                    id: id.clone(),
                    rounds: 100,
                    public_logs_presigned,
                    defender_logs_presigned,
                    challenger_logs_presigned,
                })?)
                .send()
                .await?;
            Ok::<(), anyhow::Error>(())
        } {
            Ok(_) => {
                log::info!("Pushed game task to queue");
            }
            Err(_) => {
                log::error!("Failed to push game task to queue");
                diesel::delete(schema::games::dsl::games.filter(schema::games::dsl::id.eq(id)))
                    .execute(self)?;
            }
        }
        //   Ok(())
        //})?;
        Ok(local_id)
    }
}
