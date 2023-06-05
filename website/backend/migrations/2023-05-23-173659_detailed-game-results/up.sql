CREATE TYPE game_error AS ENUM ('RUNTIME', 'COMPILE', 'TIMEOUT', 'MEMORY', 'UNKNOWN');
ALTER TABLE games ADD COLUMN error_type game_error;
ALTER TABLE games ADD COLUMN error_message TEXT;