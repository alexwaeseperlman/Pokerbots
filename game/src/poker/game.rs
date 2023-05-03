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

#[derive(Clone, PartialEq, Debug)]
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

#[derive(Debug, PartialEq)]
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

        // Deal hole cards
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
        out.target_push = min(
            out.player_states[!button as usize].pushed,
            min(stacks[0], stacks[1]),
        );

        if out.target_push == 1 {
            out.showdown()
        } else {
            out
        }
    }

    pub fn should_act(&self, pos: bool) -> bool {
        (!self.player_states[pos as usize].acted
            || self.player_states[pos as usize].pushed < self.target_push)
            && self.round != Round::End
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
        hands::hand_eval::best5(&cards)
    }

    pub fn showdown(self) -> GameState {
        let mut out = self;
        // draw the rest of the cards
        while out.community_cards.len() < 5 {
            out.community_cards.push(out.deck.pop().unwrap());
        }
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
        out
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
                        out.target_push + amt,
                        min(
                            out.player_states[turn].stack,
                            out.player_states[1 - turn].stack,
                        ),
                    );
                    if added > out.player_states[turn].pushed {
                        out.last_aggressor = turn == 1;
                    }
                    out.player_states[turn].pushed = added;
                    out.target_push = added;
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
            // If someone is all in then skip straight to showdown
            if out.player_states[0].pushed == out.player_states[0].stack
                || out.player_states[1].pushed == out.player_states[1].stack
            {
                out.round = Round::End;
                out = out.showdown();
                return Ok(out);
            }
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
                    out = out.showdown()
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

    // GameState tests
    #[test]
    fn whose_turn_works() {
        let mut state = GameState::new(&[50, 50], false);

        // Start with the button
        assert_eq!(state.whose_turn(), Some(0));
        assert_eq!(state.round, Round::PreFlop);
        // It is the little blind's turn
        state = state.post_action(Action::Call).unwrap();
        assert_eq!(state.whose_turn(), Some(1));
        // It is the big blind's turn
        state = state.post_action(Action::Check).unwrap();

        assert_eq!(state.round, Round::Flop);
        assert_eq!(state.whose_turn(), Some(1));
        state = state.post_action(Action::Check).unwrap();
        assert_eq!(state.whose_turn(), Some(0));
        state = state.post_action(Action::Check).unwrap();

        assert_eq!(state.round, Round::Turn);
        assert_eq!(state.whose_turn(), Some(1));
        state = state.post_action(Action::Check).unwrap();
        assert_eq!(state.whose_turn(), Some(0));
        state = state.post_action(Action::Check).unwrap();

        assert_eq!(state.round, Round::River);
        assert_eq!(state.whose_turn(), Some(1));
        state = state.post_action(Action::Check).unwrap();
        assert_eq!(state.whose_turn(), Some(0));
        state = state.post_action(Action::Check).unwrap();
        // The round should be over
        assert_eq!(state.round, Round::End);

        // If the round is over then whose_turn should return None
        assert_eq!(state.whose_turn(), None);
    }

    #[test]
    fn raise_forces_call() {
        let mut state = GameState::new(&[50, 50], true);

        state = state.post_action(Action::Raise { amt: 10 }).unwrap();
        assert_eq!(state.player_states[1].pushed, 12);
        assert_eq!(state.player_states[0].pushed, 2);
        assert_eq!(state.target_push, 12);
        assert_eq!(state.whose_turn(), Some(0));
        state = state.post_action(Action::Call).unwrap();

        assert_eq!(state.player_states[0].pushed, 12);
        assert_eq!(state.player_states[1].pushed, 12);

        assert_eq!(state.target_push, 12);
        assert_eq!(state.whose_turn(), Some(0));
    }

    #[test]
    pub fn bb_raise_gives_sb_action() {
        let mut state = GameState::new(&[50, 50], false);

        state = state.post_action(Action::Call).unwrap();
        assert_eq!(state.player_states[0].pushed, 2);
        assert_eq!(state.player_states[1].pushed, 2);
        assert_eq!(state.target_push, 2);
        assert_eq!(state.whose_turn(), Some(1));
        // bb raises 10
        state = state.post_action(Action::Raise { amt: 10 }).unwrap();
        // target push is now 12
        assert_eq!(state.target_push, 12);
        assert_eq!(state.player_states[1].pushed, 12);
        // round is still pre-flop
        assert_eq!(state.round, Round::PreFlop);
        // sb should have the option to call or raise
        state = state.post_action(Action::Call).unwrap();
        // sb should have pushed 12
        assert_eq!(state.player_states[0].pushed, 12);
        // target push should be 12
        assert_eq!(state.target_push, 12);
        // round is now flop
        assert_eq!(state.round, Round::Flop);
        assert_eq!(state.whose_turn(), Some(1));

        // bb folds and sb wins
        state = state.post_action(Action::Fold).unwrap();
        assert_eq!(state.round, Round::End);

        // bb stack should be 50 - 12 = 38
        assert_eq!(state.player_states[1].stack, 38);
        // sb stack should be 50 + 12 = 62
        assert_eq!(state.player_states[0].stack, 62);
    }

    #[test]
    pub fn sb_raise_bb_fold_preflop() {
        let mut state = GameState::new(&[50, 50], false);

        state = state.post_action(Action::Raise { amt: 10 }).unwrap();
        assert_eq!(state.player_states[0].pushed, 12);
        assert_eq!(state.player_states[1].pushed, 2);
        assert_eq!(state.target_push, 12);
        assert_eq!(state.whose_turn(), Some(1));

        // bb cannot check, so this throws an error
        assert!(state.clone().post_action(Action::Check).is_err());

        // bb folds
        state = state.post_action(Action::Fold).unwrap();
        assert_eq!(state.round, Round::End);

        // sb stack should be 50 + 2 = 52
        assert_eq!(state.player_states[0].stack, 52);
        // bb stack should be 50 - 2 = 48
        assert_eq!(state.player_states[1].stack, 48);
    }

    #[test]
    pub fn all_in_limited_raise_skip_to_showdown() {
        let mut state = GameState::new(&[50, 40], false);

        // post flop we want to give the win to bb
        // so we give sb a worse hand
        // In this case we have sb with 3 twos
        // and bb with a flush (also 3 jacks)
        state.deck = hands::hand_eval::cards_from("2h3h9hJsQc");
        state.player_states[0].hole_cards = hands::hand_eval::cards_from("2s2c");
        state.player_states[1].hole_cards = hands::hand_eval::cards_from("QhTh");

        // split into paths a and b
        state = state.post_action(Action::Raise { amt: 100 }).unwrap();
        // sb should be limited to the bb stack size
        assert_eq!(state.player_states[0].pushed, 40);
        assert_eq!(state.player_states[1].pushed, 2);
        assert_eq!(state.target_push, 40);
        assert_eq!(state.whose_turn(), Some(1));

        // bb raises but they are already maxed
        state = state.post_action(Action::Raise { amt: 2 }).unwrap();
        assert_eq!(state.player_states[0].pushed, 40);
        assert_eq!(state.player_states[1].pushed, 40);
        assert_eq!(state.target_push, 40);

        // now we should be post-showdown since no one can act anymore
        assert_eq!(state.round, Round::End);
        // Since bb won the hand, they should have 50 + 40 = 90
        assert_eq!(state.player_states[1].stack, 80);
        // sb should have 50 - 40 = 10
        assert_eq!(state.player_states[0].stack, 10);
    }

    #[test]
    pub fn broke_bb_straight_to_showdown() {
        // can't control the deck so we run this multiple times
        for i in 0..10 {
            let mut state = GameState::new(&[50, 1], i % 2 == 1);

            assert_eq!(state.round, Round::End);

            // The bb should be all in, so the target push is 1
            // We should already be at showdown since no one can act
            // for the whole game
            assert_eq!(state.round, Round::End);
            assert_eq!(
                state.player_states[0].stack == 49,
                state.player_states[1].stack == 2
            );
        }
    }
}
