CREATE TABLE game_states (
  game TEXT NOT NULL,
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
  round TEXT NOT NULL,
  action_time INTEGER NOT NULL,
  last_action TEXT NOT NULL,
  PRIMARY KEY(game, step),
  FOREIGN KEY(game) REFERENCES games(id)
)
