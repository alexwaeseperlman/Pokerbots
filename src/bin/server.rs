use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::io::{Read, Write};

pub static SOCKET_PATH: &str = "/tmp/sock";

fn main() {
    let socket = Path::new(SOCKET_PATH);
    if socket.exists() {
        std::fs::remove_file(socket).expect(&format!{"Failed to delete socket at {:?}", socket});
    }

    let listener = UnixListener::bind(socket).expect("Failed to bind socket to listener");

    loop {
        let (unix_stream, _) = listener
            .accept()
            .expect("Failed to accept connection");
        handle_stream(unix_stream);
    }
}

fn handle_stream(mut stream: UnixStream) {
    let mut message = String::new();
    stream.read_to_string(&mut message).expect("Failed to read stream");
    println!("Received message: {}", message);

    stream.write_all(b"Hellooooo").expect("Failed to write to stream");
}
