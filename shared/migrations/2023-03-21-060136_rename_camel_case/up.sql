ALTER TABLE teams
RENAME COLUMN teamname TO team_name;

ALTER TABLE users
RENAME COLUMN displayname TO display_name;

ALTER TABLE users
RENAME COLUMN teamid TO team_id;
