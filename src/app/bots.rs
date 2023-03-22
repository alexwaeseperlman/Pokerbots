use log::{error, info};
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

pub struct Engine {
    bots: Vec<Box<Bot>>,
}

impl Bot {
    fn build(&self) -> std::io::Result<()> {
        if self.build_cmd.is_some() {
            std::process::Command::new(self.build_cmd.clone().unwrap()).spawn()?;
        }
        info!("Bot built: {}", self.team_name);
        Ok(())
    }

    fn run(&self) -> std::io::Result<()> {
        if self.build_cmd.is_some() {
            std::process::Command::new(self.build_cmd.clone().unwrap()).spawn()?;
        }
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

impl Engine {
    fn add_bot(&mut self, bot: Box<Bot>) {
        self.bots.push(bot);
    }

    fn run(&self) {
        for bot in &self.bots {
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
