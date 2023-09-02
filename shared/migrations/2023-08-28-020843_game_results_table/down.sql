ALTER TABLE bots DROP COLUMN rating;
ALTER TABLE games DROP COLUMN defender_rating;
ALTER TABLE games DROP COLUMN challenger_rating;
ALTER TABLE bots ADD COLUMN score REAL NOT NULL DEFAULT 0;

ALTER TABLE games ADD COLUMN defender_score INT;
ALTER TABLE games ADD COLUMN challenger_score INT;
ALTER TABLE games ADD COLUMN error_type TEXT;
ALTER TABLE games ADD COLUMN error_bot INT;

UPDATE games SET defender_score = r.defender_score, challenger_score = r.challenger_score, error_type = r.error_type, error_bot = r.error_bot
FROM (
    SELECT id, defender_score, challenger_score, error_type, error_bot FROM game_results
) r
WHERE games.id = r.id;

DROP TABLE game_results;
