ALTER TABLE teams
RENAME COLUMN team_name TO teamname;

ALTER TABLE users
RENAME COLUMN display_name TO displayname;

ALTER TABLE users
RENAME COLUMN team_id TO teamid;
