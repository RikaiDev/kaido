# API Contracts: Kaido AI Shell Core

**Date**: 2025-10-22  
**Feature**: Kaido AI Shell Core  
**Purpose**: Define internal API contracts and interfaces

## Core Interfaces

### Shell REPL Interface

```rust
pub trait ShellRepl {
    /// Start the REPL loop
    async fn run(&mut self) -> Result<()>;
    
    /// Process a single line of input
    async fn process_input(&mut self, input: &str) -> Result<ReplResponse>;
    
    /// Get current session state
    fn get_session_state(&self) -> &SessionState;
    
    /// Update session state
    fn update_session_state(&mut self, state: SessionState);
}
```

**Methods**:
- `run()`: Main REPL loop, handles user input continuously
- `process_input()`: Process single command or natural language input
- `get_session_state()`: Access current session state
- `update_session_state()`: Update session state after command execution

### Command Executor Interface

```rust
pub trait CommandExecutor {
    /// Execute a command and return results
    async fn execute(&self, command: &str, context: &ExecutionContext) -> Result<CommandResult>;
    
    /// Execute a plan step by step
    async fn execute_plan(&self, plan: &TaskPlan) -> Result<PlanResult>;
    
    /// Check if command is safe to execute
    fn is_safe(&self, command: &str) -> SafetyCheck;
    
    /// Get execution context for current session
    fn get_execution_context(&self) -> ExecutionContext;
}
```

**Methods**:
- `execute()`: Execute single command with context
- `execute_plan()`: Execute multi-step plan
- `is_safe()`: Check command safety before execution
- `get_execution_context()`: Get current execution environment

### AI Model Interface

```rust
pub trait AIModel {
    /// Load the model from file or API
    async fn load(&mut self) -> Result<()>;
    
    /// Generate a task plan from natural language
    async fn plan_task(&self, input: &str, context: &AIContext) -> Result<TaskPlan>;
    
    /// Explain a command error
    async fn explain_error(&self, command: &str, error: &str) -> Result<String>;
    
    /// Generate command from natural language
    async fn generate_command(&self, input: &str, context: &AIContext) -> Result<String>;
    
    /// Check if model is ready for inference
    fn is_ready(&self) -> bool;
}
```

**Methods**:
- `load()`: Initialize model for inference
- `plan_task()`: Convert natural language to task plan
- `explain_error()`: Generate error explanations
- `generate_command()`: Convert natural language to single command
- `is_ready()`: Check model availability

### Safety Detector Interface

```rust
pub trait SafetyDetector {
    /// Check if command requires confirmation
    fn requires_confirmation(&self, command: &str) -> Option<SafetyWarning>;
    
    /// Add a new safety rule
    fn add_rule(&mut self, rule: SafetyRule) -> Result<()>;
    
    /// Remove a safety rule
    fn remove_rule(&mut self, rule_id: &str) -> Result<()>;
    
    /// Get all active safety rules
    fn get_active_rules(&self) -> Vec<&SafetyRule>;
    
    /// Check command against all rules
    fn check_command(&self, command: &str) -> Vec<SafetyWarning>;
}
```

**Methods**:
- `requires_confirmation()`: Check if command needs user confirmation
- `add_rule()`: Add new safety rule
- `remove_rule()`: Remove existing rule
- `get_active_rules()`: List all active safety rules
- `check_command()`: Comprehensive safety check

## Data Transfer Objects

### ReplResponse

```rust
pub struct ReplResponse {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub command_executed: Option<String>,
    pub plan_generated: Option<TaskPlan>,
    pub requires_confirmation: bool,
}
```

**Fields**:
- `success`: Whether operation succeeded
- `output`: Response text to display
- `error`: Error message if failed
- `command_executed`: Command that was executed
- `plan_generated`: Task plan if AI generated one
- `requires_confirmation`: Whether user confirmation needed

### CommandResult

```rust
pub struct CommandResult {
    pub execution_id: String,
    pub command: String,
    pub status: ExecutionStatus,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub execution_time: Duration,
    pub timestamp: DateTime,
}
```

**Fields**:
- `execution_id`: Unique execution identifier
- `command`: Command that was executed
- `status`: Execution result status
- `exit_code`: Process exit code
- `stdout`: Standard output
- `stderr`: Standard error
- `execution_time`: How long execution took
- `timestamp`: When command was executed

### PlanResult

```rust
pub struct PlanResult {
    pub plan_id: String,
    pub status: PlanStatus,
    pub steps_completed: usize,
    pub total_steps: usize,
    pub failed_steps: Vec<String>,
    pub total_execution_time: Duration,
    pub results: Vec<CommandResult>,
}
```

**Fields**:
- `plan_id`: Unique plan identifier
- `status`: Overall plan execution status
- `steps_completed`: Number of steps completed
- `total_steps`: Total number of steps
- `failed_steps`: List of failed step IDs
- `total_execution_time`: Total time for all steps
- `results`: Results from each step execution

### ExecutionContext

```rust
pub struct ExecutionContext {
    pub working_directory: PathBuf,
    pub environment: HashMap<String, String>,
    pub session_id: String,
    pub user_id: Option<String>,
    pub timestamp: DateTime,
}
```

**Fields**:
- `working_directory`: Current working directory
- `environment`: Environment variables
- `session_id`: Current session identifier
- `user_id`: Optional user identifier
- `timestamp`: Context creation time

### SafetyWarning

```rust
pub struct SafetyWarning {
    pub rule_id: String,
    pub severity: SeverityLevel,
    pub message: String,
    pub command: String,
    pub requires_confirmation: bool,
    pub suggested_alternative: Option<String>,
}
```

**Fields**:
- `rule_id`: ID of safety rule that triggered
- `severity`: How dangerous the operation is
- `message`: Warning message for user
- `command`: Command that triggered warning
- `requires_confirmation`: Whether user must confirm
- `suggested_alternative`: Safer alternative command

### AIContext

```rust
pub struct AIContext {
    pub session_id: String,
    pub conversation_history: Vec<ConversationTurn>,
    pub current_directory: PathBuf,
    pub recent_commands: Vec<String>,
    pub user_preferences: UserPreferences,
}
```

**Fields**:
- `session_id`: Current session identifier
- `conversation_history`: Previous AI interactions
- `current_directory`: Current working directory
- `recent_commands`: Recently executed commands
- `user_preferences`: User-specific AI preferences

### ConversationTurn

```rust
pub struct ConversationTurn {
    pub user_input: String,
    pub ai_response: String,
    pub timestamp: DateTime,
    pub command_executed: Option<String>,
}
```

**Fields**:
- `user_input`: What user said
- `ai_response`: AI's response
- `timestamp`: When interaction occurred
- `command_executed`: Command that was executed

### UserPreferences

```rust
pub struct UserPreferences {
    pub preferred_explanation_style: ExplanationStyle,
    pub safety_level: SafetyLevel,
    pub auto_execute_plans: bool,
    pub max_plan_steps: usize,
    pub preferred_model: Option<String>,
}
```

**Fields**:
- `preferred_explanation_style`: How AI should explain things
- `safety_level`: How cautious to be with commands
- `auto_execute_plans`: Whether to auto-execute AI plans
- `max_plan_steps`: Maximum steps in a plan
- `preferred_model`: Preferred AI model to use

## Error Types

### KaidoError

```rust
pub enum KaidoError {
    /// AI model related errors
    ModelError {
        message: String,
        model_name: String,
    },
    
    /// Command execution errors
    ExecutionError {
        command: String,
        exit_code: Option<i32>,
        stderr: String,
    },
    
    /// Safety rule violations
    SafetyViolation {
        command: String,
        rule_id: String,
        message: String,
    },
    
    /// Configuration errors
    ConfigError {
        file_path: PathBuf,
        message: String,
    },
    
    /// General application errors
    ApplicationError {
        message: String,
        context: Option<String>,
    },
}
```

**Variants**:
- `ModelError`: AI model loading or inference failures
- `ExecutionError`: Command execution failures
- `SafetyViolation`: Safety rule violations
- `ConfigError`: Configuration file problems
- `ApplicationError`: General application errors

## Configuration Contracts

### Model Configuration

```toml
[model]
name = "phi3-mini"
path = "models/phi3-mini.gguf"
type = "local"  # or "cloud"
max_tokens = 2048
temperature = 0.7
```

### Safety Configuration

```toml
[safety]
require_confirmation_for = [
    "rm -rf",
    "sudo",
    "chmod 777",
    "dd if=",
]
auto_confirm_safe_commands = true
log_all_commands = true
```

### Shell Configuration

```toml
[shell]
default_prompt = "kaido> "
history_size = 1000
auto_complete = true
show_execution_time = true
```

## Integration Points

### File System Operations

```rust
pub trait FileSystemOps {
    fn read_file(&self, path: &Path) -> Result<String>;
    fn write_file(&self, path: &Path, content: &str) -> Result<()>;
    fn create_directory(&self, path: &Path) -> Result<()>;
    fn list_directory(&self, path: &Path) -> Result<Vec<PathBuf>>;
}
```

### Process Management

```rust
pub trait ProcessManager {
    fn spawn_process(&self, command: &str, context: &ExecutionContext) -> Result<ProcessHandle>;
    fn wait_for_process(&self, handle: ProcessHandle) -> Result<ProcessResult>;
    fn kill_process(&self, handle: ProcessHandle) -> Result<()>;
}
```

### Logging Interface

```rust
pub trait Logger {
    fn log_command_execution(&self, execution: &CommandExecution);
    fn log_ai_interaction(&self, interaction: &ConversationTurn);
    fn log_safety_event(&self, event: &SafetyWarning);
    fn log_error(&self, error: &KaidoError);
}
```

## Testing Contracts

### Mock Implementations

All interfaces must have mock implementations for testing:

```rust
pub struct MockShellRepl {
    // Test implementation
}

pub struct MockCommandExecutor {
    // Test implementation
}

pub struct MockAIModel {
    // Test implementation
}

pub struct MockSafetyDetector {
    // Test implementation
}
```

### Test Utilities

```rust
pub struct TestContext {
    pub temp_directory: PathBuf,
    pub mock_environment: HashMap<String, String>,
    pub test_session_id: String,
}

impl TestContext {
    pub fn new() -> Self;
    pub fn create_test_file(&self, name: &str, content: &str) -> PathBuf;
    pub fn cleanup(&self);
}
```
