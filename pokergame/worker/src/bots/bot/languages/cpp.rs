use std::{io, process::ExitStatus};
use tokio::process::{Child, Command};

use super::{BuildResult, Language};

pub struct CPP {}
#[async_trait::async_trait]
impl Language for CPP {
    async fn build(&self) -> io::Result<()> {
        let status = Command::new("g++")
            .arg("main.cpp")
            .arg("-O2")
            .arg("-o")
            .arg("main")
            .spawn()?
            .wait()
            .await?;
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to compile"));
        }
        Ok(())
    }
    fn run(&self, configure: fn(command: &mut Command) -> &mut Command) -> io::Result<Child> {
        configure(&mut Command::new("./main")).spawn()
    }
}
