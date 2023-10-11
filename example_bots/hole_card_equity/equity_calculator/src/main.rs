use futures::future::join_all;
use gameplay::poker::{
    game::GameState,
    hands::hand_eval,
    hands::Suite,
    hands::{self, Card},
};
use itertools::Itertools;
use rand::{rngs::StdRng, seq::IteratorRandom, thread_rng, Rng, SeedableRng};
use std::{fs::File, io::Write, sync::Arc};
use tokio::sync::Mutex;

fn get_deck() -> Vec<Card> {
    GameState::get_shuffled_deck(&mut thread_rng())
}

#[tokio::main]
async fn main() {
    // try all pocket hands
    let mut file = File::create("hands.csv").unwrap();
    writeln!(file, "lo,hi,suited,win,tie,loss").unwrap();
    let mut file: Arc<Mutex<File>> = Arc::new(Mutex::new(file));

    let mut handles = vec![];
    for i in 1..=13 {
        for j in i..=13 {
            for suited in if i == j {
                vec![false]
            } else {
                vec![false, true]
            } {
                let file = file.clone();
                handles.push(tokio::spawn(async move {
                    println!("{:?} {:?} {:?}", i, j, suited);
                    let mut rng = StdRng::from_entropy();
                    let hole = [
                        Card {
                            value: i,
                            suite: Suite::Spades,
                        },
                        Card {
                            value: j,
                            suite: if suited { Suite::Spades } else { Suite::Hearts },
                        },
                    ];

                    let deck = get_deck()
                        .into_iter()
                        .filter(|x| !hole.contains(x))
                        .collect_vec();
                    let mut wins = 0;
                    let mut losses = 0;
                    let mut ties = 0;
                    for board in deck
                        .clone()
                        .into_iter()
                        .combinations(5)
                        .choose_multiple(&mut rng, 1000)
                    {
                        let cur = deck
                            .clone()
                            .into_iter()
                            .filter(|x| !board.contains(x))
                            .collect_vec();
                        let best = [hole.to_vec(), board.clone()].concat();
                        let best = hand_eval::best5(&best);
                        let best = hand_eval::hand_value(&best.cards.try_into().unwrap());
                        for op_hole in cur.into_iter().combinations(2) {
                            let op_best = hand_eval::hand_value(
                                &hand_eval::best5(&[op_hole.to_vec(), board.clone()].concat())
                                    .cards
                                    .into(),
                            );
                            match best.0.cmp(&op_best.0) {
                                std::cmp::Ordering::Greater => wins += 1,
                                std::cmp::Ordering::Less => losses += 1,
                                std::cmp::Ordering::Equal => ties += 1,
                            }
                        }
                    }
                    println!(
                        "{:?} {:?} {:?}",
                        wins,
                        losses,
                        wins as f64 / (wins + losses) as f64
                    );
                    let mut file = file.lock().await;
                    writeln!(
                        file,
                        "{},{},{},{},{},{}",
                        i, j, suited as u32, wins, ties, losses
                    )
                    .unwrap();
                }));
            }
        }
    }
    join_all(handles).await;
}
