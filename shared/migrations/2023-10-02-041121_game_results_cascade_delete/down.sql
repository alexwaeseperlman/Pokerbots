ALTER TABLE game_results DROP CONSTRAINT game_results_id_fkey;
ALTER TABLE game_results ADD CONSTRAINT game_results_id_fkey FOREIGN KEY (id) REFERENCES games (id);