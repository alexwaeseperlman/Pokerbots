use rand::Rng;
pub mod bots;
pub mod poker;

fn main() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let botA = std::env::args()
                .nth(1)
                .expect("first arg not set. should be link to botA");
            let botB = std::env::args()
                .nth(2)
                .expect("second arg not set. should be link to botB");
            let outUrl = std::env::args().nth(3).expect("outUrl");
            let client = reqwest::Client::new();

            // randomly choose score change for now
            let mut scoreChange = rand::thread_rng().gen_range(-50..=50);
            client
                .put(&outUrl)
                .body(scoreChange.to_string())
                .send()
                .await
                .unwrap()
        });
}
