ALTER TABLE games
RENAME COLUMN challenger TO bot_b;
ALTER TABLE games DROP CONSTRAINT challenger_fk_bots_id;

ALTER TABLE games
RENAME COLUMN defender TO bot_a;
ALTER TABLE games DROP CONSTRAINT defender_fk_bots_id;