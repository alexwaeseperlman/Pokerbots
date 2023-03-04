CREATE TABLE teams (
    id SERIAL PRIMARY KEY NOT NULL,
    teamName TEXT UNIQUE NOT NULL CHECK(
        length(teamName) <= 20
    )
);
CREATE TABLE users (
    email TEXT PRIMARY KEY UNIQUE NOT NULL,
    displayName TEXT NOT NULL,
    teamID INTEGER,
    FOREIGN KEY(teamID) REFERENCES teams(id)
)