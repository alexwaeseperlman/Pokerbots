use itertools::Itertools;
use std::{cmp::Ordering, fmt::Display};

#[derive(PartialEq, Eq, Clone, Debug, Copy)]
pub enum Suite {
    Clubs,
    Spades,
    Hearts,
    Diamonds,
}

impl Display for Suite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            &match self {
                Self::Clubs => "c",
                Self::Spades => "s",
                Self::Hearts => "h",
                Self::Diamonds => "d",
            }
            .to_string(),
        )
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct Card {
    pub value: u32,
    pub suite: Suite,
}
impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            match self.value {
                1 => "A".to_string(),
                2..=9 => self.value.to_string(),
                10 => "T".to_string(),
                11 => "J".to_string(),
                12 => "Q".to_string(),
                13 => "K".to_string(),
                _ => panic!("Invalid card value"),
            },
            self.suite.to_string()
        )
    }
}

#[derive(Clone, Debug)]
pub struct Hand {
    pub cards: [Card; 5],
}

impl PartialEq for Hand {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for Hand {}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        hand_eval::compare_hands(&self.cards, &other.cards)
    }
}

pub mod hand_eval {
    use std::cmp::Ordering;

    use super::*;

    fn hand_value(hand: &[Card; 5]) -> (u8, Vec<u8>, Vec<u8>) {
        let mut hist = hand
            .iter()
            .counts_by(|c| if c.value == 1 { 14 } else { c.value })
            .iter()
            .map(|(k, v)| (u8::try_from(*v).unwrap(), u8::try_from(*k).unwrap()))
            .sorted()
            .rev()
            .collect_vec();
        // check low straight
        if hist == vec![(1, 14), (1, 5), (1, 4), (1, 3), (1, 2)] {
            hist = vec![(1, 5), (1, 4), (1, 3), (1, 2), (1, 1)];
        }

        (
            if hist.len() < 5 {
                (hist[0].0 + hist[1].0 == 5) as u8 * 4
            } else {
                (hand.map(|c| c.suite).iter().all_equal()) as u8 * 3
                    + (hist[0].1 == hist[4].1 + 4) as u8 * 2
            },
            hist.iter().map(|(k, _)| *k).collect(),
            hist.iter().map(|(_, v)| *v).collect(),
        )
    }

    pub fn compare_hands(hand1: &[Card; 5], hand2: &[Card; 5]) -> Ordering {
        hand_value(hand1).cmp(&hand_value(hand2))
    }

    pub fn best5(hand: &Vec<Card>) -> Hand {
        if hand.len() < 5 {
            panic!("Not enough cards");
        }
        let h = hand
            .iter()
            .combinations(5)
            .max_by(|a, b| {
                let a = [*a[0], *a[1], *a[2], *a[3], *a[4]];
                let b = [*b[0], *b[1], *b[2], *b[3], *b[4]];
                compare_hands(&a, &b)
            })
            .unwrap();
        Hand {
            cards: [*h[0], *h[1], *h[2], *h[3], *h[4]],
        }
    }
    impl Card {
        pub(crate) fn from(code: &str) -> Card {
            let value = match code.chars().nth(0).unwrap() {
                'A' => 1,
                'T' => 10,
                'J' => 11,
                'Q' => 12,
                'K' => 13,
                _ => code.chars().nth(0).unwrap().to_digit(10).unwrap() as u32,
            };
            let suite = match code.chars().nth(1).unwrap() {
                'c' => Suite::Clubs,
                's' => Suite::Spades,
                'h' => Suite::Hearts,
                'd' => Suite::Diamonds,
                _ => panic!("Invalid suite"),
            };
            Self { value, suite }
        }
    }

    pub(crate) fn cards_from(code: &str) -> Vec<Card> {
        code.chars()
            .chunks(2)
            .into_iter()
            .map(|c| Card::from(c.collect::<String>().as_str()))
            .collect()
    }
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        pub fn hand_comparison() {
            // 100 randomly generated hands of 7 cards in order of strength
            // The current and previous algorithm agree on this ordering,
            // so we can have confidence that it is correct
            let hands = [
                "Js6d2c8h7d3s4s",
                "Jc3d4h5s2sTdKs",
                "3h2cKd7c9dJdTh",
                "Kd3dQc7h5c9c8c",
                "4c7sQc6cKs8cTd",
                "4c6d5dJdQcKc7c",
                "6dTsJsQs7d5hKc",
                "8dKdTcJs6dQs7s",
                "7c9dJhAc2c3s5s",
                "9d3dQcTs2cAc5s",
                "Ah3s5hQdJc2h6c",
                "AdJhTs6d9h4cQc",
                "9hKs7s6sTc4dAd",
                "7s4c2d2h6cJsTh",
                "4dTsQc2d2h7cJs",
                "5h8s6dJcAh2c2d",
                "4d5sTh3d3s6sKs",
                "TdKs5h3s3dQd4d",
                "2d9c3c4cAdKh3s",
                "6h8c4d9d4s7c2d",
                "8s4cTd7cAd6d4s",
                "AsKsQc4d9h4h2s",
                "8s5d5hQhKc6d7s",
                "Qs6c5h5cTsAs9h",
                "5cTdKd7cQsAd5s",
                "6c2s6d4hQs7dJs",
                "4c6d9h5s8hKc6h",
                "Qc2h6hKs7c4s6d",
                "4s2d9sAcJs6c6d",
                "Ks8s6dJc4h6sAh",
                "Jc3d7hTsQs7c8d",
                "2s7cAc3c7s8h4c",
                "JdKs3c8c8d4s7d",
                "Ks6d4d8s8c3dQh",
                "8cJd8d6c7sKcQc",
                "8dJd3d2hAsTd8s",
                "Qd5sAh8c8d6d2d",
                "8hQh8dTsAd3c4h",
                "8c9dAd6c8sKdJs",
                "5c4c8d9c9h2hQs",
                "9cTd4s6h8s9sQd",
                "Kc9cQd4d9h3d2s",
                "9hAh4dTh3sJc9s",
                "5h7c2dThTd9h6h",
                "Ts7d9c2sJd3dTd",
                "8s3cQs7hTcTh4c",
                "5s8hTd7d3cTcKd",
                "Kc8hAc7sQdTcTd",
                "9sJd8cJc3h6c4s",
                "JdKsAcJc8d3cTc",
                "Qh5d8d2sQcJh3s",
                "Qh3c7cKhQs4s8h",
                "Kh5c8dKd3h2s7c",
                "Td2cKh7d5h3hKc",
                "2c3dQsKcKdJd9d",
                "Ks2cKcAh8hQhTd",
                "5cTdAh9h4d6dAd",
                "AsAcTs6d4h9dJc",
                "3dAcQhAh5c6hJh",
                "AcKdAdTcJc6h3s",
                "6hAh9dAcQdKh5h",
                "4cKsTsJs2h4h2c",
                "Jh3h4d3c2dTs4c",
                "5hAd2d7h4c5c2c",
                "4d5hQhJd6c4h5d",
                "6s5c3c3d6hTdQh",
                "8d3h3sKh6c6d4s",
                "Kd9c6d4s4d6hQs",
                "6h5cQh5s6s9dTc",
                "4c4d8d5c8s7c5d",
                "6h6d4d7c9h9c8d",
                "Tc3c9s6h9h8c6d",
                "Js8h6d6c9d2c9c",
                "4hJsTcTd3h2s3c",
                "Qc2h3sJsAhJd3c",
                "Jc6hJs7d8cTc8d",
                "Qh7c9hQc9c5d4c",
                "Kh5h9c3d3hKdTh",
                "8cQsKd3dKc3c4h",
                "9s7s4c7hKd3dKc",
                "5hKd8sKsQsJh8c",
                "KsQhKd9dTsQc5d",
                "Ac2dAh5s5h",
                "Jd8h3sJcAcAdQh",
                "9s5sJdQhQsQc2d",
                "2cKhKs4c7h5sKd",
                "TdKs5hKcKd2c6s",
                "Ac7dAhAs6s5dTh",
                "4sAhTdAdJsAc9c",
                "2cKh6h8c4d5h3c",
                "As9s5s8h7s6d5c",
                "2s8d3c9cQcJcTh",
                "Jc3s2s5s8sQdTs",
                "Th7s2d5hJhQh4h",
                "6cTc2c9cKc4d2s",
                "Qs8h7s2sJcKs6s",
                "Jh7h7c2h8h3sAh",
                "6h6d2cAc5h2h2s",
                "4s9d9s9h8s2s4c",
                "TsQhThJdTd4cJh",
                "As7cAh5s2c2sAc",
            ];
            let hands_order = hands.map(|a| best5(&cards_from(a)));

            for a in 0..hands_order.len() {
                for b in 0..hands_order.len() {
                    println!("{} vs {}", hands[a], hands[b]);
                    println!(
                        "{:?} vs {:?}",
                        hand_value(&hands_order[a].cards),
                        hand_value(&hands_order[b].cards)
                    );
                    assert_eq!(
                        hands_order[a].cmp(&hands_order[b]),
                        a.cmp(&b),
                        "hand order between {} and {} should be the same as given",
                        hands[a],
                        hands[b]
                    )
                }
            }
        }
    }
}
