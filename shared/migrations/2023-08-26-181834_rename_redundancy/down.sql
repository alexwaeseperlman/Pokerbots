ALTER TABLE team_invites RENAME COLUMN team TO teamid;
ALTER TABLE team_invites RENAME COLUMN code TO invite_code;
ALTER TABLE teams RENAME COLUMN name TO team_name;
ALTER TABLE users RENAME COLUMN team TO team_id;
