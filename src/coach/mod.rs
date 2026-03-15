pub mod event_router;
pub mod context;
pub mod ui;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachResponse {
    pub diagnosis: Option<String>,
    pub explanation: Option<String>,
    pub best_practice: Option<String>,
    pub commands: Vec<CoachCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachCommand {
    pub cmd: String,
    pub purpose: String,
}

impl CoachResponse {
    pub fn new() -> Self {
        Self {
            diagnosis: None,
            explanation: None,
            best_practice: None,
            commands: Vec::new(),
        }
    }

    pub fn with_diagnosis(mut self, diagnosis: &str) -> Self {
        self.diagnosis = Some(diagnosis.to_string());
        self
    }

    pub fn with_explanation(mut self, explanation: &str) -> Self {
        self.explanation = Some(explanation.to_string());
        self
    }

    pub fn with_best_practice(mut self, best: &str) -> Self {
        self.best_practice = Some(best.to_string());
        self
    }

    pub fn add_command(mut self, cmd: &str, purpose: &str) -> Self {
        self.commands.push(CoachCommand {
            cmd: cmd.to_string(),
            purpose: purpose.to_string(),
        });
        self
    }

    pub fn is_empty(&self) -> bool {
        self.diagnosis.is_none()
            && self.explanation.is_none()
            && self.best_practice.is_none()
            && self.commands.is_empty()
    }
}

impl Default for CoachResponse {
    fn default() -> Self {
        Self::new()
    }
}
