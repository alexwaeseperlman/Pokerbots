use log::{debug, error, info};
use rand::Rng;
use std::{
    fs,
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

pub struct Game {
    bots: Vec<Bot>,
    num_players: u32,
    dealer: Dealer,
    button: u32,
    pot: u32,
    hole_cards: Vec<Card>,
    community_cards: Vec<Card>,
}

impl Bot {
    pub fn new(
        team_name: String,
        path: PathBuf,
        build_cmd: Option<String>,
        run_cmd: Option<String>,
    ) -> Bot {
        Self {
            team_name,
            path,
            build_cmd,
            run_cmd,
        }
    }

    /// builds files necesary for bot in subprocess
    fn build(&self) -> std::io::Result<()> {
        if self.build_cmd.is_some() {
            std::process::Command::new(self.build_cmd.as_ref().unwrap()).spawn()?;
        }
        info!("Bot built: {}", self.team_name);
        Ok(())
    }

    /// runs the bot in subprocess
    fn run(&self) -> std::io::Result<()> {
        std::process::Command::new(self.run_cmd.as_ref().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Other, "Run command failed to parse")
        })?)
        .spawn()?;
        info!("Bot ran: {}", self.team_name);
        Ok(())
    }

    /// connects bot to socket_path
    async fn connect(&self, socket_path: &Path) -> std::io::Result<()> {
        let mut stream = UnixStream::connect(socket_path)?;
        info!("CLIENT {} has connected!", self.team_name);

        stream.write_all(b"Hello, world!")?;
        stream.shutdown(std::net::Shutdown::Write)?;

        let mut response = String::new();
        stream.read_to_string(&mut response)?;
        info!("Response: {}", response);
        Ok(())
    }

    /// plays current bot against all other bots
    /// spawns a Dealer and Game per game and begins
    /// games in their own socket files
    pub async fn play(&self, bots: &Vec<Bot>) -> std::io::Result<()> {
        info!("PLAYING");
        debug!("{bots:?}");
        let socket_path = PathBuf::from(format!("/tmp/pokerzero/{}/socket", self.team_name));
        if !socket_path.exists() {
            fs::create_dir_all(&socket_path)?;
        }
        for b in bots {
            if b.team_name != self.team_name {
                let socket_file =
                    socket_path.join(format!("{}_vs_{}.sock", self.team_name, b.team_name));
                let bots_game = vec![b.clone(), self.clone()];
                let dealer = Dealer::new();
                let game = Game::new(bots_game, dealer);
                let socket_file_ = socket_file.clone();
                thread::spawn(move || game.start_server(&socket_file));
                thread::park_timeout(Duration::from_secs(1));
                self.connect(&socket_file_).await?;
                b.connect(&socket_file_).await?;
            }
        }
        Ok(())
    }
}

impl Game {
    pub fn new(bots: Vec<Bot>, dealer: Dealer) -> Self {
        let l = bots.len() as u32;
        Self {
            bots,
            num_players: l,
            dealer,
            button: 0,
            pot: 0,
            hole_cards: Vec::new(),
            community_cards: Vec::new(),
        }
    }

    fn add_bot(&mut self, bot: Bot) {
        self.bots.push(bot);
        self.num_players = self.num_players + 1;
    }

    fn run(&self) -> std::io::Result<()> {
        for bot in &self.bots {
            bot.build()?;
            bot.run()?;
        }
        Ok(())
    }

    fn start_server(&self, socket_path: &Path) -> std::io::Result<()> {
        let socket = Path::new(socket_path);
        if socket.exists() {
            std::fs::remove_file(socket)?;
        }

        let listener = UnixListener::bind(socket)?;
        info!("SERVER on {socket_path:?} started...");

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
