
ALTER TABLE teams
ADD CONSTRAINT owner_isUserId
FOREIGN KEY (owner) REFERENCES users(email)
ON DELETE CASCADE;


ALTER TABLE users
ADD CONSTRAINT users_teamid_fkey
FOREIGN KEY (team_id) REFERENCES teams(id)
ON DELETE SET NULL;


ALTER TABLE team_invites
ADD CONSTRAINT team_invites_teamid_fkey
FOREIGN KEY (teamID) REFERENCES teams(id)
ON DELETE CASCADE;

