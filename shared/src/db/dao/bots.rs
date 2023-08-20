use crate::db::models::Bot;

use super::*;

pub trait BotsDao {
    fn get_bots_with_teams(&mut self, teams: Vec<i32>) -> Vec<BotWithTeam<Team>>;
}

impl BotsDao for PgConnection {
    fn get_bots_with_teams(&mut self, teams: Vec<i32>) -> Vec<BotWithTeam<Team>> {
        schema::bots::dsl::bots
            .filter(schema::bots::dsl::team.eq_any(teams))
            .inner_join(schema::teams::dsl::teams)
            .load::<(Bot, Team)>(self)
            .unwrap()
            .into_iter()
            .map(|(bot, team)| BotWithTeam {
                team,
                id: bot.id,
                name: bot.name,
                description: bot.description,
                score: bot.score,
                created: bot.created,
                uploaded_by: bot.uploaded_by,
                build_status: bot.build_status,
            })
            .collect()
    }
}
