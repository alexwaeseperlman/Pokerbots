ALTER TABLE games RENAME COLUMN score_change TO defender_score;
ALTER TABLE games ADD COLUMN challenger_score INTEGER;
UPDATE games SET challenger_score = -defender_score;
ALTER TABLE games ADD COLUMN error_bot INTEGER;
ALTER TABLE games DROP COLUMN error_message;