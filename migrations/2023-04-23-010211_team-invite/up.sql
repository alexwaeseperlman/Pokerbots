CREATE TABLE team_invites (
    id SERIAL PRIMARY KEY NOT NULL,
    teamID INTEGER NOT NULL,
    invite_code BIGINT NOT NULL,
    FOREIGN KEY(teamID) REFERENCES teams(id),
    -- use bigint for utc time stamp
    expires BIGINT NOT NULL,
    used BOOLEAN NOT NULL DEFAULT false
);