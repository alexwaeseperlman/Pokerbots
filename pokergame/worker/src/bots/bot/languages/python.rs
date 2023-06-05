use std::{
    io,
    process::{self, Command},
};

use super::{BuildResult, Language, RunResult};

pub struct Python {}
impl Language for Python {
    fn build(&self) -> BuildResult {
        BuildResult::Success
    }
    fn run(
        &self,
        configure: fn(command: &mut Command) -> &mut Command,
    ) -> io::Result<process::Child> {
        configure(process::Command::new("python3").arg("main.py")).spawn()
    }
}
