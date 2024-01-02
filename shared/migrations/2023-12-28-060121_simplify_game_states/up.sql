DROP TABLE game_states;

CREATE TABLE game_states (
  game_id VARCHAR NOT NULL,
  step INTEGER NOT NULL,
  challenger_stack INTEGER NOT NULL,
  defender_stack INTEGER NOT NULL,
  challenger_pushed INTEGER NOT NULL,
  defender_pushed INTEGER NOT NULL,
  challenger_hand VARCHAR NOT NULL,
  defender_hand VARCHAR NOT NULL,
  community_cards VARCHAR NOT NULL,
  sb INTEGER NOT NULL,
  action_time INTEGER NOT NULL,
  whose_turn INTEGER,
  action_val VARCHAR NOT NULL,
  end_reason VARCHAR,
  PRIMARY KEY(game_id, step),
  FOREIGN KEY(game_id) REFERENCES games(id)
)