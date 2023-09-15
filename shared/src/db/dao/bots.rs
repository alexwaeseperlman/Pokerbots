use super::*;

pub trait BotsDao {
    fn get_bots_with_teams(&mut self, teams: Vec<i32>) -> Vec<BotWithTeam<Team>>;
}

impl BotsDao for PgConnection {
    fn get_bots_with_teams(&mut self, teams: Vec<i32>) -> Vec<BotWithTeam<Team>> {
        schema::bots::dsl::bots
            .filter(schema::bots::dsl::team.eq_any(teams))
            .inner_join(
                schema::teams::dsl::teams.on(schema::bots::dsl::team.eq(schema::teams::dsl::id)),
            )
            .inner_join(
                schema::users::dsl::users
                    .on(schema::bots::dsl::uploaded_by.eq(schema::users::dsl::id)),
            )
            .load::<(Bot, Team, User)>(self)
            .unwrap()
            .into_iter()
            .map(|(bot, team, user)| BotWithTeam {
                team,
                id: bot.id,
                name: bot.name,
                description: bot.description,
                rating: bot.rating,
                created: bot.created,
                uploaded_by: user,
                build_status: bot.build_status,
            })
            .collect()
    }
}
