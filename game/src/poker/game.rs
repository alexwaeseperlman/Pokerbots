use std::{future::Future, path::PathBuf};

use rand::Rng;

use super::hands::{Card, Suite};

pub trait Player {
    fn get_action(&self, state: GameState) -> Box<dyn Future<Output = Action>>;
    fn post_response(&self, state: GameState, response: EngineResponse);
    fn post_action(&self, state: GameState, action: Action, player: usize);
}
#[derive(Debug, Clone)]
pub struct Bot {
    team_name: String,
    path: PathBuf,
    build_cmd: Option<String>,
    run_cmd: Option<String>,
}
pub enum Action {
    Check,
    Call,
    Raise { amt: usize },
    Fold,
}

pub enum Round {
    Undealt,
    PreFlop,
    Flop,
    Turn,
    River,
}
pub enum EngineResponse {}
pub struct PlayerState {
    stack: usize,
    hole_cards: Vec<Card>,
    folded: bool,
}

pub struct GameState {
    // Cards in the deck
    deck: Vec<Card>,
    player_states: Vec<PlayerState>,
    // Amount of money each player has bet in the current round
    pushed: Vec<usize>,
    community_cards: Vec<Card>,
    round: Round,
    // Pot size including the amount each player bet in the current round
    pot_size: usize,
    button: usize,
}

impl GameState {
    pub fn new(stacks: &Vec<usize>, button: usize) -> GameState {
        let mut out = Self {
            deck: vec![],
            community_cards: vec![],
            round: Round::Undealt,
            pot_size: 0,
            button: button,
            player_states: stacks
                .into_iter()
                .map(|stack: &usize| PlayerState {
                    hole_cards: vec![],
                    stack: stack.clone(),
                    folded: false,
                })
                .collect(),
            pushed: vec![],
        };
        out.reset_deck();
        out
    }

    pub fn reset_deck(&mut self) {
        self.deck = Vec::new();
        self.deck.reserve(52);
        for value in 1..=13 {
            self.deck.push(Card {
                value,
                suite: Suite::Clubs,
            });
            self.deck.push(Card {
                value,
                suite: Suite::Spades,
            });
            self.deck.push(Card {
                value,
                suite: Suite::Hearts,
            });
            self.deck.push(Card {
                value,
                suite: Suite::Diamonds,
            });
        }
    }

    pub fn shuffle(&mut self) {
        // Fisher-Yates
        for i in (0..=51).rev() {
            let j = rand::thread_rng().gen_range(0..=i);
            self.deck.swap(i, j);
        }
    }

    pub fn get_small_blind(&self) -> usize {
        (self.button + 2) % self.player_states.len()
    }
    pub fn get_big_blind(&self) -> usize {
        (self.button + 1) % self.player_states.len()
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.deck.pop()
    }
}
pub struct Game {
    pub players: Vec<Box<dyn Player>>,
    pub state: GameState,
    pub stacks: Vec<usize>,
}

impl Game {
    pub fn new(stacks: Vec<usize>) -> Game {
        let mut out = Self {
            state: GameState::new(&stacks, 0),
            stacks: stacks,
            players: vec![],
        };
        out.state.reset_deck();
        out
    }
    async fn run_betting_round(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn card_draw_works() {
        let mut dealer = Game::new(vec![]);
        dealer.state.shuffle();
        assert!(dealer.state.deck.len() == 52);
        for i in 1..=10 {
            dealer.state.draw();
        }
        assert!(
            (dealer.state.deck.len() == 42),
            "deck length is {} but should be 42",
            dealer.state.deck.len()
        );
    }
    #[test]
    fn deck_is_full() {
        let mut dealer = Game::new(vec![]);
        dealer.state.reset_deck();
        let initial = dealer.state.deck.clone();
        dealer.state.shuffle();
        dealer.state.shuffle();
        assert!(
            dealer.state.deck != initial,
            "shuffled deck should not be equal to the initial deck"
        );
        let mut counts = vec![0; 52];
        for c in dealer.state.deck {
            counts[((c.value - 1) * 4
                + (match c.suite {
                    Suite::Clubs => 0,
                    Suite::Diamonds => 1,
                    Suite::Hearts => 2,
                    Suite::Spades => 3,
                })) as usize] += 1;
        }
        assert!(
            counts.into_iter().all(|x| x == 1),
            "All cards should only exist once"
        );
    }
}
