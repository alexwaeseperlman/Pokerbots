CREATE TABLE team_invites (
    invite_code TEXT UNIQUE NOT NULL PRIMARY KEY,
    teamID INTEGER NOT NULL,
    FOREIGN KEY(teamID) REFERENCES teams(id) ON DELETE CASCADE,
    -- use bigint for utc time stamp
    expires BIGINT NOT NULL
);