use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// OpenAI API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub timeout_seconds: u64,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(), // Must be set by user
            model: "gpt-4-turbo-preview".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            timeout_seconds: 10,
        }
    }
}

/// Audit log configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub database_path: PathBuf,
    pub retention_days: u32,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            database_path: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".kaido")
                .join("audit.db"),
            retention_days: 90,
        }
    }
}

/// Safety configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub confirm_destructive: bool,
    pub require_typed_confirmation_in_production: bool,
    pub log_commands: bool,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            confirm_destructive: true,
            require_typed_confirmation_in_production: true,
            log_commands: true,
        }
    }
}

/// Display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub show_confidence_threshold: u8,
    pub show_reasoning: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_confidence_threshold: 70,
            show_reasoning: false,
        }
    }
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ai: OpenAIConfig,
    pub audit: AuditConfig,
    pub safety: SafetyConfig,
    pub display: DisplayConfig,
    
    /// Gemini API key (optional, can also be set via GEMINI_API_KEY env var)
    pub gemini_api_key: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ai: OpenAIConfig::default(),
            audit: AuditConfig::default(),
            safety: SafetyConfig::default(),
            display: DisplayConfig::default(),
            gemini_api_key: None,
        }
    }
}

impl Config {

    /// Load configuration from TOML file
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::get_config_path()?;
        
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to TOML file
    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::get_config_path()?;
        
        // Create config directory if not exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, contents)?;
        
        // Set permissions to 600 (user read/write only) on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&config_path, permissions)?;
        }
        
        Ok(())
    }

    /// Get config file path
    pub fn get_config_path() -> anyhow::Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
        
        Ok(home.join(".kaido").join("config.toml"))
    }
}

/// Logging level enumeration (preserved for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Quiet,
    Normal,
    Verbose,
    Debug,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Quiet => "quiet",
            LogLevel::Normal => "normal",
            LogLevel::Verbose => "verbose",
            LogLevel::Debug => "debug",
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
