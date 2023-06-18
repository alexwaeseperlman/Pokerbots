use std::process::ExitStatus;

use tokio::io;
use tokio::process::{Child, Command};

pub mod cpp;
pub mod python;
pub mod rust;

#[derive(PartialEq, Debug)]
pub enum BuildResult {
    Success,
    Failure,
}

#[async_trait::async_trait]
pub trait Language {
    async fn build(&self) -> io::Result<ExitStatus>;
    fn run(&self, configure: fn(command: &mut Command) -> &mut Command) -> io::Result<Child>;
}

pub fn detect_language(name: &str) -> Option<Box<dyn Language>> {
    match name {
        "python" => Some(Box::new(python::Python {})),
        "rust" => Some(Box::new(rust::Rust {})),
        "c++" | "cpp" => Some(Box::new(cpp::CPP {})),
        _ => None,
    }
}
