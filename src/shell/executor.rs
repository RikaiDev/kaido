use crate::config::Config;
use std::collections::HashMap;
use std::path::PathBuf;

/// Command executor for the shell
pub struct CommandExecutor {
    config: Config,
    working_directory: PathBuf,
    environment: HashMap<String, String>,
}

impl CommandExecutor {
    // CommandExecutor is preserved for potential future non-TUI use
    // TUI currently executes commands directly through SafeExecutor
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