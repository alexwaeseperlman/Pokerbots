ALTER TABLE bots ADD COLUMN rating REAL NOT NULL DEFAULT 0;
ALTER TABLE bots DROP COLUMN score;
ALTER TABLE games ADD COLUMN defender_rating REAL NOT NULL DEFAULT 0;
ALTER TABLE games ADD COLUMN challenger_rating REAL NOT NULL DEFAULT 0;

CREATE TABLE game_results (
    id TEXT NOT NULL PRIMARY KEY REFERENCES games (id),
    challenger_rating_change REAL NOT NULL,
    defender_rating_change REAL NOT NULL,
    defender_score INT NOT NULL,
    challenger_score INT NOT NULL,
    error_type TEXT,
    error_bot INT,
    updated_at BIGINT NOT NULL DEFAULT extract(epoch from CURRENT_TIMESTAMP)
);

INSERT INTO game_results (id, challenger_rating_change, defender_rating_change, defender_score, challenger_score, error_type, error_bot, updated_at)
    SELECT id, 0, 0, defender_score, challenger_score, error_type, error_bot, created FROM games WHERE defender_score IS NOT NULL AND challenger_score IS NOT NULL;

ALTER TABLE games DROP COLUMN defender_score;
ALTER TABLE games DROP COLUMN challenger_score;
ALTER TABLE games DROP COLUMN error_type;
ALTER TABLE games DROP COLUMN error_bot;