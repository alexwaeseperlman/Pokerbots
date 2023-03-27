use log::{error, info};
use rand::Rng;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;

pub static SOCKET_PATH: &str = "/tmp/sock";

pub struct Bot {
    team_name: String,
    path: std::path::PathBuf,
    build_cmd: Option<String>,
    run_cmd: Option<String>,
}

enum Suite {
    Clubs,
    Spades,
    Hearts,
    Diamonds,
}

struct Card {
    value: u32,
    suite: Suite,
}

pub struct Dealer {
    deck: Vec<Card>,
}

pub struct Game {
    bots: Vec<Bot>,
    num_players: u32,
    engine: Dealer,
    button: u32,
    pot: u32,
    hole_cards: Vec<Card>,
    community_cards: Vec<Card>,
}

impl Bot {
    fn build(&self) -> std::io::Result<()> {
        if self.build_cmd.is_some() {
            std::process::Command::new(self.build_cmd.as_ref().unwrap()).spawn()?;
        }
        info!("Bot built: {}", self.team_name);
        Ok(())
    }

    fn run(&self) -> std::io::Result<()> {
        std::process::Command::new(self.run_cmd.
                                   as_ref().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Other, "Run command failed to parse")
        })?)
        .spawn()?;
        info!("Bot ran: {}", self.team_name);
        Ok(())
    }

    fn connect(&self) -> std::io::Result<()> {
        let socket = Path::new(SOCKET_PATH);

        let mut stream = UnixStream::connect(socket)?;

        stream.write_all(b"Hello, world!")?;

        stream.shutdown(std::net::Shutdown::Write)?;

        let mut response = String::new();
        stream.read_to_string(&mut response)?;
        println!("Response: {}", response);
        Ok(())
    }
}

impl Dealer {
    fn new() -> Self {
        let mut deck = Vec::new();
        deck.reserve(52);
        for value in 1..=13 {
            deck.push(Card {
                value,
                suite: Suite::Clubs,
            });
            deck.push(Card {
                value,
                suite: Suite::Spades,
            });
            deck.push(Card {
                value,
                suite: Suite::Hearts,
            });
            deck.push(Card {
                value,
                suite: Suite::Diamonds,
            });
        }
        return Self { deck };
    }

    fn shuffle(&mut self) {
        // Fisher-Yates
        for i in 51..=1 {
            let j = rand::thread_rng().gen_range(0..=i);
            self.deck.swap(i, j);
        }
    }

    fn deal(&mut self) -> Option<Card> {
        self.deck.pop()
    }
}

impl Game {
    fn add_bot(&mut self, bot: Bot) {
        self.bots.push(bot);
        self.num_players = self.num_players + 1;
    }

    fn run(&self) {
        for bot in &self.bots {
            bot.build();
            bot.run();
        }
    }

    fn start_server(&self) -> std::io::Result<()> {
        let socket = Path::new(SOCKET_PATH);
        if socket.exists() {
            std::fs::remove_file(socket)?;
        }

        let listener = UnixListener::bind(socket)?;

        info!("Server started");
        loop {
            let (unix_stream, _addr) = listener.accept().expect("Failed to accept connection");
            if let Err(e) = self.handle(unix_stream) {
                error!("Failed to handle stream: {}", e);
                break;
            }
        }
        Ok(())
    }

    fn handle(&self, mut stream: UnixStream) -> std::io::Result<()> {
        let mut message = String::new();
        stream.read_to_string(&mut message)?;
        info!("Received message: {}", message);

        stream.write_all(b"Hellooooo")?;
        Ok(())
    }
}
