use crate::config::Config;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Output};

/// Command executor for the shell
pub struct CommandExecutor {
    config: Config,
    working_directory: PathBuf,
    environment: HashMap<String, String>,
}

impl CommandExecutor {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            environment: std::env::vars().collect(),
        }
    }

    pub fn execute(&self, command: &str, args: &[&str]) -> std::io::Result<Output> {
        let mut cmd = Command::new(command);
        cmd.args(args);
        cmd.output().map_err(|e| e.into())
    }
}

impl Clone for CommandExecutor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            working_directory: self.working_directory.clone(),
            environment: self.environment.clone(),
        }
    }
}
