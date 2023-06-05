use std::{
    io,
    process::{self, Command, Stdio},
};

use super::{BuildResult, Language};

pub struct Rust {}
impl Language for Rust {
    fn build(&self) -> BuildResult {
        match Command::new("cargo").arg("build").arg("--release").spawn() {
            Ok(_) => BuildResult::Success,
            Err(_) => BuildResult::Failure,
        }
    }
    fn run(
        &self,
        configure: fn(command: &mut Command) -> &mut Command,
    ) -> io::Result<process::Child> {
        configure(Command::new("cargo").arg("run"))
            .arg("--release")
            .spawn()
    }
}
