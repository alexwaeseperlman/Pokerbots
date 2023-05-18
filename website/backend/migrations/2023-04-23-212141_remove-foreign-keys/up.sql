ALTER TABLE teams DROP CONSTRAINT owner_isUserId;
ALTER TABLE users DROP CONSTRAINT users_teamid_fkey;
ALTER TABLE team_invites DROP CONSTRAINT team_invites_teamid_fkey; 