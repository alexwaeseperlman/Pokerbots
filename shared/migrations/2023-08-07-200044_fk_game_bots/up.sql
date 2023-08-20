DELETE FROM games WHERE NOT EXISTS(SELECT NULL FROM bots b where b.id = bot_a);
DELETE FROM games WHERE NOT EXISTS(SELECT NULL FROM bots b where b.id = bot_b);
ALTER TABLE games
RENAME COLUMN bot_b TO challenger;
ALTER TABLE games ADD CONSTRAINT challenger_fk_bots_id FOREIGN KEY (challenger) REFERENCES bots(id) ON DELETE CASCADE;

ALTER TABLE games
RENAME COLUMN bot_a TO defender;
ALTER TABLE games ADD CONSTRAINT defender_fk_bots_id FOREIGN KEY (defender) REFERENCES bots(id) ON DELETE CASCADE;