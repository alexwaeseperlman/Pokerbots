use std::io;
use tokio::process::{Child, Command};

use super::{BuildResult, Language};

pub struct Rust {}
impl Language for Rust {
    fn build(&self) -> io::Result<()> {
        Command::new("cargo")
            .arg("build")
            .arg("--release")
            .spawn()?;
        Ok(())
    }
    fn run(&self, configure: fn(command: &mut Command) -> &mut Command) -> io::Result<Child> {
        configure(Command::new("cargo").arg("run"))
            .arg("--release")
            .spawn()
    }
}
