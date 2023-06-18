use std::process::ExitStatus;
use tokio::io;
use tokio::process::{Child, Command};

use super::{BuildResult, Language};

pub struct CPP {}
#[async_trait::async_trait]
impl Language for CPP {
    async fn build(&self) -> io::Result<ExitStatus> {
        Command::new("g++")
            .arg("main.cpp")
            .arg("-O2")
            .arg("-o")
            .arg("main")
            .spawn()?
            .wait()
            .await
    }
    fn run(&self, configure: fn(command: &mut Command) -> &mut Command) -> io::Result<Child> {
        configure(&mut Command::new("./main")).spawn()
    }
}
