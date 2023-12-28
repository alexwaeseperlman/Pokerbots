use std::fmt::Display;

use shared::WhichBot;

use shared::poker::game::HoleCards;
use shared::poker::{
    game::{EndReason, GameState, PlayerPosition, PlayerState},
    hands::Card,
};

pub enum EngineCommunication {
    StartGame,
    BettingState {
        sb_pushed: u32,
        sb_stack: u32,
        bb_pushed: u32,
        bb_stack: u32,
    },
    PreFlopCards(HoleCards, HoleCards),
    FlopCards([Card; 3]),
    TurnCard(Card),
    RiverCard(Card),
    EndGame {
        end_reason: EndReason,
        last_aggressor: PlayerPosition,
        sb_hole_cards: HoleCards,
        bb_hole_cards: HoleCards,
    },
}

impl EngineCommunication {
    pub fn get_betting_state(game_state: &GameState) -> EngineCommunication {
        EngineCommunication::BettingState {
            sb_pushed: game_state.player_states[0].pushed,
            sb_stack: game_state.player_states[0].stack,
            bb_pushed: game_state.player_states[1].pushed,
            bb_stack: game_state.player_states[1].stack,
        }
    }

    pub fn get_round_end(game_state: &GameState) -> EngineCommunication {
        let end_reason = match game_state.end_reason.as_ref().unwrap() {
            EndReason::LastToAct(winner) => EndReason::LastToAct(*winner),
            EndReason::WonShowdown(winner) => EndReason::WonShowdown(*winner),
            EndReason::Tie => EndReason::Tie,
        };
        EngineCommunication::EndGame {
            end_reason,
            last_aggressor: game_state.last_aggressor,
            sb_hole_cards: game_state.player_states[0].hole_cards.clone(),
            bb_hole_cards: game_state.player_states[1].hole_cards.clone(),
        }
    }

    pub fn render_for_bot(&self, position: PlayerPosition) -> String {
        match self {
            EngineCommunication::StartGame => {
                format!("START {}", position)
            }
            EngineCommunication::BettingState {
                bb_pushed,
                bb_stack,
                sb_pushed,
                sb_stack,
            } => match position {
                PlayerPosition::BigBlind => format!(
                    "STACK {} {} {} {}",
                    bb_pushed, bb_stack, sb_pushed, sb_stack
                ),
                PlayerPosition::SmallBlind => format!(
                    "STACK {} {} {} {}",
                    sb_pushed, sb_stack, bb_pushed, bb_stack
                ),
            },
            EngineCommunication::PreFlopCards(sb_cards, bb_cards) => match position {
                PlayerPosition::SmallBlind => {
                    format!("PREFLOP {} {}", sb_cards.0[0], sb_cards.0[1])
                }
                PlayerPosition::BigBlind => {
                    format!("PREFLOP {} {}", bb_cards.0[0], bb_cards.0[1])
                }
            },
            EngineCommunication::FlopCards(cards) => {
                format!("FLOP {} {} {}", cards[0], cards[1], cards[2])
            }
            EngineCommunication::TurnCard(card) => format!("TURN {}", card),
            EngineCommunication::RiverCard(card) => format!("RIVER {}", card),
            EngineCommunication::EndGame {
                end_reason,
                sb_hole_cards,
                bb_hole_cards,
                last_aggressor,
            } => {
                let other_cards = match position {
                    PlayerPosition::BigBlind => sb_hole_cards,
                    PlayerPosition::SmallBlind => bb_hole_cards,
                };
                match end_reason {
                    EndReason::LastToAct(winner) => {
                        format!("END FOLD {}", winner.other())
                    }
                    EndReason::WonShowdown(winner) => {
                        // winner always shows cards
                        // loser shows cards if they are the last aggressor
                        if *winner == position && *last_aggressor != *winner {
                            format!(
                                "END SHOWDOWN WINNER {} SHOWN {} {}",
                                winner, other_cards.0[0], other_cards.0[1]
                            )
                        } else if *winner == position && *last_aggressor == *winner {
                            format!("END SHOWDOWN WINNER {} HIDDEN", winner)
                        } else {
                            format!(
                                "END SHOWDOWN WINNER {} SHOWN {} {}",
                                winner, other_cards.0[0], other_cards.0[1]
                            )
                        }
                    }
                    EndReason::Tie => {
                        format!("END SHOWDOWN TIE {} {}", other_cards[0], other_cards[1])
                    }
                }
            }
        }
    }
}

pub fn parse_action<T: AsRef<str>>(
    line: T,
) -> Result<shared::poker::game::Action, shared::GameActionError> {
    let line = line.as_ref();
    Ok(match line.as_ref() {
        "F" => shared::poker::game::Action::Fold,
        "C" => shared::poker::game::Action::Raise(0),
        _ => {
            if line.chars().nth(0) != Some('R') {
                Err(shared::GameActionError::CouldNotParse)?;
            }
            let amount = line[1..]
                .parse::<u32>()
                .map_err(|_| shared::GameActionError::CouldNotParse)?;
            shared::poker::game::Action::Raise(amount)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::parse_action;
    #[test]
    fn parse_action_check() {
        assert!(parse_action(&"X".to_owned()).is_err());
    }

    #[test]
    fn parse_action_fold() {
        assert_eq!(
            parse_action(&"F".to_owned()).unwrap(),
            shared::poker::game::Action::Fold
        );
    }

    #[test]
    fn parse_action_call() {
        assert_eq!(
            parse_action(&"C".to_owned()).unwrap(),
            shared::poker::game::Action::Raise(0)
        );
    }

    #[test]
    fn parse_action_raise() {
        assert_eq!(
            parse_action(&"R1234".to_owned()).unwrap(),
            shared::poker::game::Action::Raise(1234)
        );
    }

    #[test]
    fn parse_action_raise_invalid() {
        assert!(parse_action(&"R".to_owned()).is_err());
    }

    #[test]
    fn parse_action_raise_invalid2() {
        assert!(parse_action(&"R1234a".to_owned()).is_err());
    }

    #[test]
    fn parse_action_raise_invalid3() {
        assert!(parse_action(&"R-1234".to_owned()).is_err());
    }

    #[test]
    fn parse_action_raise_invalid4() {
        assert!(parse_action(&"R-1".to_owned()).is_err());
    }

    #[test]
    fn parse_action_raise_invalid5() {
        assert!(parse_action(&"R1234.0".to_owned()).is_err());
    }

    #[test]
    fn parse_action_raise_invalid6() {
        assert!(parse_action(&"B".to_owned()).is_err());
    }
}
