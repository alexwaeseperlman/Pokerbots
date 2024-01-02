ALTER TABLE game_results DROP COLUMN error_bot;
UPDATE game_results SET error_type = NULL;