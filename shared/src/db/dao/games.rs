use diesel::{
    pg::Pg,
    query_builder::{BoxedSqlQuery, Query, QueryBuilder, QueryFragment, SelectQuery},
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::*;

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

pub trait GamesDao {
    fn count_games(&mut self, options: GameQueryOptions) -> Result<i64, diesel::result::Error>;
    fn get_games(
        &mut self,
        games_options: GameQueryOptions,
        page_options: PageOptions,
    ) -> Result<Vec<GameWithBotsWithResult<BotWithTeam<Team>>>, diesel::result::Error>;
}

impl GamesDao for PgConnection {
    fn count_games(
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

    fn get_games(
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

        let result: Vec<(Game, Bot, Bot, Team, Team, Option<GameResult>)> = base.load(self)?;
        Ok(result
            .into_iter()
            .map(
                |(game, defender, challenger, defender_team, challenger_team, game_result)| {
                    GameWithBotsWithResult {
                        id: game.id,
                        defender: BotWithTeam::from_bot_and_team(defender, defender_team),
                        challenger: BotWithTeam::from_bot_and_team(challenger, challenger_team),
                        created: game.created,
                        defender_rating: game.defender_rating,
                        challenger_rating: game.challenger_rating,
                        result: game_result,
                    }
                },
            )
            .collect())
    }
}
