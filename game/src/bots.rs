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

#[derive(Debug, Clone)]
pub struct Bot {
    team_name: String,
    path: PathBuf,
    build_cmd: Option<String>,
    run_cmd: Option<String>,
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
}
/*
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
*/
