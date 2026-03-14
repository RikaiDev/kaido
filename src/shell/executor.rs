use std::process::{Command, Output};

pub struct CommandExecutor;

impl CommandExecutor {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, command: &str, args: &[&str]) -> std::io::Result<Output> {
        let mut cmd = Command::new(command);
        cmd.args(args);
        cmd.output().map_err(|e| e.into())
    }
}

impl Clone for CommandExecutor {
    fn clone(&self) -> Self {
        Self
    }
}
