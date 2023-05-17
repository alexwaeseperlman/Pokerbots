use std::process::Command;

use rand::Rng;
pub mod bots;
pub mod poker;

fn main() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let bota = std::env::args()
                .nth(1)
                .expect("first arg not set. should be link to botA");
            let bota = std::env::args()
                .nth(2)
                .expect("second arg not set. should be link to botB");
            let out_url = std::env::args().nth(3).expect("id");
            println!("Args good");

            // randomly choose score change for now
            let score_change = rand::thread_rng().gen_range(-50..=50);
            Command::new("curl")
                .arg(format!(
                    "https://{}?id={}&change={}",
                    std::env::var("API_URL").expect("API_URL not set"),
                    out_url,
                    score_change
                ))
                .output()
                .expect("Failed to reach outUrl");
        });
}
