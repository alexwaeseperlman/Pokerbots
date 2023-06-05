use std::process::exit;

use clap::Parser;
use pokergame::bots::bot::languages;

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Args {
    /// The path to the socket file
    #[arg(short, long)]
    socket_path: String,

    /// The path to the bots directory
    #[arg(short, long, default_value = "bots")]
    bot_zip: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Unzip the bot
    std::process::Command::new("unzip")
        .arg(args.bot_zip)
        .spawn()
        .expect("Failed to unzip bot");

    let bot = pokergame::bots::bot::Bot::new(std::path::PathBuf::from(".")).unwrap();
    // serialize errors
    let build_result = bot.build();
    if build_result == languages::BuildResult::Failure {
        exit(1);
    }
    let cproc = bot.run();
    if let Err(e) = cproc {
        exit(1);
    }
    let mut cproc = cproc.unwrap();
    let mut stdout = cproc.stdout.unwrap();
    let mut stdin = cproc.stdin.unwrap();

    let socket_connection = tokio::net::UnixStream::connect(args.socket_path)
        .await
        .unwrap();
}
