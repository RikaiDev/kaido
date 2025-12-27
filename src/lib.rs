// Kaido AI Shell - Universal Ops AI Assistant
// Library exports for testing

pub mod agent;
pub mod ai;
pub mod audit;
pub mod commands;
pub mod config;
pub mod error;
pub mod kubectl;
pub mod learning;
pub mod mcp;
pub mod mentor;
pub mod shell;
pub mod tools;
pub mod ui;
pub mod utils;

// Re-export commonly used items
pub use agent::{AgentLoop, AgentState, AgentStep, StepType};
pub use tools::{Tool, ToolRegistry, RiskLevel};
pub use error::PatternMatcher;
pub use commands::{CommandEngine, CommandResult};
pub use ai::{AIManager, GeminiBackend, OllamaBackend};
pub use shell::{KaidoShell, ShellConfig, PromptBuilder, PtyExecutor, PtyExecutionResult};
pub use mentor::{ErrorDetector, ErrorInfo, ErrorType, MentorDisplay, Verbosity};
pub use learning::{LearningProgress, LearningTracker};
pub use mcp::{McpServer, KaidoTools};

