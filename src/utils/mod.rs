use std::fmt;

pub type KaidoResult<T> = Result<T, KaidoError>;

// These structures are preserved for future implementation but not used in MVP

/*
/// AI context (not used in MVP)
#[derive(Debug, Clone)]
pub struct AIContext {
    pub session_id: String,
    pub conversation_history: Vec<String>,
    pub current_directory: std::path::PathBuf,
    pub recent_commands: Vec<String>,
    pub user_preferences: UserPreferences,
}

impl AIContext {
    pub fn new() -> Self {
        Self {
            session_id: "default".to_string(),
            conversation_history: vec![],
            current_directory: std::env::current_dir().unwrap_or_default(),
            recent_commands: vec![],
            user_preferences: UserPreferences::default(),
        }
    }
}

/// User preferences (not used in MVP)
#[derive(Debug, Clone)]
pub struct UserPreferences {
    pub preferred_language: String,
    pub verbosity_level: String,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            preferred_language: "en".to_string(),
            verbosity_level: "normal".to_string(),
        }
    }
}

/// Performance metrics (not used in MVP)
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_inferences: u64,
    pub average_response_time_ms: f64,
    pub memory_usage_mb: f64,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            total_inferences: 0,
            average_response_time_ms: 0.0,
            memory_usage_mb: 0.0,
        }
    }
}
*/

/// Kaido error types
#[derive(Debug)]
pub enum KaidoError {
    ApplicationError {
        message: String,
        context: Option<String>,
    },
    ModelError {
        message: String,
        model_name: String,
    },
}

impl fmt::Display for KaidoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KaidoError::ApplicationError { message, context } => {
                write!(f, "Application error: {message}")?;
                if let Some(ctx) = context {
                    write!(f, " ({ctx})")?;
                }
                Ok(())
            }
            KaidoError::ModelError {
                message,
                model_name,
            } => {
                write!(f, "Model '{model_name}' error: {message}")
            }
        }
    }
}

impl std::error::Error for KaidoError {}
