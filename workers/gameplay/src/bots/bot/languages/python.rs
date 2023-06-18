use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

use tokio::io;
use tokio::process::{Child, Command};

use super::{BuildResult, Language};

pub struct Python {}
#[async_trait::async_trait]
impl Language for Python {
    async fn build(&self) -> io::Result<ExitStatus> {
        Ok(ExitStatusExt::from_raw(0))
    }
    fn run(&self, configure: fn(command: &mut Command) -> &mut Command) -> io::Result<Child> {
        configure(Command::new("python3").arg("main.py")).spawn()
    }
}
