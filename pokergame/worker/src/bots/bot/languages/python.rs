use std::io;
use tokio::process::{Child, Command};

use super::{BuildResult, Language};

pub struct Python {}
#[async_trait::async_trait]
impl Language for Python {
    async fn build(&self) -> io::Result<()> {
        Ok(())
    }
    fn run(&self, configure: fn(command: &mut Command) -> &mut Command) -> io::Result<Child> {
        configure(Command::new("python3").arg("main.py")).spawn()
    }
}
