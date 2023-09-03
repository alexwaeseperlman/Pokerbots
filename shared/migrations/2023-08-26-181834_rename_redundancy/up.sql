ALTER TABLE team_invites RENAME COLUMN teamid TO team;
ALTER TABLE team_invites RENAME COLUMN invite_code TO code;
ALTER TABLE teams RENAME COLUMN team_name TO name;
ALTER TABLE users RENAME COLUMN team_id TO team;