CREATE TABLE teams (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    teamname TEXT NOT NULL CHECK(
        typeof("name") = "text" AND
        length("name") <= 20
    )
);
CREATE TABLE users (
    email TEXT PRIMARY KEY UNIQUE NOT NULL,
    teamID INTEGER,
    FOREIGN KEY(teamID) REFERENCES team(id)
)