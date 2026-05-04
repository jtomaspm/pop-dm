use std::process::{Command, ExitStatus};

use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCommand {
    pub program: String,
    pub args: Vec<String>,
}

impl SessionCommand {
    pub fn new(program: String, args: Vec<String>) -> Self {
        Self { program, args }
    }
}

pub struct SessionExit {
    pub code: Option<i32>,
}

pub fn run_session(command: &SessionCommand) -> Result<ExitStatus> {
    Ok(
        Command::new(&command.program)
            .args(&command.args)
            .spawn()?
            .wait()?   
    )
}