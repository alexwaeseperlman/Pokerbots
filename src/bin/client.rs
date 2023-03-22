use std::os::unix::net::UnixStream;
use std::io::{Read, Write};
use std::path::Path;
use server::SOCKET_PATH;

mod server;

fn main() {
    let socket = Path::new(SOCKET_PATH);

    let mut stream = UnixStream::connect(socket).expect("Failed to connect stream to socket");

    stream.write_all(b"Hello, world!").unwrap();

    stream.shutdown(std::net::Shutdown::Write).unwrap();

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    println!("Response: {}", response);
}
