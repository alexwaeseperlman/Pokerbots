ALTER TABLE games RENAME COLUMN defender_score TO score_change;
ALTER TABLE games DROP COLUMN challenger_score;
ALTER TABLE games DROP COLUMN error_bot;
ALTER TABLE games ADD COLUMN error_message TEXT;