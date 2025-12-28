use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

pub mod apache2;
pub mod docker;
pub mod drush;
pub mod kubectl_tool;
pub mod network;
pub mod nginx;
pub mod registry;
pub mod sql;

// Re-export for convenience
pub use apache2::Apache2Tool;
pub use docker::DockerTool;
pub use drush::DrushTool;
pub use kubectl_tool::KubectlTool;
pub use network::NetworkTool;
pub use nginx::NginxTool;
pub use registry::ToolRegistry;
pub use sql::{SQLDialect, SQLTool};

/// Risk level for command operations (4-tier system)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Read-only operations (get, describe, logs, SELECT)
    Low,
    /// State-modifying operations (apply, scale, INSERT, UPDATE)
    Medium,
    /// Destructive operations (delete, DROP TABLE, TRUNCATE)
    High,
    /// Batch destructive operations (DROP DATABASE, DELETE FROM without WHERE)
    Critical,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Low => "LOW",
            RiskLevel::Medium => "MEDIUM",
            RiskLevel::High => "HIGH",
            RiskLevel::Critical => "CRITICAL",
        }
    }

    pub fn requires_confirmation(&self) -> bool {
        match self {
            RiskLevel::Low => false,
            RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical => true,
        }
    }

    pub fn requires_typed_confirmation(&self, is_production: bool) -> bool {
        match self {
            RiskLevel::High | RiskLevel::Critical if is_production => true,
            RiskLevel::Critical => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Translation result from natural language to command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Translation {
    /// Generated command
    pub command: String,

    /// AI confidence score (0-100)
    pub confidence: u8,

    /// AI reasoning/explanation
    pub reasoning: String,

    /// Tool name that generated this command
    pub tool_name: String,

    /// Files that need to exist for this command to work
    pub requires_files: Vec<PathBuf>,
}

/// Execution result from running a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Exit code from command
    pub exit_code: i32,

    /// Standard output
    pub stdout: String,

    /// Standard error
    pub stderr: String,

    /// Execution duration
    pub duration: Duration,
}

/// Tool context containing environment information
#[derive(Debug, Clone)]
pub struct ToolContext {
    pub working_directory: PathBuf,
    pub environment_vars: HashMap<String, String>,
    pub user: String,

    // Tool-specific contexts
    pub kubectl_context: Option<crate::kubectl::KubectlContext>,
    pub docker_host: Option<String>,
    pub db_connection: Option<DatabaseConnection>,
}

impl Default for ToolContext {
    fn default() -> Self {
        Self {
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            environment_vars: std::env::vars().collect(),
            user: users::get_current_username()
                .and_then(|u| u.into_string().ok())
                .unwrap_or_else(|| "unknown".to_string()),
            kubectl_context: None,
            docker_host: std::env::var("DOCKER_HOST").ok(),
            db_connection: None,
        }
    }
}

/// Database connection information
#[derive(Debug, Clone)]
pub struct DatabaseConnection {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub is_production: bool,
}

impl DatabaseConnection {
    /// Get connection string for display
    pub fn connection_string(&self) -> String {
        format!(
            "{}@{}:{}/{}",
            self.username, self.host, self.port, self.database
        )
    }

    /// Check if this is a production database
    pub fn is_prod(&self) -> bool {
        self.is_production
    }
}

/// Error explanation for intelligent error diagnosis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorExplanation {
    /// Error type (brief classification)
    pub error_type: String,

    /// Human-readable reason
    pub reason: String,

    /// Possible causes (2-3 items)
    pub possible_causes: Vec<String>,

    /// Solutions with commands
    pub solutions: Vec<Solution>,

    /// Recommended solution index (0-based)
    pub recommended_solution: usize,

    /// Documentation links (optional)
    pub documentation_links: Vec<String>,
}

/// Solution for fixing an error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solution {
    pub description: String,
    pub command: Option<String>,
    pub risk_level: RiskLevel,
}

/// Tool call made by AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique ID for this tool call
    pub id: String,

    /// Tool name (kubectl, docker, nginx, etc.)
    pub tool_name: String,

    /// Command to execute
    pub command: String,

    /// Purpose/reason for this call
    pub purpose: String,

    /// Risk level of this command
    pub risk_level: RiskLevel,

    /// Whether this is safe to auto-execute (diagnostic commands)
    pub auto_executable: bool,

    /// Execution result (filled after execution)
    pub result: Option<ExecutionResult>,

    /// Timestamp
    pub timestamp: std::time::SystemTime,
}

impl ToolCall {
    /// Create a new tool call
    pub fn new(tool_name: String, command: String, purpose: String, risk_level: RiskLevel) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            tool_name,
            command,
            purpose,
            risk_level,
            auto_executable: matches!(risk_level, RiskLevel::Low),
            result: None,
            timestamp: std::time::SystemTime::now(),
        }
    }

    /// Create a diagnostic tool call (auto-executable)
    pub fn diagnostic(tool_name: String, command: String, purpose: String) -> Self {
        let mut call = Self::new(tool_name, command, purpose, RiskLevel::Low);
        call.auto_executable = true;
        call
    }

    /// Set execution result
    pub fn set_result(&mut self, result: ExecutionResult) {
        self.result = Some(result);
    }

    /// Check if execution was successful
    pub fn is_successful(&self) -> bool {
        self.result
            .as_ref()
            .map(|r| r.exit_code == 0)
            .unwrap_or(false)
    }

    /// Get output (stdout or stderr)
    pub fn get_output(&self) -> Option<String> {
        self.result.as_ref().map(|r| {
            if !r.stdout.is_empty() {
                r.stdout.clone()
            } else {
                r.stderr.clone()
            }
        })
    }
}

/// Tool executor for AI agent
/// Manages tool calls and execution
pub struct ToolExecutor {
    registry: ToolRegistry,
}

impl ToolExecutor {
    pub fn new() -> Self {
        Self {
            registry: ToolRegistry::new(),
        }
    }

    /// Execute a tool call
    pub async fn execute(&self, tool_call: &mut ToolCall) -> Result<()> {
        let tool = self
            .registry
            .get_tool(&tool_call.tool_name)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", tool_call.tool_name))?;

        let result = tool.execute(&tool_call.command).await?;
        tool_call.set_result(result);

        Ok(())
    }

    /// Execute multiple tool calls in sequence
    pub async fn execute_batch(&self, tool_calls: &mut [ToolCall]) -> Result<()> {
        for call in tool_calls.iter_mut() {
            self.execute(call).await?;
        }
        Ok(())
    }

    /// Filter tool calls that are safe to auto-execute
    pub fn filter_auto_executable(tool_calls: &[ToolCall]) -> Vec<&ToolCall> {
        tool_calls.iter().filter(|c| c.auto_executable).collect()
    }

    /// Filter tool calls that require confirmation
    pub fn filter_requires_confirmation(tool_calls: &[ToolCall]) -> Vec<&ToolCall> {
        tool_calls.iter().filter(|c| !c.auto_executable).collect()
    }
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// LLM Backend abstraction (local GGUF or OpenAI)
#[async_trait]
pub trait LLMBackend: Send + Sync {
    async fn infer(&self, prompt: &str) -> Result<LLMResponse>;
}

/// LLM response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub command: String,
    pub confidence: u8,
    pub reasoning: String,
}

/// Universal tool interface - all tools must implement this trait
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool name (kubectl, docker, mysql, drush)
    fn name(&self) -> &'static str;

    /// Detect if input belongs to this tool
    /// Returns confidence score 0.0-1.0
    ///
    /// Examples:
    /// - "get pods" → kubectl returns 0.9
    /// - "docker ps" → docker returns 1.0
    fn detect_intent(&self, input: &str) -> f32;

    /// Translate natural language to command
    async fn translate(
        &self,
        input: &str,
        context: &ToolContext,
        llm: &dyn LLMBackend,
    ) -> Result<Translation>;

    /// Classify risk level of a command
    fn classify_risk(&self, command: &str, context: &ToolContext) -> RiskLevel;

    /// Execute the command
    async fn execute(&self, command: &str) -> Result<ExecutionResult>;

    /// Explain error (optional implementation)
    fn explain_error(&self, _error: &str) -> Option<ErrorExplanation> {
        None // Default: no special error explanation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_requires_confirmation() {
        assert!(!RiskLevel::Low.requires_confirmation());
        assert!(RiskLevel::Medium.requires_confirmation());
        assert!(RiskLevel::High.requires_confirmation());
        assert!(RiskLevel::Critical.requires_confirmation());
    }

    #[test]
    fn test_risk_level_typed_confirmation() {
        assert!(!RiskLevel::Low.requires_typed_confirmation(false));
        assert!(!RiskLevel::Medium.requires_typed_confirmation(false));
        assert!(!RiskLevel::High.requires_typed_confirmation(false));
        assert!(RiskLevel::High.requires_typed_confirmation(true));
        assert!(RiskLevel::Critical.requires_typed_confirmation(false));
        assert!(RiskLevel::Critical.requires_typed_confirmation(true));
    }

    #[test]
    fn test_tool_context_default() {
        let ctx = ToolContext::default();
        assert!(!ctx.user.is_empty());
        assert!(ctx.working_directory.exists() || ctx.working_directory.as_os_str() == "/");
    }
}
