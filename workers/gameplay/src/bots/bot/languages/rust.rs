use std::process::ExitStatus;

use tokio::io;
use tokio::process::{Child, Command};

use super::{BuildResult, Language};

pub struct Rust {}
#[async_trait::async_trait]
impl Language for Rust {
    async fn build(&self) -> io::Result<ExitStatus> {
        Command::new("cargo")
            .arg("build")
            .arg("--release")
            .spawn()?
            .wait()
            .await
    }
    fn run(&self, configure: fn(command: &mut Command) -> &mut Command) -> io::Result<Child> {
        configure(Command::new("cargo").arg("run"))
            .arg("--release")
            .spawn()
    }
}
