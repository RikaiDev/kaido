use serde::{Deserialize, Serialize};

/// Single step in ReAct loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReActStep {
    pub thought: String,
    pub command: String,
    pub output: String,
}


/// Next action from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextAction {
    pub thought: String,
    pub command: String,
    pub done: bool,
}

/// Validation result from adversarial AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub reason: String,
}

impl ReActStep {
    pub fn new(thought: String, command: String, output: String) -> Self {
        Self {
            thought,
            command,
            output,
        }
    }
}

impl NextAction {
    pub fn done() -> Self {
        Self {
            thought: "Task completed".to_string(),
            command: String::new(),
            done: true,
        }
    }
}

