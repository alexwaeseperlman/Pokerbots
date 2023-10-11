use core::panic;
use std::cmp::{min, Ordering};

use rand::{seq::SliceRandom, Rng};
use shared::GameActionError;

use super::hands::{self, Card, Suite};

#[derive(PartialEq, Debug)]
pub enum Action {
    // Call and check are the same as raising 0
    Raise(u32),
    Fold,
}

#[derive(Clone, PartialEq, Debug, Copy)]
pub enum Round {
    PreFlop,
    Flop,
    Turn,
    River,
    End,
}

#[derive(Clone, Debug)]
pub struct PlayerState {
    pub stack: u32,
    pub hole_cards: Vec<Card>,
    pub pushed: u32,
    // Did the player act yet in the current betting round
    pub acted: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum PlayerPosition {
    Button = 0,
    BigBlind = 1,
}
impl PlayerPosition {
    pub fn not(self) -> PlayerPosition {
        match self {
            PlayerPosition::Button => PlayerPosition::BigBlind,
            PlayerPosition::BigBlind => PlayerPosition::Button,
        }
    }
}

#[derive(Clone, Debug)]
pub enum WinReason {
    Showdown(PlayerPosition),
    OtherPlayerFolded(PlayerPosition),
    Tie,
}

#[derive(Clone, Debug)]
pub struct GameState {
    // Cards in the deck
    pub deck: Vec<Card>,
    pub player_states: [PlayerState; 2],
    // Amount of money each player has bet in the current round
    pub community_cards: Vec<Card>,
    pub round: Round,
    // The index of the player who was the last aggressor
    // If no player has raised then this is the non-button player
    // Using bool instead of usize because there are only 2 players
    pub last_aggressor: PlayerPosition,
    // The amount of money the next player to act must push to call
    pub target_push: u32,
    pub win_reason: Option<WinReason>,
}

pub enum RoundResult {
    Accept,
    End { payouts: Vec<u32> },
}

impl GameState {
    /// The button is always player 0
    pub fn new<T: Into<[u32; 2]>>(stacks: T, deck: Vec<Card>) -> GameState {
        let stacks: [u32; 2] = stacks.into();
        if stacks[0] == 0 || stacks[1] == 0 {
            panic!("Stacks must be greater than 0");
        }
        let mut out = Self {
            deck,
            community_cards: vec![],
            round: Round::PreFlop,
            last_aggressor: PlayerPosition::BigBlind,
            target_push: 2,
            player_states: stacks.map(|stack: u32| PlayerState {
                hole_cards: vec![],
                stack: stack.clone(),
                acted: false,
                pushed: 0,
            }),
            win_reason: None,
        };

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
        out.player_states[0].pushed = 1;
        out.player_states[1].pushed = min(2, stacks[1]);

        out.target_push = min(out.player_states[1].pushed, min(stacks[0], stacks[1]));

        out
    }

    pub fn get_deck() -> Vec<Card> {
        let mut out = Vec::new();
        out.reserve(52);
        for value in 1..=13 {
            out.push(Card {
                value,
                suite: Suite::Clubs,
            });
            out.push(Card {
                value,
                suite: Suite::Spades,
            });
            out.push(Card {
                value,
                suite: Suite::Hearts,
            });
            out.push(Card {
                value,
                suite: Suite::Diamonds,
            });
        }
        out
    }

    pub fn get_shuffled_deck<R: Rng>(rng: &mut R) -> Vec<Card> {
        let mut out = GameState::get_deck();
        out.shuffle(rng);
        out
    }
    pub fn should_act(&self, pos: PlayerPosition) -> bool {
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
    pub fn whose_turn(&self) -> Option<PlayerPosition> {
        let pos = if self.round == Round::PreFlop {
            PlayerPosition::Button
        } else {
            PlayerPosition::BigBlind
        };

        if self.should_act(pos) {
            Some(pos)
        } else if self.should_act(pos.not()) {
            Some(pos.not())
        } else {
            None
        }
    }

    pub fn round_over(&self) -> bool {
        !self.should_act(PlayerPosition::Button) && !self.should_act(PlayerPosition::BigBlind)
    }

    pub fn showdown(self) -> GameState {
        let mut out = self;
        // draw the rest of the cards
        while out.community_cards.len() < 5 {
            out.community_cards.push(out.deck.pop().unwrap());
        }
        out.round = Round::End;
        // Calculate payout
        match out
            .get_player_hand(PlayerPosition::Button)
            .cmp(&out.get_player_hand(PlayerPosition::BigBlind))
        {
            Ordering::Equal => {
                // Players get back what they put in
                // So do nothing
                out.win_reason = Some(WinReason::Tie);
            }
            Ordering::Greater => {
                // Player 0 wins
                out.player_states[0].stack += out.player_states[1].pushed;
                out.player_states[1].stack -= out.player_states[1].pushed;
                out.win_reason = Some(WinReason::Showdown(PlayerPosition::Button));
            }
            Ordering::Less => {
                // Player 1 wins
                out.player_states[1].stack += out.player_states[0].pushed;
                out.player_states[0].stack -= out.player_states[0].pushed;
                out.win_reason = Some(WinReason::Showdown(PlayerPosition::BigBlind));
            }
        }
        out
    }

    pub fn post_action(self, action: Action) -> Result<GameState, GameActionError> {
        if self.round == Round::End {
            return Err(GameActionError::GameOver);
        }
        let turn = self.whose_turn();
        let mut out: GameState = self;
        if let Some(turn) = turn {
            let mut cur_state = out.get_state(turn);
            let mut other_state = out.get_state(turn.not());
            match action {
                Action::Raise(amt) => {
                    let added = min(
                        out.target_push + amt,
                        min(cur_state.stack, other_state.stack),
                    );
                    if amt > 0 {
                        out.last_aggressor = turn;
                    }
                    cur_state.pushed = added;
                    out.target_push = added.max(out.target_push);
                    cur_state.acted = true;
                    out.set_state(turn, cur_state);
                    out.set_state(turn.not(), other_state);
                }
                Action::Fold => {
                    // If the player folds then they lose all of their pushed chips
                    // Set the round to End
                    cur_state.stack -= cur_state.pushed;
                    other_state.stack += cur_state.pushed;
                    out.round = Round::End;
                    out.set_state(turn, cur_state);
                    out.set_state(turn.not(), other_state);
                    out.win_reason = Some(WinReason::OtherPlayerFolded(turn.not()));
                    return Ok(out);
                }
            }
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
                    out = out.showdown()
                }
                Round::End => return Err(GameActionError::GameOver),
            }
        }
        Ok(out)
    }

    pub fn get_player_hand(&self, player: PlayerPosition) -> hands::Hand {
        let mut cards = self.community_cards.clone();
        cards.extend(self.player_states[player as usize].hole_cards.clone());
        hands::hand_eval::best5(&cards)
    }

    pub fn get_state(&self, player: PlayerPosition) -> PlayerState {
        self.player_states[player as usize].clone()
    }

    pub fn set_state(&mut self, player: PlayerPosition, state: PlayerState) {
        self.player_states[player as usize] = state
    }
}
#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use rand::{rngs::StdRng, SeedableRng};

    use crate::poker::{
        game::{Action, GameState, PlayerPosition, Round, WinReason},
        hands::{
            self,
            hand_eval::{self, cards_from},
        },
    };

    #[test]
    fn card_draw_works() {
        let mut rng = StdRng::from_seed([0; 32]);
        let state1 = GameState::new([50, 50], GameState::get_shuffled_deck(&mut rng));
        let state2 = GameState::new([50, 50], GameState::get_shuffled_deck(&mut rng));
        assert_ne!(state1.deck, state2.deck);
        assert_eq!(state1.deck.len(), 48);
        assert_eq!(state2.deck.len(), 48);
    }

    // GameState tests
    #[test]
    fn whose_turn_works() {
        let mut rng = StdRng::from_seed([0; 32]);
        let mut state = GameState::new([50, 50], GameState::get_shuffled_deck(&mut rng));

        // Start with the button
        assert!(matches!(state.whose_turn(), Some(PlayerPosition::Button)));
        assert_eq!(state.round, Round::PreFlop);
        // It is the little blind's turn
        state = state.post_action(Action::Raise(0)).unwrap();
        assert!(matches!(state.whose_turn(), Some(PlayerPosition::BigBlind)));
        // It is the big blind's turn
        state = state.post_action(Action::Raise(0)).unwrap();

        assert_eq!(state.round, Round::Flop);
        assert!(matches!(state.whose_turn(), Some(PlayerPosition::BigBlind)));
        state = state.post_action(Action::Raise(0)).unwrap();
        assert!(matches!(state.whose_turn(), Some(PlayerPosition::Button)));
        state = state.post_action(Action::Raise(0)).unwrap();

        assert_eq!(state.round, Round::Turn);
        assert!(matches!(state.whose_turn(), Some(PlayerPosition::BigBlind)));
        state = state.post_action(Action::Raise(0)).unwrap();
        assert!(matches!(state.whose_turn(), Some(PlayerPosition::Button)));
        state = state.post_action(Action::Raise(0)).unwrap();

        assert_eq!(state.round, Round::River);
        assert!(matches!(state.whose_turn(), Some(PlayerPosition::BigBlind)));
        state = state.post_action(Action::Raise(0)).unwrap();
        assert!(matches!(state.whose_turn(), Some(PlayerPosition::Button)));
        state = state.post_action(Action::Raise(0)).unwrap();
        // The round should be over
        assert_eq!(state.round, Round::End);

        // If the round is over then whose_turn should return None
        assert!(matches!(state.whose_turn(), None));
    }

    #[test]
    fn raise_forces_call() {
        let mut rng = StdRng::from_seed([0; 32]);
        let mut state = GameState::new([50, 50], GameState::get_shuffled_deck(&mut rng));

        state = state.post_action(Action::Raise(10)).unwrap();
        assert_eq!(state.player_states[0].pushed, 12);
        assert_eq!(state.player_states[1].pushed, 2);
        assert_eq!(state.target_push, 12);
        assert!(matches!(state.whose_turn(), Some(PlayerPosition::BigBlind)));
        assert_eq!(state.round, Round::PreFlop);
        state = state.post_action(Action::Raise(0)).unwrap();

        assert_eq!(state.round, Round::Flop);

        assert_eq!(state.player_states[1].pushed, 12);
        assert_eq!(state.player_states[0].pushed, 12);

        assert_eq!(state.target_push, 12);
        assert!(matches!(state.whose_turn(), Some(PlayerPosition::BigBlind)));
        assert_eq!(
            state
                .clone()
                .post_action(Action::Fold)
                .unwrap()
                .player_states[0]
                .stack,
            62
        );
        state = state.post_action(Action::Raise(0)).unwrap();
        assert_eq!(
            state
                .clone()
                .post_action(Action::Fold)
                .unwrap()
                .player_states[0]
                .stack,
            38
        );
    }

    #[test]
    pub fn bb_raise_gives_sb_action() {
        let mut rng = StdRng::from_seed([0; 32]);
        let mut state = GameState::new([50, 50], GameState::get_shuffled_deck(&mut rng));

        state = state.post_action(Action::Raise(0)).unwrap();
        assert_eq!(state.player_states[0].pushed, 2);
        assert_eq!(state.player_states[1].pushed, 2);
        assert_eq!(state.target_push, 2);
        assert!(matches!(state.whose_turn(), Some(PlayerPosition::BigBlind)));
        // bb raises 10
        state = state.post_action(Action::Raise(10)).unwrap();
        // target push is now 12
        assert_eq!(state.target_push, 12);
        assert_eq!(state.player_states[1].pushed, 12);
        // round is still pre-flop
        assert_eq!(state.round, Round::PreFlop);
        // sb should have the option to call or raise
        state = state.post_action(Action::Raise(0)).unwrap();
        // sb should have pushed 12
        assert_eq!(state.player_states[0].pushed, 12);
        // target push should be 12
        assert_eq!(state.target_push, 12);
        // round is now flop
        assert_eq!(state.round, Round::Flop);
        assert!(matches!(state.whose_turn(), Some(PlayerPosition::BigBlind)));

        // bb folds and sb wins
        state = state.post_action(Action::Fold).unwrap();
        assert_eq!(state.round, Round::End);

        // bb stack should be 50 - 12 = 38
        assert_eq!(state.player_states[1].stack, 38);
        // sb stack should be 50 + 12 = 62
        assert_eq!(state.player_states[0].stack, 62);

        assert!(matches!(
            state.win_reason,
            Some(WinReason::OtherPlayerFolded(PlayerPosition::Button))
        ));
    }

    #[test]
    pub fn sb_raise_bb_fold_preflop() {
        let mut rng = StdRng::from_seed([0; 32]);
        let mut state = GameState::new([50, 50], GameState::get_shuffled_deck(&mut rng));

        state = state.post_action(Action::Raise(10)).unwrap();
        assert_eq!(state.player_states[0].pushed, 12);
        assert_eq!(state.player_states[1].pushed, 2);
        assert_eq!(state.target_push, 12);
        assert!(matches!(state.whose_turn(), Some(PlayerPosition::BigBlind)));

        // bb folds
        state = state.post_action(Action::Fold).unwrap();
        assert_eq!(state.round, Round::End);

        // sb stack should be 50 + 2 = 52
        assert_eq!(state.player_states[0].stack, 52);
        // bb stack should be 50 - 2 = 48
        assert_eq!(state.player_states[1].stack, 48);

        assert!(matches!(
            state.win_reason,
            Some(WinReason::OtherPlayerFolded(PlayerPosition::Button))
        ));
    }

    #[test]
    pub fn all_in_limited_raise_dont_skip_to_showdown() {
        let mut rng = StdRng::from_seed([0; 32]);
        let mut state = GameState::new([50, 40], GameState::get_shuffled_deck(&mut rng));

        // post flop we want to give the win to bb
        // so we give sb a worse hand
        // In this case we have sb with 3 twos
        // and bb with a flush (also 3 jacks)
        state.deck = hands::hand_eval::cards_from("2h3h9hJsQc");
        state.player_states[0].hole_cards = hands::hand_eval::cards_from("2s2c");
        state.player_states[1].hole_cards = hands::hand_eval::cards_from("QhTh");

        state = state.post_action(Action::Raise(100)).unwrap();
        // sb should be limited to the bb stack size
        assert_eq!(state.player_states[0].pushed, 40);
        assert_eq!(state.player_states[1].pushed, 2);
        assert_eq!(state.target_push, 40);
        assert!(matches!(state.whose_turn(), Some(PlayerPosition::BigBlind)));

        // bb raises but they are already maxed
        state = state.post_action(Action::Raise(2)).unwrap();
        assert_eq!(state.player_states[0].pushed, 40);
        assert_eq!(state.player_states[1].pushed, 40);
        assert_eq!(state.target_push, 40);
        assert!(matches!(state.whose_turn(), Some(PlayerPosition::BigBlind)));

        // in a normal game now we would be at the showdown since no one can act anymore
        // however this engine should force the game to be played out, despite
        // the fact that no one can act
        assert_eq!(state.round, Round::Flop);

        // raising does nothing
        state = state.post_action(Action::Raise(5)).unwrap();
        assert_eq!(state.player_states[0].pushed, 40);
        assert_eq!(state.player_states[1].pushed, 40);
        assert_eq!(state.target_push, 40);
        assert!(matches!(state.whose_turn(), Some(PlayerPosition::Button)));
        state = state.post_action(Action::Raise(0)).unwrap();
        assert_eq!(state.round, Round::Turn);

        assert!(matches!(state.whose_turn(), Some(PlayerPosition::BigBlind)));
        state = state.post_action(Action::Raise(0)).unwrap();
        assert!(matches!(state.whose_turn(), Some(PlayerPosition::Button)));
        state = state.post_action(Action::Raise(0)).unwrap();
        assert_eq!(state.round, Round::River);

        assert!(matches!(state.whose_turn(), Some(PlayerPosition::BigBlind)));
        state = state.post_action(Action::Raise(0)).unwrap();
        assert!(matches!(state.whose_turn(), Some(PlayerPosition::Button)));
        state = state.post_action(Action::Raise(0)).unwrap();
        assert_eq!(state.round, Round::End);
        // Since bb won the hand, they should have 50 + 40 = 90
        assert_eq!(state.player_states[1].stack, 80);
        // sb should have 50 - 40 = 10
        assert_eq!(state.player_states[0].stack, 10);
        assert!(matches!(
            state.win_reason,
            Some(WinReason::Showdown(PlayerPosition::BigBlind))
        ));
    }

    #[test]
    pub fn broke_blind_straight_to_showdown() {
        for i in 0..20 {
            let mut rng = StdRng::from_seed([i; 32]);
            let mut state = GameState::new([50, 1], GameState::get_shuffled_deck(&mut rng));
            // make sure no one can bet anything
            assert_eq!(state.target_push, 1);
            assert_eq!(state.player_states[0].pushed, 1);
            assert_eq!(state.player_states[1].pushed, 1);
            {
                // If button folds then bb gets all the money
                let mut button_fold = state.clone();
                button_fold = button_fold.post_action(Action::Fold).unwrap();
                assert_eq!(button_fold.round, Round::End);
                assert_eq!(button_fold.player_states[0].stack, 49);
                assert_eq!(button_fold.player_states[1].stack, 2);
                assert!(matches!(
                    button_fold.win_reason,
                    Some(WinReason::OtherPlayerFolded(PlayerPosition::BigBlind))
                ));
            }
            {
                // If button raises then nothing happens
                let button_raise = state.clone().post_action(Action::Raise(2)).unwrap();
                assert_eq!(button_raise.round, Round::PreFlop);
                assert_eq!(button_raise.target_push, 1);
                assert_eq!(button_raise.player_states[0].pushed, 1);
            }
            // What if neither player can cover the blind
            state = GameState::new([1, 1], GameState::get_shuffled_deck(&mut rng));
            {
                // If button folds then bb gets all the money
                let mut button_fold = state.clone();
                button_fold = button_fold.post_action(Action::Fold).unwrap();
                assert_eq!(button_fold.round, Round::End);
                assert_eq!(button_fold.player_states[0].stack, 0);
                assert_eq!(button_fold.player_states[1].stack, 2);
                assert!(matches!(
                    button_fold.win_reason,
                    Some(WinReason::OtherPlayerFolded(PlayerPosition::BigBlind))
                ));
            }
            {
                // If button raises then nothing happens
                let button_raise = state.clone().post_action(Action::Raise(2)).unwrap();
                assert_eq!(button_raise.round, Round::PreFlop);
                assert_eq!(button_raise.target_push, 1);
                assert_eq!(button_raise.player_states[0].pushed, 1);
            }
            // skip to the end
            state = state.post_action(Action::Raise(0)).unwrap();
            {
                // if bb folds then button gets all the money
                let mut bb_fold = state.clone();
                bb_fold = bb_fold.post_action(Action::Fold).unwrap();
                assert_eq!(bb_fold.round, Round::End);
                assert_eq!(bb_fold.player_states[0].stack, 2);
                assert_eq!(bb_fold.player_states[1].stack, 0);
                assert!(matches!(
                    bb_fold.win_reason,
                    Some(WinReason::OtherPlayerFolded(PlayerPosition::Button))
                ));
            }
            state = state.post_action(Action::Raise(0)).unwrap();

            state = state.post_action(Action::Raise(0)).unwrap();
            state = state.post_action(Action::Raise(0)).unwrap();

            state = state.post_action(Action::Raise(0)).unwrap();
            state = state.post_action(Action::Raise(0)).unwrap();

            state = state.post_action(Action::Raise(0)).unwrap();
            state = state.post_action(Action::Raise(0)).unwrap();

            // The bb should be all in, so the target push is 1
            // We should already be at showdown since no one can act
            // for the whole game
            assert_eq!(state.round, Round::End);
            match (state.player_states[0].stack, state.player_states[1].stack) {
                (1, 1) => {
                    assert!(matches!(state.win_reason, Some(WinReason::Tie)))
                }
                (0, 2) => {
                    assert!(matches!(
                        state.win_reason,
                        Some(WinReason::Showdown(PlayerPosition::BigBlind))
                    ))
                }
                (2, 0) => {
                    assert!(matches!(
                        state.win_reason,
                        Some(WinReason::Showdown(PlayerPosition::Button))
                    ))
                }
                _ => panic!("stacks should be 1,1 or 0,2 or 2,0"),
            }
        }
    }

    #[test]
    pub fn all_in_on_turn_dont_skip_to_showdown() {
        // randomly choose the deck so we can test multiple times
        // we will check that the stacks are correct
        for i in 0..20 {
            let mut rng = StdRng::from_seed([i; 32]);
            let mut state = GameState::new([50, 50], GameState::get_shuffled_deck(&mut rng));

            // sb raises
            state = state.post_action(Action::Raise(4)).unwrap();
            // bb calls
            state = state.post_action(Action::Raise(0)).unwrap();

            // flop
            state = state.post_action(Action::Raise(0)).unwrap();
            state = state.post_action(Action::Raise(4)).unwrap();

            // it should be possible to have a bidding war here
            assert!(matches!(state.whose_turn(), Some(PlayerPosition::BigBlind)));
            state = state.post_action(Action::Raise(6)).unwrap();
            assert!(matches!(state.whose_turn(), Some(PlayerPosition::Button)));
            assert_eq!(state.round, Round::Flop);
            assert_eq!(state.target_push, 16);

            // call
            state = state.post_action(Action::Raise(0)).unwrap();

            // both players are in 16
            // bb goes all in in the turn
            // we should skip the river

            assert_eq!(state.round, Round::Turn);
            // turn
            assert!(matches!(state.whose_turn(), Some(PlayerPosition::BigBlind)));
            state = state.post_action(Action::Raise(100)).unwrap();
            assert_eq!(state.target_push, 50);
            assert_eq!(state.round, Round::Turn);
            // if sb folds then bb gets all the money
            {
                let mut sb_fold = state.clone();
                sb_fold = sb_fold.post_action(Action::Fold).unwrap();
                assert_eq!(sb_fold.round, Round::End);
                assert_eq!(sb_fold.player_states[0].stack, 34);
                assert_eq!(sb_fold.player_states[1].stack, 66);
            }
            state = state.post_action(Action::Raise(0)).unwrap();
            assert_eq!(state.round, Round::River);
            state = state.post_action(Action::Raise(0)).unwrap();
            state = state.post_action(Action::Raise(0)).unwrap();

            // River should have been skipped
            assert_eq!(state.round, Round::End);

            // Player with better hand should have 100, player with worse should have 0
            // Also last aggressor is bb
            assert!(matches!(state.last_aggressor, PlayerPosition::BigBlind));

            match (state.player_states[0].stack, state.player_states[1].stack) {
                (100, 0) => {
                    assert!(matches!(
                        state.win_reason,
                        Some(WinReason::Showdown(PlayerPosition::Button))
                    ))
                }
                (50, 50) => {
                    assert!(matches!(state.win_reason, Some(WinReason::Tie)))
                }
                (0, 100) => {
                    assert!(matches!(
                        state.win_reason,
                        Some(WinReason::Showdown(PlayerPosition::BigBlind))
                    ))
                }
                _ => panic!("stacks should be 100,0 or 0,100"),
            }
        }
    }
    #[test]
    pub fn better_hand_wins_showdown() {
        let mut rng = StdRng::from_seed(core::array::from_fn(|i| i as u8 + 1));
        for _ in 0..1000 {
            let mut state = GameState::new([20, 50], GameState::get_shuffled_deck(&mut rng));
            state = state.post_action(Action::Raise(49)).unwrap();
            state = state.post_action(Action::Raise(0)).unwrap();

            // check until the end
            state = state.post_action(Action::Raise(0)).unwrap();
            state = state.post_action(Action::Raise(0)).unwrap();

            state = state.post_action(Action::Raise(0)).unwrap();
            state = state.post_action(Action::Raise(0)).unwrap();

            state = state.post_action(Action::Raise(0)).unwrap();
            assert!(matches!(state.win_reason, None));
            state = state.post_action(Action::Raise(0)).unwrap();

            assert_eq!(state.round, Round::End);
            // Player with better hand should have 100, player with worse should have 0
            // If tied they should have equal amounts
            // Also last aggressor is sb
            assert!(matches!(state.last_aggressor, PlayerPosition::Button));

            let stacks = (state.player_states[0].stack, state.player_states[1].stack);
            match hand_eval::compare_hands(
                &state.get_player_hand(PlayerPosition::Button).cards,
                &state.get_player_hand(PlayerPosition::BigBlind).cards,
            ) {
                std::cmp::Ordering::Equal => {
                    assert_eq!(stacks, (20, 50));
                    assert!(matches!(state.win_reason, Some(WinReason::Tie)))
                }
                std::cmp::Ordering::Less => {
                    assert_eq!(stacks, (0, 70));
                    assert!(matches!(
                        state.win_reason,
                        Some(WinReason::Showdown(PlayerPosition::BigBlind))
                    ))
                }
                std::cmp::Ordering::Greater => {
                    assert_eq!(stacks, (40, 30));
                    assert!(matches!(
                        state.win_reason,
                        Some(WinReason::Showdown(PlayerPosition::Button))
                    ))
                }
            }
        }
    }

    #[test]
    pub fn snapshot_1() {
        /*
        99ms Defender <<< P 1
        99ms Challenger <<< P 0
        99ms Defender <<< C 7s 7h
        99ms Challenger <<< C Jc 9h
        99ms Challenger <<< S 2 1 2 48 52
        99ms Challenger >>> C
        99ms Defender <<< S 2 2 2 48 52
        99ms Defender >>> R5
        99ms Challenger <<< S 7 2 7 48 52
        99ms Challenger >>> C
        99ms Defender <<< C 7s 7h Qh 7d Qc
        99ms Challenger <<< C Jc 9h Qh 7d Qc
        99ms Defender <<< S 7 7 7 48 52
        99ms Defender >>> R5
        99ms Challenger <<< S 12 7 12 48 52
        99ms Challenger >>> C
        99ms Defender <<< C 7s 7h Qh 7d Qc 6h
        99ms Challenger <<< C Jc 9h Qh 7d Qc 6h
        99ms Defender <<< S 12 12 12 48 52
        99ms Defender >>> R5
        99ms Challenger <<< S 17 12 17 48 52
        99ms Challenger >>> C
        99ms Defender <<< C 7s 7h Qh 7d Qc 6h 4c
        99ms Challenger <<< C Jc 9h Qh 7d Qc 6h 4c
        99ms Defender <<< S 17 17 17 48 52
        99ms Defender >>> R5
        99ms Challenger <<< S 22 17 22 48 52
        99ms Challenger >>> C
         */
        let mut state = GameState::new(
            [48, 52],
            cards_from("Jc9h7s7hQh7dQc6h4c")
                .into_iter()
                .rev()
                .collect_vec(),
        );
        assert_eq!(state.player_states[0].hole_cards, cards_from("Jc9h"));
        assert_eq!(state.player_states[1].hole_cards, cards_from("7s7h"));

        state = state.post_action(Action::Raise(0)).unwrap();

        state = state.post_action(Action::Raise(5)).unwrap();

        state = state.post_action(Action::Raise(0)).unwrap();

        state = state.post_action(Action::Raise(5)).unwrap();

        state = state.post_action(Action::Raise(0)).unwrap();

        state = state.post_action(Action::Raise(5)).unwrap();

        state = state.post_action(Action::Raise(0)).unwrap();

        state = state.post_action(Action::Raise(5)).unwrap();

        assert!(matches!(state.win_reason, None));
        state = state.post_action(Action::Raise(0)).unwrap();

        assert_eq!(state.round, Round::End);
        assert!(matches!(state.last_aggressor, PlayerPosition::BigBlind));
        assert!(matches!(state.get_state(PlayerPosition::Button).stack, 26));
        assert!(matches!(
            state.get_state(PlayerPosition::BigBlind).stack,
            74
        ));
        assert!(matches!(
            state.win_reason,
            Some(WinReason::Showdown(PlayerPosition::BigBlind))
        ));
    }
}
