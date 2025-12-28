use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::tools::{ExecutionResult, LLMBackend, ToolContext};

/// Maximum number of iterations before forcing termination
const MAX_ITERATIONS: usize = 20;

/// Maximum total execution time (5 minutes)
const MAX_EXECUTION_TIME: Duration = Duration::from_secs(300);

/// Type of step in the ReAct loop
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepType {
    /// AI is thinking about the problem
    Thought,
    /// AI is executing a tool/command
    Action,
    /// Result from action execution
    Observation,
    /// AI reflecting on progress and deciding next step
    Reflection,
    /// Final solution or conclusion
    Solution,
}

/// Single step in the ReAct loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    /// Step number (1-indexed)
    pub step_number: usize,

    /// Type of this step
    pub step_type: StepType,

    /// Content of this step (thought, command, observation, etc.)
    pub content: String,

    /// Tool used (if Action step)
    pub tool_used: Option<String>,

    /// Success status (for Action/Observation)
    pub success: Option<bool>,

    /// Timestamp
    pub timestamp: std::time::SystemTime,

    /// Educational explanation of the command (for explain mode)
    #[serde(default)]
    pub explanation: Option<String>,
}

/// Status of agent execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentStatus {
    /// Agent is actively working
    Running,

    /// Agent completed successfully
    Completed,

    /// Agent paused, waiting for user confirmation
    AwaitingConfirmation,

    /// Agent failed with error
    Failed(String),

    /// Agent stopped (max iterations or timeout)
    Stopped(String),
}

/// State of the agent during execution
#[derive(Debug, Clone)]
pub struct AgentState {
    /// Original problem/task description
    pub task: String,

    /// Current status
    pub status: AgentStatus,

    /// All steps taken so far
    pub history: Vec<AgentStep>,

    /// Information collected during diagnosis
    pub collected_info: Vec<(String, String)>,

    /// Identified root cause (if found)
    pub root_cause: Option<String>,

    /// Proposed solution plan
    pub solution_plan: Option<Vec<String>>,

    /// Iteration count
    pub iteration: usize,

    /// Start time
    pub start_time: Instant,
}

impl AgentState {
    pub fn new(task: String) -> Self {
        Self {
            task,
            status: AgentStatus::Running,
            history: Vec::new(),
            collected_info: Vec::new(),
            root_cause: None,
            solution_plan: None,
            iteration: 0,
            start_time: Instant::now(),
        }
    }

    /// Add a step to history
    pub fn add_step(
        &mut self,
        step_type: StepType,
        content: String,
        tool_used: Option<String>,
        success: Option<bool>,
    ) {
        let step = AgentStep {
            step_number: self.history.len() + 1,
            step_type,
            content,
            tool_used,
            success,
            timestamp: std::time::SystemTime::now(),
            explanation: None,
        };
        self.history.push(step);
    }

    /// Set explanation on the last step (for explain mode)
    pub fn set_last_step_explanation(&mut self, explanation: String) {
        if let Some(last_step) = self.history.last_mut() {
            last_step.explanation = Some(explanation);
        }
    }

    /// Check if should continue execution
    pub fn should_continue(&self) -> bool {
        match self.status {
            AgentStatus::Running => {
                // Check iteration limit
                if self.iteration >= MAX_ITERATIONS {
                    return false;
                }

                // Check time limit
                if self.start_time.elapsed() >= MAX_EXECUTION_TIME {
                    return false;
                }

                true
            }
            _ => false,
        }
    }

    /// Get last N steps of specific type
    pub fn get_recent_steps(&self, step_type: StepType, count: usize) -> Vec<&AgentStep> {
        self.history
            .iter()
            .rev()
            .filter(|s| s.step_type == step_type)
            .take(count)
            .collect()
    }

    /// Get execution summary
    pub fn summary(&self) -> String {
        let duration = self.start_time.elapsed();
        let steps_count = self.history.len();
        let actions_count = self
            .history
            .iter()
            .filter(|s| s.step_type == StepType::Action)
            .count();

        format!(
            "Task: {}\nStatus: {:?}\nSteps: {} (Actions: {})\nDuration: {:?}",
            self.task, self.status, steps_count, actions_count, duration
        )
    }
}

/// Main ReAct agent loop
pub struct AgentLoop {
    /// Current state
    state: AgentState,

    /// Tool context for execution (reserved for future use)
    #[allow(dead_code)]
    context: ToolContext,

    /// Tool registry for executing commands
    tool_registry: crate::tools::ToolRegistry,

    /// Callback for displaying progress (optional)
    #[allow(clippy::type_complexity)]
    progress_callback: Option<Box<dyn Fn(&AgentStep) + Send>>,

    /// Enable explain mode for educational command breakdowns
    explain_mode: bool,
}

impl AgentLoop {
    /// Create new agent loop for a task
    pub fn new(task: String, context: ToolContext) -> Self {
        Self {
            state: AgentState::new(task),
            context,
            tool_registry: crate::tools::ToolRegistry::new(),
            progress_callback: None,
            explain_mode: true, // Default ON for learning
        }
    }

    /// Enable or disable explain mode
    pub fn with_explain_mode(mut self, enabled: bool) -> Self {
        self.explain_mode = enabled;
        self
    }

    /// Set progress callback for live updates
    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&AgentStep) + Send + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Get current state
    pub fn state(&self) -> &AgentState {
        &self.state
    }

    /// Get mutable state
    pub fn state_mut(&mut self) -> &mut AgentState {
        &mut self.state
    }

    /// Execute one iteration of the ReAct loop
    /// Returns true if should continue, false if done
    pub async fn step(&mut self, llm: &dyn LLMBackend) -> Result<bool> {
        // Check if should continue
        if !self.state.should_continue() {
            if self.state.iteration >= MAX_ITERATIONS {
                self.state.status = AgentStatus::Stopped("Maximum iterations reached".to_string());
            } else if self.state.start_time.elapsed() >= MAX_EXECUTION_TIME {
                self.state.status =
                    AgentStatus::Stopped("Maximum execution time exceeded".to_string());
            }
            return Ok(false);
        }

        self.state.iteration += 1;

        // ReAct cycle:
        // 1. Thought - AI decides what to do next
        let thought = self.generate_thought(llm).await?;
        self.add_and_notify_step(StepType::Thought, thought.clone(), None, None);

        // 2. Check if AI thinks task is complete
        if self.is_completion_thought(&thought) {
            self.state.status = AgentStatus::Completed;
            return Ok(false);
        }

        // 3. Action - Extract and validate action
        let action = self.parse_action_from_thought(&thought)?;
        self.add_and_notify_step(
            StepType::Action,
            action.command.clone(),
            Some(action.tool_name.clone()),
            None,
        );

        // 3.5. Generate educational explanation if explain mode is enabled
        if self.explain_mode {
            if let Ok(explanation) =
                crate::ai::CommandExplainer::explain(&action.command, &action.tool_name, llm).await
            {
                self.state.set_last_step_explanation(explanation);
                // Re-notify with updated explanation
                if let Some(ref callback) = self.progress_callback {
                    if let Some(last_step) = self.state.history.last() {
                        callback(last_step);
                    }
                }
            }
        }

        // 4. Execute action (auto-execute if diagnostic, else may need confirmation)
        let execution_result = self.execute_action(&action).await?;

        // 5. Observation - Record result
        let observation = self.format_observation(&execution_result);
        let success = execution_result.exit_code == 0;
        self.add_and_notify_step(
            StepType::Observation,
            observation.clone(),
            None,
            Some(success),
        );

        // Store collected info
        self.state
            .collected_info
            .push((action.command.clone(), observation));

        // 6. Reflection - AI analyzes if making progress
        let reflection = self.generate_reflection(llm).await?;
        self.add_and_notify_step(StepType::Reflection, reflection.clone(), None, None);

        // Continue loop
        Ok(true)
    }

    /// Run the complete agent loop until completion or termination
    pub async fn run_until_complete(&mut self, llm: &dyn LLMBackend) -> Result<AgentState> {
        while self.step(llm).await? {
            // Continue until step returns false
        }

        Ok(self.state.clone())
    }

    /// Generate thought using LLM
    async fn generate_thought(&self, llm: &dyn LLMBackend) -> Result<String> {
        let prompt = self.build_thought_prompt();
        let response = llm.infer(&prompt).await?;
        Ok(response.reasoning)
    }

    /// Generate reflection using LLM
    async fn generate_reflection(&self, llm: &dyn LLMBackend) -> Result<String> {
        let prompt = self.build_reflection_prompt();
        let response = llm.infer(&prompt).await?;
        Ok(response.reasoning)
    }

    /// Build prompt for thought generation
    fn build_thought_prompt(&self) -> String {
        let available_tools = self.tool_registry.list_tools();

        let mut prompt = format!(
            "You are an autonomous ops troubleshooting agent.\n\
            Task: {}\n\n\
            Available tools: {}\n\n",
            self.state.task,
            available_tools.join(", ")
        );

        // Add history context
        if !self.state.history.is_empty() {
            prompt.push_str("What you've done so far:\n");
            for step in self.state.history.iter().rev().take(6).rev() {
                let content_preview = step.content.chars().take(150).collect::<String>();
                prompt.push_str(&format!(
                    "Step {}: {:?} - {}\n",
                    step.step_number, step.step_type, content_preview
                ));
            }
            prompt.push('\n');
        }

        prompt.push_str(
            "Think about what to do next:\n\
            \n\
            To gather information, respond with:\n\
            ACTION: [tool_name] [command]\n\
            Example: ACTION: nginx nginx -t\n\
            Example: ACTION: network netstat -tuln\n\
            Example: ACTION: apache2 apache2ctl -S\n\
            \n\
            When you've identified the root cause, respond with:\n\
            SOLUTION: [explanation and fix]\n\
            \n\
            Your thought:",
        );

        prompt
    }

    /// Build prompt for reflection
    fn build_reflection_prompt(&self) -> String {
        let last_observation = self
            .state
            .get_recent_steps(StepType::Observation, 1)
            .first()
            .map(|s| s.content.as_str())
            .unwrap_or("No observation");

        format!(
            "Task: {}\n\
            Latest observation: {}\n\n\
            Reflect on progress:\n\
            - Are you making progress toward the goal?\n\
            - Have you identified the root cause?\n\
            - What information is still missing?\n\
            - Should you continue investigating or propose a solution?\n\n\
            Your reflection:",
            self.state.task, last_observation
        )
    }

    /// Check if thought indicates completion
    fn is_completion_thought(&self, thought: &str) -> bool {
        thought.to_lowercase().contains("solution:")
            || thought.to_lowercase().contains("task complete")
            || thought.to_lowercase().contains("problem solved")
    }

    /// Parse action from thought
    fn parse_action_from_thought(&self, thought: &str) -> Result<ActionCommand> {
        // Look for ACTION: [tool] [command] pattern
        if let Some(action_line) = thought
            .lines()
            .find(|l| l.trim().to_lowercase().starts_with("action:"))
        {
            let action_content = action_line.trim()[7..].trim(); // Remove "ACTION:"

            // Parse tool and command
            let parts: Vec<&str> = action_content.splitn(2, ' ').collect();
            if parts.len() == 2 {
                return Ok(ActionCommand {
                    tool_name: parts[0].to_string(),
                    command: parts[1].to_string(),
                });
            }
        }

        // Fallback: treat whole thought as command
        Ok(ActionCommand {
            tool_name: "shell".to_string(),
            command: thought.to_string(),
        })
    }

    /// Execute action using proper tool
    async fn execute_action(&self, action: &ActionCommand) -> Result<ExecutionResult> {
        let start = std::time::Instant::now();

        // Get tool from registry
        if let Some(tool) = self.tool_registry.get_tool(&action.tool_name) {
            log::info!(
                "Using tool '{}' to execute: {}",
                action.tool_name,
                action.command
            );
            let result = tool.execute(&action.command).await?;
            Ok(result)
        } else {
            // Fallback to shell execution for unknown tools
            log::warn!(
                "Tool '{}' not found, falling back to shell",
                action.tool_name
            );
            use tokio::process::Command;

            let output = Command::new("sh")
                .arg("-c")
                .arg(&action.command)
                .output()
                .await?;

            Ok(ExecutionResult {
                exit_code: output.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                duration: start.elapsed(),
            })
        }
    }

    /// Format execution result as observation
    fn format_observation(&self, result: &ExecutionResult) -> String {
        if result.exit_code == 0 {
            if !result.stdout.is_empty() {
                result.stdout.clone()
            } else {
                "Command executed successfully (no output)".to_string()
            }
        } else {
            format!(
                "Command failed (exit code {}): {}",
                result.exit_code,
                if !result.stderr.is_empty() {
                    &result.stderr
                } else {
                    &result.stdout
                }
            )
        }
    }

    /// Add step and notify callback
    fn add_and_notify_step(
        &mut self,
        step_type: StepType,
        content: String,
        tool_used: Option<String>,
        success: Option<bool>,
    ) {
        self.state.add_step(step_type, content, tool_used, success);

        if let Some(ref callback) = self.progress_callback {
            if let Some(last_step) = self.state.history.last() {
                callback(last_step);
            }
        }
    }
}

/// Parsed action command
#[derive(Debug, Clone)]
struct ActionCommand {
    tool_name: String,
    command: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_state_creation() {
        let state = AgentState::new("Test task".to_string());
        assert_eq!(state.task, "Test task");
        assert_eq!(state.status, AgentStatus::Running);
        assert_eq!(state.iteration, 0);
    }

    #[test]
    fn test_agent_state_add_step() {
        let mut state = AgentState::new("Test".to_string());
        state.add_step(StepType::Thought, "Thinking...".to_string(), None, None);

        assert_eq!(state.history.len(), 1);
        assert_eq!(state.history[0].step_type, StepType::Thought);
        assert_eq!(state.history[0].content, "Thinking...");
    }

    #[test]
    fn test_should_continue() {
        let mut state = AgentState::new("Test".to_string());
        assert!(state.should_continue());

        state.iteration = MAX_ITERATIONS;
        assert!(!state.should_continue());

        state.iteration = 0;
        state.status = AgentStatus::Completed;
        assert!(!state.should_continue());
    }
}
