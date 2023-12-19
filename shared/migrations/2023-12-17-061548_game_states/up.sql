CREATE TABLE game_states (
  game_id TEXT NOT NULL,
  step INTEGER NOT NULL,
  challenger_stack INTEGER NOT NULL,
  defender_stack INTEGER NOT NULL,
  challenger_pushed INTEGER NOT NULL,
  defender_pushed INTEGER NOT NULL,
  challenger_hand TEXT NOT NULL,
  defender_hand TEXT NOT NULL,
  flop TEXT,
  turn TEXT,
  river TEXT,
  button TEXT NOT NULL,
  sb TEXT NOT NULL,
  action_time INTEGER NOT NULL,
  last_action TEXT NOT NULL,
  PRIMARY KEY(game_id, step),
  FOREIGN KEY(game_id) REFERENCES games(id)
)
