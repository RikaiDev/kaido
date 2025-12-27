// Shell history management for Kaido
//
// Handles command history persistence using rustyline's FileHistory.
// History is stored in ~/.kaido/history

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Get the default history file path
pub fn default_history_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".kaido")
        .join("history")
}

/// Ensure the history directory exists
pub fn ensure_history_dir() -> Result<PathBuf> {
    let history_path = default_history_path();

    if let Some(parent) = history_path.parent() {
        std::fs::create_dir_all(parent)
            .context("Failed to create ~/.kaido directory")?;
    }

    Ok(history_path)
}

/// Shell history configuration
#[derive(Debug, Clone)]
pub struct HistoryConfig {
    /// Maximum number of entries to keep
    pub max_entries: usize,
    /// Path to history file
    pub file_path: PathBuf,
    /// Whether to ignore duplicate consecutive entries
    pub ignore_dups: bool,
    /// Whether to ignore entries starting with space
    pub ignore_space: bool,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            file_path: default_history_path(),
            ignore_dups: true,
            ignore_space: true,
        }
    }
}

impl HistoryConfig {
    /// Create config with custom path
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self {
            file_path: path.into(),
            ..Default::default()
        }
    }

    /// Set maximum entries
    pub fn max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_history_path() {
        let path = default_history_path();
        assert!(path.ends_with("history"));
        assert!(path.to_string_lossy().contains(".kaido"));
    }

    #[test]
    fn test_history_config_default() {
        let config = HistoryConfig::default();
        assert_eq!(config.max_entries, 10000);
        assert!(config.ignore_dups);
        assert!(config.ignore_space);
    }

    #[test]
    fn test_history_config_with_path() {
        let config = HistoryConfig::with_path("/tmp/test_history");
        assert_eq!(config.file_path, PathBuf::from("/tmp/test_history"));
    }

    #[test]
    fn test_ensure_history_dir() {
        // This should not panic
        let result = ensure_history_dir();
        assert!(result.is_ok());
    }
}
