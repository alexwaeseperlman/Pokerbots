use core::panic;
use std::{
    cmp::{min, Ordering},
    future::Future,
    path::PathBuf,
};

use itertools::Itertools;
use rand::{seq::SliceRandom, Rng};

use super::hands::{self, Card, Suite};

pub trait Player {
    fn get_action(&self, state: GameState) -> Box<dyn Future<Output = Action>>;
    fn post_response(&self, state: GameState, response: EngineResponse);
    fn post_action(&self, state: GameState, action: Action, player: usize);
}

#[derive(PartialEq)]
pub enum Action {
    // Call and check are the same
    Call,
    Check,
    Raise { amt: u32 },
    Fold,
}

#[derive(Clone, PartialEq)]
pub enum Round {
    PreFlop,
    Flop,
    Turn,
    River,
    End,
}
pub enum EngineResponse {}

#[derive(Clone)]
pub struct PlayerState {
    stack: u32,
    hole_cards: Vec<Card>,
    pushed: u32,
    // Did the player act yet in the current betting round
    acted: bool,
}

#[derive(Clone)]

pub struct GameState {
    // Cards in the deck
    pub deck: Vec<Card>,
    player_states: [PlayerState; 2],
    // Amount of money each player has bet in the current round
    community_cards: Vec<Card>,
    round: Round,
    button: bool,
    // The index of the player who was the last aggressor
    // If no player has raised then this is the non-button player
    // Using bool instead of usize because there are only 2 players
    last_aggressor: bool,
    // The amount of money the next player to act must push to call
    target_push: u32,
}

pub enum RoundResult {
    Accept,
    End { payouts: Vec<u32> },
}

pub enum GameError {
    InvalidCheck,
    Raise0,
    GameOver,
}

impl GameState {
    pub fn new(stacks: &[u32; 2], button: bool) -> GameState {
        if stacks[0] == 0 || stacks[1] == 0 {
            panic!("Stacks must be greater than 0");
        }
        let mut out = Self {
            deck: vec![],
            community_cards: vec![],
            round: Round::PreFlop,
            button: button,
            last_aggressor: !button,
            target_push: 2,
            player_states: stacks.map(|stack: u32| PlayerState {
                hole_cards: vec![],
                stack: stack.clone(),
                acted: false,
                pushed: 0,
            }),
        };
        out.deck = Vec::new();
        out.deck.reserve(52);
        for value in 1..=13 {
            out.deck.push(Card {
                value,
                suite: Suite::Clubs,
            });
            out.deck.push(Card {
                value,
                suite: Suite::Spades,
            });
            out.deck.push(Card {
                value,
                suite: Suite::Hearts,
            });
            out.deck.push(Card {
                value,
                suite: Suite::Diamonds,
            });
        }
        out.deck.shuffle(&mut rand::thread_rng());

        for player in 0..2 {
            out.player_states[player]
                .hole_cards
                .push(out.deck.pop().unwrap());
            out.player_states[player]
                .hole_cards
                .push(out.deck.pop().unwrap());
        }

        // Pay little and big blinds
        out.player_states[button as usize].pushed = 1;
        out.player_states[!button as usize].pushed = min(2, stacks[!button as usize]);

        // If the big blind is all in then the small blind does not have to call
        out.target_push = out.player_states[!button as usize].pushed;

        out
    }

    pub fn should_act(&self, pos: bool) -> bool {
        !self.player_states[pos as usize].acted
            || self.player_states[pos as usize].pushed < self.target_push
    }

    // Returns the index of the player who is acting next
    // Starts to the left of the button and goes clockwise
    // until finding a player who has not folded, and either has not acted
    // yet, or has acted, is not all in, and has not covered the highest bet
    // If the round is PreFlop then it starts two seats to the left of the button
    // Returns None if the betting round is over
    pub fn whose_turn(&self) -> Option<usize> {
        let pos = if self.round == Round::PreFlop {
            self.button
        } else {
            !self.button
        };

        if self.should_act(pos) {
            Some(pos as usize)
        } else if self.should_act(!pos) {
            Some(!pos as usize)
        } else {
            None
        }
    }

    pub fn round_over(&self) -> bool {
        !self.should_act(false) && !self.should_act(true)
    }

    pub fn get_player_hand(&self, player: bool) -> hands::Hand {
        let mut cards = self.community_cards.clone();
        cards.extend(self.player_states[player as usize].hole_cards.clone());
        hands::HandEval::best5(&cards)
    }

    pub fn post_action(self, action: Action) -> Result<GameState, GameError> {
        if self.round == Round::End {
            return Err(GameError::GameOver);
        }
        let mut out: GameState = self;
        let turn = out.whose_turn();
        if let Some(turn) = turn {
            // If a check is not possible then revert to folding
            if action == Action::Check && out.target_push > out.player_states[turn].pushed {
                return Err(GameError::InvalidCheck);
            }
            // Raise amount must be positive
            if let Action::Raise { amt } = action {
                if amt == 0 {
                    return Err(GameError::Raise0);
                }
            }
            match action {
                Action::Check => {
                    // Do nothing
                }
                Action::Call => {
                    let to_call = min(out.target_push, out.player_states[turn].stack)
                        - out.player_states[turn].pushed;

                    out.player_states[turn].pushed += to_call;
                }
                Action::Raise { amt } => {
                    let added = min(
                        out.target_push + amt - out.player_states[turn].pushed,
                        out.player_states[turn].stack - out.player_states[turn].pushed,
                    );
                    out.player_states[turn].pushed += added;
                    if added > out.target_push {
                        out.last_aggressor = turn == 1;
                    }
                    out.target_push += added;
                }
                Action::Fold => {
                    // If the player folds then they lose all of their pushed chips
                    // Set the round to End
                    out.player_states[turn].stack -= out.player_states[turn].pushed;
                    out.player_states[1 - turn].stack += out.player_states[turn].pushed;
                    out.round = Round::End;
                    return Ok(out);
                }
            }
            out.player_states[turn].acted = true;
        }
        if out.round_over() {
            out.player_states.iter_mut().for_each(|ps| {
                ps.acted = false;
            });
            match out.round {
                Round::PreFlop => {
                    out.round = Round::Flop;
                    out.community_cards.push(out.deck.pop().unwrap());
                    out.community_cards.push(out.deck.pop().unwrap());
                    out.community_cards.push(out.deck.pop().unwrap());
                }
                Round::Flop => {
                    out.round = Round::Turn;
                    out.community_cards.push(out.deck.pop().unwrap());
                }
                Round::Turn => {
                    out.round = Round::River;
                    out.community_cards.push(out.deck.pop().unwrap());
                }
                Round::River => {
                    out.round = Round::End;
                    // Calculate payout
                    match out.get_player_hand(false).cmp(&out.get_player_hand(true)) {
                        Ordering::Equal => {
                            // Players get back what they put in
                            // So do nothing
                        }
                        Ordering::Greater => {
                            // Player 1 wins
                            out.player_states[0].stack += out.player_states[1].pushed;
                            out.player_states[1].stack -= out.player_states[1].pushed;
                        }
                        Ordering::Less => {
                            // Player 2 wins
                            out.player_states[1].stack += out.player_states[0].pushed;
                            out.player_states[0].stack -= out.player_states[0].pushed;
                        }
                    }
                }
                Round::End => return Err(GameError::GameOver),
            }
        }
        Ok(out)
    }
}
pub struct Game {
    pub state: GameState,
    pub stacks: Vec<u32>,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn card_draw_works() {
        let state1 = GameState::new(&[50, 50], false);
        let state2 = GameState::new(&[50, 50], false);
        assert_ne!(state1.deck, state2.deck);
        assert_eq!(state1.deck.len(), 48);
        assert_eq!(state2.deck.len(), 48);
    }
}
