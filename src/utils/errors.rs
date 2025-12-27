use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main error type for Kaido AI Shell
#[derive(Debug, thiserror::Error)]
pub enum KaidoError {
    /// AI model related errors
    #[error("Model error: {message} (model: {model_name})")]
    ModelError { message: String, model_name: String },

    /// Command execution errors
    #[error("Execution error: {message} (command: {command}, exit_code: {exit_code:?})")]
    ExecutionError {
        command: String,
        exit_code: Option<i32>,
        message: String,
    },

    /// Safety rule violations
    #[error("Safety violation: {message} (command: {command}, rule: {rule_id})")]
    SafetyViolation {
        command: String,
        rule_id: String,
        message: String,
    },

    /// Configuration errors
    #[error("Configuration error: {message} (file: {file_path})")]
    ConfigError { file_path: PathBuf, message: String },

    /// General application errors
    #[error("Application error: {message}")]
    ApplicationError {
        message: String,
        context: Option<String>,
    },

    /// IO errors
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// TOML parsing errors
    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),

    /// Network errors
    #[error("Network error: {message}")]
    NetworkError { message: String },

    /// Authentication errors
    #[error("Authentication error: {message}")]
    AuthenticationError { message: String },

    /// Rate limit errors
    #[error("Rate limit error: {message}")]
    RateLimitError { message: String },

    /// Internationalization errors
    #[error("i18n error: {message}")]
    I18nError { message: String },

    /// Locale detection errors
    #[error("Locale error: {message}")]
    LocaleError { message: String },

    /// Translation errors
    #[error("Translation error: {message}")]
    TranslationError { message: String },

    /// Cultural context errors
    #[error("Cultural context error: {message}")]
    CulturalError { message: String },

    /// Readline errors
    #[error("Readline error: {0}")]
    ReadlineError(#[from] rustyline::error::ReadlineError),
}

/// Result type alias for Kaido operations
pub type KaidoResult<T> = Result<T, KaidoError>;

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Error context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub working_directory: Option<PathBuf>,
    pub command_history: Vec<String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            user_id: None,
            session_id: None,
            working_directory: None,
            command_history: Vec::new(),
        }
    }

    pub fn with_session(session_id: String) -> Self {
        Self {
            session_id: Some(session_id),
            ..Self::new()
        }
    }
}

/// Error reporting and handling utilities
pub struct ErrorHandler {
    log_errors: bool,
    context: ErrorContext,
}

impl ErrorHandler {
    pub fn new(log_errors: bool) -> Self {
        Self {
            log_errors,
            context: ErrorContext::new(),
        }
    }

    pub fn with_context(context: ErrorContext, log_errors: bool) -> Self {
        Self {
            log_errors,
            context,
        }
    }

    pub fn handle_error(&self, error: &KaidoError) -> String {
        let error_message = match error {
            KaidoError::ModelError {
                message,
                model_name,
            } => {
                format!("AI model '{model_name}' encountered an error: {message}")
            }
            KaidoError::ExecutionError {
                command,
                exit_code,
                message,
            } => {
                format!("Command '{command}' failed (exit code: {exit_code:?}): {message}")
            }
            KaidoError::SafetyViolation {
                command,
                rule_id,
                message,
            } => {
                format!("Safety rule '{rule_id}' blocked command '{command}': {message}")
            }
            KaidoError::ConfigError { file_path, message } => {
                format!(
                    "Configuration error in '{}': {}",
                    file_path.display(),
                    message
                )
            }
            KaidoError::ApplicationError { message, context } => {
                if let Some(ctx) = context {
                    format!("Application error: {message} (context: {ctx})")
                } else {
                    format!("Application error: {message}")
                }
            }
            _ => format!("Unexpected error: {error}"),
        };

        if self.log_errors {
            log::error!(
                "Error occurred: {} | Context: {:?}",
                error_message,
                self.context
            );
        }

        error_message
    }

    pub fn get_user_friendly_message(&self, error: &KaidoError) -> String {
        match error {
            KaidoError::ModelError { model_name, .. } => {
                format!(" The AI model '{model_name}' is having trouble. Please try again or check your model configuration.")
            }
            KaidoError::ExecutionError { command, .. } => {
                format!(" The command '{command}' failed to execute. Let me help you fix this.")
            }
            KaidoError::SafetyViolation { command, .. } => {
                format!("️  The command '{command}' was blocked for safety reasons. Please review and try a safer alternative.")
            }
            KaidoError::ConfigError { .. } => {
                "️  There's a problem with your configuration. Please check your settings."
                    .to_string()
            }
            KaidoError::ApplicationError { .. } => {
                " An unexpected error occurred. Please try again or restart Kaido.".to_string()
            }
            _ => " An unexpected error occurred. Please try again.".to_string(),
        }
    }
}

/// Convenience functions for common error scenarios
pub fn model_error(message: &str, model_name: &str) -> KaidoError {
    KaidoError::ModelError {
        message: message.to_string(),
        model_name: model_name.to_string(),
    }
}

pub fn execution_error(command: &str, exit_code: Option<i32>, message: &str) -> KaidoError {
    KaidoError::ExecutionError {
        command: command.to_string(),
        exit_code,
        message: message.to_string(),
    }
}

pub fn safety_violation(command: &str, rule_id: &str, message: &str) -> KaidoError {
    KaidoError::SafetyViolation {
        command: command.to_string(),
        rule_id: rule_id.to_string(),
        message: message.to_string(),
    }
}

pub fn config_error(file_path: PathBuf, message: &str) -> KaidoError {
    KaidoError::ConfigError {
        file_path,
        message: message.to_string(),
    }
}

pub fn application_error(message: &str, context: Option<&str>) -> KaidoError {
    KaidoError::ApplicationError {
        message: message.to_string(),
        context: context.map(|s| s.to_string()),
    }
}
