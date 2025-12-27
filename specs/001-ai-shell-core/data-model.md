# Data Model: Kaido AI Shell Core

**Date**: 2025-10-22  
**Feature**: Kaido AI Shell Core  
**Purpose**: Define core data structures and relationships

## Core Entities

### TaskPlan

Represents a multi-step execution plan generated from natural language input.

```rust
struct TaskPlan {
    id: String,                    // Unique identifier for the plan
    original_input: String,        // Original natural language input
    steps: Vec<PlanStep>,          // Ordered list of execution steps
    status: PlanStatus,            // Current execution status
    created_at: DateTime,          // When plan was created
    executed_at: Option<DateTime>, // When execution completed
}
```

**Fields**:
- `id`: UUID for tracking and debugging
- `original_input`: User's natural language request
- `steps`: Ordered commands to execute
- `status`: Pending, Executing, Completed, Failed
- `created_at`: Timestamp for audit trail
- `executed_at`: Completion timestamp

**Validation Rules**:
- Must have at least one step
- Steps must be ordered sequentially
- Original input cannot be empty

### PlanStep

Individual command within a task plan.

```rust
struct PlanStep {
    id: String,                    // Unique step identifier
    command: String,               // Command to execute
    description: String,            // Human-readable description
    expected_outcome: String,      // What this step should accomplish
    status: StepStatus,            // Execution status
    output: Option<String>,        // Command output
    error: Option<String>,         // Error message if failed
    execution_time: Option<Duration>, // How long it took to execute
}
```

**Fields**:
- `id`: UUID for step tracking
- `command`: Shell command to execute
- `description`: AI-generated explanation
- `expected_outcome`: What should happen
- `status`: Pending, Running, Success, Failed
- `output`: Standard output from command
- `error`: Error output if command failed
- `execution_time`: Performance tracking

**Validation Rules**:
- Command cannot be empty
- Description must be non-empty
- Status must be valid enum value

### CommandExecution

Represents a single command execution with metadata.

```rust
struct CommandExecution {
    id: String,                    // Unique execution identifier
    command: String,               // Command that was executed
    working_directory: PathBuf,    // Directory where command was run
    environment: HashMap<String, String>, // Environment variables
    status: ExecutionStatus,       // Success/failure status
    exit_code: Option<i32>,       // Process exit code
    stdout: String,                // Standard output
    stderr: String,                // Standard error
    execution_time: Duration,      // Total execution time
    timestamp: DateTime,           // When command was executed
    is_ai_generated: bool,         // Whether command came from AI
}
```

**Fields**:
- `id`: UUID for execution tracking
- `command`: Exact command executed
- `working_directory`: Current directory
- `environment`: Environment variables snapshot
- `status`: Success, Failed, Timeout, Cancelled
- `exit_code`: Process exit code
- `stdout`: Standard output content
- `stderr`: Error output content
- `execution_time`: How long it took
- `timestamp`: When it was executed
- `is_ai_generated`: Whether AI created this command

**Validation Rules**:
- Command cannot be empty
- Working directory must exist
- Timestamp must be valid
- Exit code must be valid if present

### AIModel

Represents the AI processing capability and configuration.

```rust
struct AIModel {
    name: String,                  // Model identifier (e.g., "phi3-mini")
    model_path: PathBuf,           // Path to GGUF file
    model_type: ModelType,         // Local or Cloud
    config: ModelConfig,           // Model-specific configuration
    status: ModelStatus,           // Loaded, Loading, Failed
    loaded_at: Option<DateTime>,    // When model was loaded
    performance_metrics: PerformanceMetrics, // Speed and memory usage
}
```

**Fields**:
- `name`: Human-readable model name
- `model_path`: File system path to model
- `model_type`: Local GGUF or Cloud API
- `config`: Model-specific settings
- `status`: Current model state
- `loaded_at`: When model became available
- `performance_metrics`: Performance tracking

**Validation Rules**:
- Model path must exist for local models
- Name cannot be empty
- Status must be valid enum value

### SessionState

Represents the current shell environment state.

```rust
struct SessionState {
    working_directory: PathBuf,    // Current working directory
    environment: HashMap<String, String>, // Environment variables
    command_history: Vec<String>,  // History of executed commands
    session_id: String,           // Unique session identifier
    started_at: DateTime,         // When session started
    last_command_at: Option<DateTime>, // Last command execution time
    ai_context: AIContext,        // AI conversation context
}
```

**Fields**:
- `working_directory`: Current directory
- `environment`: Environment variables
- `command_history`: List of executed commands
- `session_id`: Unique session identifier
- `started_at`: Session start time
- `last_command_at`: Last command execution
- `ai_context`: AI conversation state

**Validation Rules**:
- Working directory must exist
- Session ID cannot be empty
- Started at must be valid timestamp

### SafetyRule

Represents a security constraint for dangerous commands.

```rust
struct SafetyRule {
    id: String,                    // Unique rule identifier
    pattern: String,               // Regex pattern to match commands
    description: String,            // Human-readable description
    severity: SeverityLevel,       // How dangerous this is
    requires_confirmation: bool,    // Whether user confirmation needed
    is_enabled: bool,              // Whether rule is active
    created_at: DateTime,          // When rule was created
}
```

**Fields**:
- `id`: UUID for rule tracking
- `pattern`: Regex pattern for command matching
- `description`: What this rule protects against
- `severity`: Low, Medium, High, Critical
- `requires_confirmation`: Whether user must confirm
- `is_enabled`: Whether rule is currently active
- `created_at`: When rule was created

**Validation Rules**:
- Pattern must be valid regex
- Description cannot be empty
- Severity must be valid enum value

## Enums and Status Types

### PlanStatus

```rust
enum PlanStatus {
    Pending,      // Plan created but not started
    Executing,    // Currently running steps
    Completed,    // All steps completed successfully
    Failed,       // One or more steps failed
    Cancelled,    // User cancelled execution
}
```

### StepStatus

```rust
enum StepStatus {
    Pending,      // Step not yet executed
    Running,      // Currently executing
    Success,      // Completed successfully
    Failed,       // Execution failed
    Skipped,      // Skipped due to previous failure
}
```

### ExecutionStatus

```rust
enum ExecutionStatus {
    Success,      // Command completed successfully
    Failed,       // Command failed with error
    Timeout,      // Command timed out
    Cancelled,    // User cancelled command
    Interrupted,  // Process was interrupted
}
```

### ModelType

```rust
enum ModelType {
    LocalGGUF,    // Local GGUF model file
    CloudAPI,     // Cloud API service
}
```

### ModelStatus

```rust
enum ModelStatus {
    NotLoaded,    // Model not yet loaded
    Loading,      // Currently loading model
    Loaded,       // Model ready for inference
    Failed,       // Model failed to load
}
```

### SeverityLevel

```rust
enum SeverityLevel {
    Low,          // Informational warning
    Medium,       // Caution required
    High,         // Dangerous operation
    Critical,     // System-threatening operation
}
```

## Relationships

### TaskPlan → PlanStep
- One-to-many relationship
- TaskPlan contains ordered list of PlanStep
- PlanStep belongs to exactly one TaskPlan

### PlanStep → CommandExecution
- One-to-one relationship
- Each PlanStep generates one CommandExecution
- CommandExecution tracks the actual execution

### SessionState → CommandExecution
- One-to-many relationship
- SessionState tracks all CommandExecution in session
- CommandExecution belongs to one SessionState

### AIModel → TaskPlan
- One-to-many relationship
- AIModel generates multiple TaskPlan
- TaskPlan created by one AIModel

### SafetyRule → CommandExecution
- Many-to-many relationship
- SafetyRule can match multiple CommandExecution
- CommandExecution can trigger multiple SafetyRule

## State Transitions

### TaskPlan Status Flow
```
Pending → Executing → Completed
    ↓         ↓
  Cancelled  Failed
```

### PlanStep Status Flow
```
Pending → Running → Success
    ↓        ↓
  Skipped  Failed
```

### Model Loading Flow
```
NotLoaded → Loading → Loaded
    ↓          ↓
  Failed    Failed
```

## Data Persistence

### Configuration Storage
- TOML files for user configuration
- Default configuration in `config/default.toml`
- User overrides in `~/.config/kaido/config.toml`

### Session Data
- In-memory during session
- Optional persistence for command history
- Log files for audit trail

### Model Storage
- GGUF files in `models/` directory
- Model metadata in configuration
- Performance metrics in memory only

## Validation and Constraints

### Input Validation
- All string fields must be non-empty where required
- Paths must exist and be accessible
- Timestamps must be valid and chronological
- UUIDs must be valid format

### Business Rules
- Only one model can be loaded at a time
- Command history limited to last 1000 commands
- Safety rules must have valid regex patterns
- Plan steps must be executable commands
