use crate::db::schema::*;

diesel::alias!(
    bots as defender_bots: DefenderBotsAlias,
    bots as challenger_bots: ChallengerBotsAlias,
    teams as defender_teams: DefenderTeamsAlias,
    teams as challenger_teams: ChallengerTeamsAlias
);
