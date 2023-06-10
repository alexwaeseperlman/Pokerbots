use std::io;
use tokio::process::{Child, Command};

use super::{BuildResult, Language};

pub struct Python {}
impl Language for Python {
    fn build(&self) -> io::Result<()> {
        Ok(())
    }
    fn run(&self, configure: fn(command: &mut Command) -> &mut Command) -> io::Result<Child> {
        configure(Command::new("python3").arg("main.py")).spawn()
    }
}
