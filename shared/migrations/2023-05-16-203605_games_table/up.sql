CREATE TABLE games (
    id TEXT UNIQUE NOT NULL PRIMARY KEY,
    bot_a INTEGER NOT NULL,
    bot_b INTEGER NOT NULL,
    score_change INTEGER,
    created BIGINT NOT NULL DEFAULT extract(epoch from CURRENT_TIMESTAMP)
)