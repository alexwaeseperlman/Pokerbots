use std::fmt::Display;

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

pub mod HandEval {
    use std::cmp::Ordering;

    use super::*;

    pub fn card_mask(cards: Vec<Card>) -> u64 {
        let mut mask = 0;
        for card in cards {
            let suiteVal = match card.suite {
                Suite::Clubs => 0,
                Suite::Spades => 1,
                Suite::Hearts => 2,
                Suite::Diamonds => 3,
            };
            mask |= 1 << (card.value - 1 + suiteVal * 14);
            if card.value == 1 {
                mask |= 1 << (13 + suiteVal * 14);
            }
        }
        mask
    }

    pub fn fours(hand: u64) -> Option<u32> {
        // Subtract 2 from the mask because we only care about the higher representation of ace
        let mask = (hand & (hand >> 14) & (hand >> 28) & (hand >> 42)) & ((1 << 14) - 2);
        if mask == 0 {
            None
        } else {
            Some(mask.trailing_zeros() + 1)
        }
    }
    pub fn full_house(hand: u64) -> Option<u32> {
        let xor = (hand ^ (hand >> 14) ^ (hand >> 28) ^ (hand >> 42)) & ((1 << 14) - 2);
        let or = (hand | (hand >> 14) | (hand >> 28) | (hand >> 42)) & ((1 << 14) - 2);
        if xor.count_ones() == 1 && or.count_ones() == 2 && fours(hand) == None {
            Some(((xor).trailing_zeros() + 1) * 15 + (or ^ xor).trailing_zeros() + 1)
        } else {
            None
        }
    }
    pub fn threes(hand: u64) -> Option<u32> {
        // necessary and sufficient that there are exactly 3 cards whose frequency have odd parity
        // and exactly 3 cards
        let xor = (hand ^ (hand >> 14) ^ (hand >> 28) ^ (hand >> 42)) & ((1 << 14) - 2);
        let or = (hand | (hand >> 14) | (hand >> 28) | (hand >> 42)) & ((1 << 14) - 2);
        if or.count_ones() == 3 && xor.count_ones() == 3 {
            Some(
                (((hand & (hand >> 14)) | (hand & (hand >> 28)) | ((hand >> 14) & (hand >> 28)))
                    & ((1 << 14) - 2))
                    .trailing_zeros()
                    + 1,
            )
        } else {
            None
        }
    }

    pub fn two_pair(hand: u64) -> Option<u32> {
        // necessary and sufficient that there are exactly 3 unique cards,
        // two of which have frequency with even parity
        let xor = (hand ^ (hand >> 14) ^ (hand >> 28) ^ (hand >> 42)) & ((1 << 14) - 2);
        let or = (hand | (hand >> 14) | (hand >> 28) | (hand >> 42)) & ((1 << 14) - 2);
        if xor.count_ones() == 1 && or.count_ones() == 3 {
            let a = (xor ^ or).trailing_zeros();
            let b = (xor ^ or ^ (1 << a)).trailing_zeros();
            Some((b + 1) * 15 + a + 1)
        } else {
            None
        }
    }

    pub fn pair(hand: u64) -> Option<u32> {
        // necessary and sufficient that there are exactly 4 unique cards
        let xor = (hand ^ (hand >> 14) ^ (hand >> 28) ^ (hand >> 42)) & ((1 << 14) - 2);
        let or = (hand | (hand >> 14) | (hand >> 28) | (hand >> 42)) & ((1 << 14) - 2);
        if or.count_ones() == 4 {
            Some((xor ^ or).trailing_zeros() + 1)
        } else {
            None
        }
    }
    /*
    Test if a hand contains a straight. It accepts any consecutive set of values,
    and also the set '10 11 12 13 1', since Ace has the lowest and highest values
     */
    pub fn straight(hand: u64) -> Option<u32> {
        let counts = (hand | (hand >> 14) | (hand >> 28) | (hand >> 42)) & ((1 << 14) - 1);
        let x = counts & (counts >> 1) & (counts >> 2) & (counts >> 3) & (counts >> 4);
        if x == 0 {
            None
        } else {
            Some(x.trailing_zeros() + 1)
        }
    }
    pub fn flush(hand: u64) -> Option<u32> {
        let mask: u64 = (1 << 14) - 1;
        if (hand == hand & mask)
            || (hand == hand & (mask << 14))
            || (hand == hand & (mask << 28))
            || (hand == hand & (mask << 42))
        {
            Some(1)
        } else {
            None
        }
    }
    pub fn straight_flush(hand: u64) -> Option<u32> {
        if let Some(x) = flush(hand) {
            straight(hand)
        } else {
            None
        }
    }

    /*
    Used a loop here. It's possible to make it cute with bitmasks by using pdep to convert
    from base 2 to base 4, and adding all the suits together
    (i.e. deposit the cards from each suit onto the string 010101...01 (13 ones),
    sum, and compare the integers)
    */
    pub fn tie_break(hand1: &[Card; 5], hand2: &[Card; 5]) -> Ordering {
        let mut counts = [0; 15];
        for card in hand1 {
            counts[card.value as usize] += 1;
            if card.value == 1 {
                counts[14] += 1;
            }
        }

        for card in hand2 {
            counts[card.value as usize] -= 1;
            if card.value == 1 {
                counts[14] -= 1;
            }
        }
        for i in (2..=14).rev() {
            if counts[i] < 0 {
                println!("Break");
                return Ordering::Less;
            } else if counts[i] > 0 {
                return Ordering::Greater;
            }
        }
        println!("Didn't break?");
        return Ordering::Equal;
    }

    const hand_tests: [fn(u64) -> Option<u32>; 8] = [
        straight_flush,
        fours,
        full_house,
        flush,
        straight,
        threes,
        two_pair,
        pair,
    ];

    /*
    Compare two hands and output if the first is greater, equal, or less than the second
     */
    pub fn compare_hands(hand1: &[Card; 5], hand2: &[Card; 5]) -> Ordering {
        let bm1 = card_mask(hand1.to_vec());
        let bm2 = card_mask(hand2.to_vec());
        for test in hand_tests {
            let a = test(bm1);
            let b = test(bm2);
            if a.is_some() && b.is_some() {
                return match a.unwrap().cmp(&b.unwrap()) {
                    Ordering::Equal => tie_break(hand1, hand2),
                    _ => a.unwrap().cmp(&b.unwrap()),
                };
            } else if a.is_some() && b.is_none() {
                return Ordering::Greater;
            } else if a.is_none() && b.is_some() {
                return Ordering::Less;
            }
        }
        tie_break(hand1, hand2)
    }
    #[cfg(test)]
    mod tests {
        use super::super::*;
        use super::*;
        impl Card {
            pub fn from(code: &str) -> Card {
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
        #[test]
        pub fn flush_straight_A2345() {
            let mut hand = [
                Card::from("As"),
                Card::from("2s"),
                Card::from("3s"),
                Card::from("4s"),
                Card::from("5s"),
            ];
            // Ensure that it works for any permutation
            for i in 0..10 {
                let mask = card_mask(hand.to_vec());
                assert!(straight(mask) == Some(1), "should be a straight from A");
                assert!(flush(mask) == Some(1), "should be a flush");
                assert!(
                    straight_flush(mask) == Some(1),
                    "should be a straight flush"
                );
                assert!(fours(mask) == None, "should not be fours");
                assert!(full_house(mask) == None, "should not be full house");

                let n = rand::random::<usize>() % 5;
                hand.swap(n, 0);
            }
        }
        #[test]
        pub fn not_flush_straight_TJQKA() {
            let mut hand = [
                Card::from("Td"),
                Card::from("Jh"),
                Card::from("Qc"),
                Card::from("Kc"),
                Card::from("Ac"),
            ];
            // Ensure that it works for any permutation
            for i in 0..10 {
                let mask = card_mask(hand.to_vec());
                assert!(straight(mask) == Some(10), "should be a straight");
                assert!(flush(mask) == None, "should not be a flush");
                assert!(
                    straight_flush(mask) == None,
                    "should not be a straight flush"
                );
                assert!(fours(mask) == None, "should not be fours");
                assert!(full_house(mask) == None, "should not be full house");
                let n = rand::random::<usize>() % 5;
                hand.swap(n, 0);
            }
        }

        #[test]
        pub fn not_flush_not_straight_9JQKA() {
            let mut hand = [
                Card::from("9s"),
                Card::from("Jd"),
                Card::from("Qc"),
                Card::from("Ks"),
                Card::from("As"),
            ];
            // Ensure that it works for any permutation
            for i in 0..10 {
                let mask = card_mask(hand.to_vec());
                assert!(straight(mask) == None, "should not be a straight");
                assert!(flush(mask) == None, "should not be a flush");
                assert!(
                    straight_flush(mask) == None,
                    "should not be a straight flush"
                );
                assert!(full_house(mask) == None, "should not be full house");

                assert!(fours(mask) == None, "should not be fours");
                let n = rand::random::<usize>() % 5;
                hand.swap(n, 0);
            }
        }
        #[test]
        pub fn not_flush_not_straight_AA234() {
            let mut hand = [
                Card::from("Ad"),
                Card::from("Ah"),
                Card::from("2d"),
                Card::from("3d"),
                Card::from("4d"),
            ];
            // Ensure that it works for any permutation
            for i in 0..10 {
                let mask = card_mask(hand.to_vec());
                assert!(straight(mask) == None, "should not be a straight");
                assert!(flush(mask) == None, "should not be a flush");
                assert!(
                    straight_flush(mask) == None,
                    "should not be a straight flush"
                );
                assert!(fours(mask) == None, "should not be fours");
                assert!(full_house(mask) == None, "should not be full house");
                let n = rand::random::<usize>() % 5;
                hand.swap(n, 0);
            }
        }
        #[test]
        pub fn fours_AAAA4() {
            let mut hand = [
                Card::from("Ac"),
                Card::from("Ad"),
                Card::from("Ah"),
                Card::from("As"),
                Card::from("4d"),
            ];
            // Ensure that it works for any permutation
            for i in 0..10 {
                let mask = card_mask(hand.to_vec());

                assert!(straight(mask) == None, "should not be a straight");
                assert!(flush(mask) == None, "should not be a flush");
                assert!(fours(mask) == Some(14), "should be four aces");
                assert!(threes(mask) == None, "should not threes");
                assert!(full_house(mask) == None, "should not be full house");
                let n = rand::random::<usize>() % 5;
                hand.swap(n, 0);
            }
        }
        #[test]
        pub fn fours_2222A() {
            let mut hand = [
                Card::from("2d"),
                Card::from("2c"),
                Card::from("2h"),
                Card::from("2s"),
                Card::from("Ad"),
            ];
            // Ensure that it works for any permutation
            for i in 0..10 {
                let mask = card_mask(hand.to_vec());

                assert!(straight(mask) == None, "should not be a straight");
                assert!(flush(mask) == None, "should not be a flush");
                assert!(fours(mask) == Some(2), "should be four twos");
                assert!(full_house(mask) == None, "should not be full house");
                let n = rand::random::<usize>() % 5;
                hand.swap(n, 0);
            }
        }

        #[test]
        pub fn full_house_222AA() {
            let mut hand = [
                Card::from("2d"),
                Card::from("2c"),
                Card::from("2h"),
                Card::from("As"),
                Card::from("Ad"),
            ];
            // Ensure that it works for any permutation
            for i in 0..10 {
                let mask = card_mask(hand.to_vec());

                assert!(straight(mask) == None, "should not be a straight");
                assert!(flush(mask) == None, "should not be a flush");
                assert!(fours(mask) == None, "should not be four twos");
                assert!(
                    full_house(mask) == Some(2 * 15 + 14),
                    "should be full house with 3 2s and 2 aces"
                );
                let n = rand::random::<usize>() % 5;
                hand.swap(n, 0);
            }
        }
        #[test]
        pub fn full_house_AAA22() {
            let mut hand = [
                Card::from("Ad"),
                Card::from("Ac"),
                Card::from("Ah"),
                Card::from("2s"),
                Card::from("2d"),
            ];
            // Ensure that it works for any permutation
            for i in 0..10 {
                let mask = card_mask(hand.to_vec());

                assert!(straight(mask) == None, "should not be a straight");
                assert!(flush(mask) == None, "should not be a flush");
                assert!(threes(mask) == None, "should not be threes");
                assert!(fours(mask) == None, "should not be four twos");
                assert!(
                    full_house(mask) == Some(14 * 15 + 2),
                    "should be full house with 2 2s and 3 aces"
                );
                let n = rand::random::<usize>() % 5;
                hand.swap(n, 0);
            }
        }
        #[test]
        pub fn threes_AAA23() {
            let mut hand = [
                Card::from("Ad"),
                Card::from("Ac"),
                Card::from("Ah"),
                Card::from("2s"),
                Card::from("3d"),
            ];
            // Ensure that it works for any permutation
            for i in 0..10 {
                let mask = card_mask(hand.to_vec());

                assert!(straight(mask) == None, "should not be a straight");
                assert!(flush(mask) == None, "should not be a flush");
                assert!(threes(mask) == Some(14), "should be three aces");
                assert!(fours(mask) == None, "should not be fours");
                assert!(full_house(mask) == None, "should not be a full house");
                let n = rand::random::<usize>() % 5;
                hand.swap(n, 0);
            }
        }
        #[test]
        pub fn threes_KKK34() {
            let mut hand = [
                Card::from("Kd"),
                Card::from("Kc"),
                Card::from("Kh"),
                Card::from("3s"),
                Card::from("4d"),
            ];
            // Ensure that it works for any permutation
            for i in 0..10 {
                let mask = card_mask(hand.to_vec());

                assert!(straight(mask) == None, "should not be a straight");
                assert!(flush(mask) == None, "should not be a flush");
                assert!(threes(mask) == Some(13), "should be three kings");
                assert!(fours(mask) == None, "should not be fours");
                assert!(full_house(mask) == None, "should not be a full house");
                let n = rand::random::<usize>() % 5;
                hand.swap(n, 0);
            }
        }
        #[test]
        pub fn two_pair_AA334() {
            let mut hand = [
                Card::from("Ad"),
                Card::from("Ac"),
                Card::from("3h"),
                Card::from("3s"),
                Card::from("4d"),
            ];
            // Ensure that it works for any permutation
            for i in 0..10 {
                let mask = card_mask(hand.to_vec());

                assert!(straight(mask) == None, "should not be a straight");
                assert!(flush(mask) == None, "should not be a flush");
                assert!(threes(mask) == None, "should not be threes");
                assert!(pair(mask) == None, "should not be pair");
                println!("{:?}", two_pair(mask));
                assert!(
                    two_pair(mask) == Some(15 * 14 + 3),
                    "should be two pair of ace and 3"
                );
                assert!(fours(mask) == None, "should not be fours");
                assert!(full_house(mask) == None, "should not be a full house");
                let n = rand::random::<usize>() % 5;
                hand.swap(n, 0);
            }
        }
        #[test]
        pub fn two_pair_KK334() {
            let mut hand = [
                Card::from("Kd"),
                Card::from("Kc"),
                Card::from("3h"),
                Card::from("3s"),
                Card::from("4d"),
            ];
            // Ensure that it works for any permutation
            for i in 0..10 {
                let mask = card_mask(hand.to_vec());

                assert!(straight(mask) == None, "should not be a straight");
                assert!(flush(mask) == None, "should not be a flush");
                assert!(threes(mask) == None, "should not be threes");
                assert!(pair(mask) == None, "should not be pair");
                println!("{:?}", two_pair(mask));
                assert!(
                    two_pair(mask) == Some(15 * 13 + 3),
                    "should be two pair of king and 3"
                );
                assert!(fours(mask) == None, "should not be fours");
                assert!(full_house(mask) == None, "should not be a full house");
                let n = rand::random::<usize>() % 5;
                hand.swap(n, 0);
            }
        }
        #[test]
        pub fn two_pair_KKAA4() {
            let mut hand = [
                Card::from("Kd"),
                Card::from("Kc"),
                Card::from("Ah"),
                Card::from("As"),
                Card::from("4d"),
            ];
            // Ensure that it works for any permutation
            for i in 0..10 {
                let mask = card_mask(hand.to_vec());

                assert!(straight(mask) == None, "should not be a straight");
                assert!(flush(mask) == None, "should not be a flush");
                assert!(threes(mask) == None, "should not be threes");
                assert!(pair(mask) == None, "should not be pair");
                println!("{:?}", two_pair(mask));
                assert!(
                    two_pair(mask) == Some(15 * 14 + 13),
                    "should be two pair of ace and king"
                );
                assert!(fours(mask) == None, "should not be fours");
                assert!(full_house(mask) == None, "should not be a full house");
                let n = rand::random::<usize>() % 5;
                hand.swap(n, 0);
            }
        }
        #[test]
        pub fn pair_KKAQ4() {
            let mut hand = [
                Card::from("Kd"),
                Card::from("Kc"),
                Card::from("Ah"),
                Card::from("Qs"),
                Card::from("4d"),
            ];
            // Ensure that it works for any permutation
            for i in 0..10 {
                let mask = card_mask(hand.to_vec());

                assert!(straight(mask) == None, "should not be a straight");
                assert!(flush(mask) == None, "should not be a flush");
                assert!(threes(mask) == None, "should not be threes");
                assert!(pair(mask) == Some(13), "should be pair king");
                assert!(two_pair(mask) == None, "should not be two pair");
                assert!(fours(mask) == None, "should not be fours");
                assert!(full_house(mask) == None, "should not be a full house");
                let n = rand::random::<usize>() % 5;
                hand.swap(n, 0);
            }
        }
        #[test]
        pub fn pair_22AQ4() {
            let mut hand = [
                Card::from("2d"),
                Card::from("2c"),
                Card::from("Ah"),
                Card::from("Qs"),
                Card::from("4d"),
            ];
            // Ensure that it works for any permutation
            for i in 0..10 {
                let mask = card_mask(hand.to_vec());

                assert!(straight(mask) == None, "should not be a straight");
                assert!(flush(mask) == None, "should not be a flush");
                assert!(threes(mask) == None, "should not be threes");
                assert!(pair(mask) == Some(2), "should be pair 2");
                assert!(two_pair(mask) == None, "should not be two pair");
                assert!(fours(mask) == None, "should not be fours");
                assert!(full_house(mask) == None, "should not be a full house");
                let n = rand::random::<usize>() % 5;
                hand.swap(n, 0);
            }
        }
        #[test]
        pub fn pair_AAKQ4() {
            let mut hand = [
                Card::from("Ad"),
                Card::from("Kc"),
                Card::from("Ah"),
                Card::from("Qs"),
                Card::from("4d"),
            ];
            // Ensure that it works for any permutation
            for i in 0..10 {
                let mask = card_mask(hand.to_vec());

                assert!(straight(mask) == None, "should not be a straight");
                assert!(flush(mask) == None, "should not be a flush");
                assert!(threes(mask) == None, "should not be threes");
                assert!(pair(mask) == Some(14), "should be pair ace");
                assert!(two_pair(mask) == None, "should not be two pair");
                assert!(fours(mask) == None, "should not be fours");
                assert!(full_house(mask) == None, "should not be a full house");
                let n = rand::random::<usize>() % 5;
                hand.swap(n, 0);
            }
        }
        #[test]
        pub fn nothing_A2KQ4() {
            let mut hand = [
                Card::from("Ad"),
                Card::from("Kc"),
                Card::from("2h"),
                Card::from("Qs"),
                Card::from("4d"),
            ];
            // Ensure that it works for any permutation
            for i in 0..10 {
                let mask = card_mask(hand.to_vec());

                assert!(straight(mask) == None, "should not be a straight");
                assert!(flush(mask) == None, "should not be a flush");
                assert!(threes(mask) == None, "should not be threes");
                assert!(pair(mask) == None, "should not be pair");
                assert!(two_pair(mask) == None, "should not be two pair");
                assert!(fours(mask) == None, "should not be fours");
                assert!(full_house(mask) == None, "should not be a full house");
                let n = rand::random::<usize>() % 5;
                hand.swap(n, 0);
            }
        }
        #[test]
        pub fn nothing_2468T() {
            let mut hand = [
                Card::from("2d"),
                Card::from("4d"),
                Card::from("6d"),
                Card::from("8d"),
                Card::from("Ts"),
            ];
            // Ensure that it works for any permutation
            for i in 0..10 {
                let mask = card_mask(hand.to_vec());

                assert!(straight(mask) == None, "should not be a straight");
                assert!(flush(mask) == None, "should not be a flush");
                assert!(threes(mask) == None, "should not be threes");
                assert!(pair(mask) == None, "should not be pair");
                assert!(two_pair(mask) == None, "should not be two pair");
                assert!(fours(mask) == None, "should not be fours");
                assert!(full_house(mask) == None, "should not be a full house");
                let n = rand::random::<usize>() % 5;
                hand.swap(n, 0);
            }
        }
        #[test]
        pub fn hand_comparison() {
            let hands = [("AcAsAdTsTd", "AhThKhQhJh"), ("QcTc3h2c5d", "KdThQd9h2s")];
            for (lower, higher) in hands {
                let mut a = vec![];
                let mut b = vec![];
                for i in 0..5 {
                    a.push(Card::from(&lower[2 * i..2 * i + 2]));
                    b.push(Card::from(&higher[2 * i..2 * i + 2]));
                }
                for i in 0..10 {
                    assert_eq!(
                        compare_hands(
                            &a.clone().try_into().unwrap(),
                            &b.clone().try_into().unwrap()
                        ),
                        Ordering::Less
                    );
                    assert_eq!(
                        compare_hands(
                            &b.clone().try_into().unwrap(),
                            &a.clone().try_into().unwrap()
                        ),
                        Ordering::Greater
                    );
                    assert_eq!(
                        compare_hands(
                            &a.clone().try_into().unwrap(),
                            &a.clone().try_into().unwrap()
                        ),
                        Ordering::Equal
                    );
                    assert_eq!(
                        compare_hands(
                            &b.clone().try_into().unwrap(),
                            &b.clone().try_into().unwrap()
                        ),
                        Ordering::Equal
                    );
                    let m = rand::random::<usize>() % 5;
                    let n = rand::random::<usize>() % 5;
                    a.swap(m, 0);
                    b.swap(n, 0);
                }
            }
        }
    }
}
