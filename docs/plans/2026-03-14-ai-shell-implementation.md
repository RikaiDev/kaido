# AI Shell Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement a complete AI-native shell with natural language input, AI translation, and learning features.

**Architecture:** New shell core built from scratch, separate from current Kaido CLI. Shell runs standalone with its own command parsing, execution, and AI integration.

**Tech Stack:** Rust, rustyline (for REPL), Ollama (for local AI), SQLite (for learning data)

---

## Phase 1: Shell Core Foundation

### Task 1: Create Shell Project Structure

**Files:**
- Create: `src/shell/core.rs` - Shell engine core
- Create: `src/shell/parser.rs` - Command parser
- Create: `src/shell/executor.rs` - Command executor
- Modify: `src/shell/mod.rs` - Export new modules

**Step 1: Create minimal shell core**

```rust
// src/shell/core.rs
pub struct Shell {
    pub running: bool,
}

impl Shell {
    pub fn new() -> Self {
        Self { running: true }
    }
    
    pub fn run(&mut self) -> Result<()> {
        while self.running {
            // Read input, parse, execute loop
        }
        Ok(())
    }
}
```

**Step 2: Run cargo build to verify**

```bash
cd .worktrees/ai-shell && cargo build
```

Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src/shell/core.rs src/shell/parser.rs src/shell/executor.rs src/shell/mod.rs
git commit -m "feat(shell): add shell core structure"
```

---

### Task 2: Implement Command Parser

**Files:**
- Modify: `src/shell/parser.rs`

**Step 1: Write test for command parsing**

```rust
// tests/shell/parser_test.rs
#[test]
fn test_parse_simple_command() {
    let parser = CommandParser::new();
    let result = parser.parse("ls -la");
    assert_eq!(result.command, "ls");
    assert_eq!(result.args, vec!["-la"]);
}

#[test]
fn test_parse_pipeline() {
    let parser = CommandParser::new();
    let result = parser.parse("ls | grep foo");
    assert_eq!(result.commands.len(), 2);
}
```

**Step 2: Run test to verify it fails**

```bash
cargo test parser_test
```

Expected: FAIL - functions not defined

**Step 3: Write parser implementation**

```rust
// src/shell/parser.rs
#[derive(Debug)]
pub struct ParsedCommand {
    pub command: String,
    pub args: Vec<String>,
    pub pipes_to: Option<Box<ParsedCommand>>,
}

pub struct CommandParser;

impl CommandParser {
    pub fn new() -> Self;
    
    pub fn parse(&self, input: &str) -> Result<ParsedCommand> {
        // Split by | for pipeline
        // Parse command and args
        // Return structured result
    }
}
```

**Step 4: Run test to verify it passes**

```bash
cargo test parser_test
```

Expected: PASS

**Step 5: Commit**

```bash
git add src/shell/parser.rs tests/shell/parser_test.rs
git commit -m "feat(shell): add command parser with pipeline support"
```

---

### Task 3: Implement Command Executor

**Files:**
- Modify: `src/shell/executor.rs`

**Step 1: Write test for command execution**

```rust
// tests/shell/executor_test.rs
#[test]
fn test_execute_simple_command() {
    let executor = CommandExecutor::new();
    let result = executor.execute("echo", &["hello"]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().output, "hello\n");
}
```

**Step 2: Run test to verify it fails**

```bash
cargo test executor_test
```

Expected: FAIL

**Step 3: Write executor implementation**

```rust
// src/shell/executor.rs
use std::process::{Command, Output};

pub struct CommandExecutor;

impl CommandExecutor {
    pub fn new() -> Self;
    
    pub fn execute(&self, command: &str, args: &[&str]) -> Result<Output> {
        let mut cmd = Command::new(command);
        cmd.args(args);
        cmd.output().map_err(|e| e.into())
    }
}
```

**Step 4: Run test to verify it passes**

**Step 5: Commit**

---

## Phase 2: AI Integration

### Task 4: AI Processor - Intent Detection

**Files:**
- Create: `src/shell/ai.rs` - AI processing module
- Create: `src/shell/translator.rs` - Command translation

**Step 1: Write test for intent detection**

```rust
// tests/shell/ai_test.rs
#[test]
fn test_detect_natural_language() {
    let ai = AIProcessor::new();
    
    // Natural language should be flagged for AI processing
    assert!(ai.is_natural_language("show me files"));
    assert!(!ai.is_natural_language("ls -la"));
}
```

**Step 2: Implement AI processor**

```rust
// src/shell/ai.rs
pub struct AIProcessor {
    // Ollama client
}

impl AIProcessor {
    pub fn new() -> Self;
    
    pub fn is_natural_language(&self, input: &str) -> bool {
        // Check if input contains natural language patterns
        // vs shell commands
    }
    
    pub async fn translate(&self, input: &str) -> Result<Translation> {
        // Call Ollama to translate intent to command
    }
}
```

**Step 3: Commit**

---

### Task 5: AI Translation with Display

**Files:**
- Modify: `src/shell/translator.rs`

**Step 1: Write test for translation display**

```rust
#[test]
fn test_translation_display() {
    let translation = Translation {
        original: "start nginx".to_string(),
        intent: "Start nginx service".to_string(),
        command: "sudo systemctl start nginx".to_string(),
        explanation: "This will start nginx, requires sudo".to_string(),
    };
    
    let display = translation.to_display_string();
    assert!(display.contains("Intent:"));
    assert!(display.contains("Translate:"));
}
```

**Step 2: Implement translation with explanation**

```rust
// src/shell/translator.rs
#[derive(Debug)]
pub struct Translation {
    pub original: String,
    pub intent: String,
    pub command: String,
    pub explanation: String,
}

impl Translation {
    pub fn to_display_string(&self) -> String {
        format!(
            "→ Intent: {}\n→ Translate: {}\n→ {}",
            self.intent, self.command, self.explanation
        )
    }
}
```

**Step 3: Commit**

---

### Task 6: Error Explanation

**Files:**
- Modify: `src/shell/ai.rs` - Add explain_error method

**Step 1: Write test for error explanation**

```rust
#[test]
fn test_error_explanation() {
    let ai = AIProcessor::new();
    let error_output = "nginx: [emerg] bind() to 0.0.0.0:80 failed";
    
    let explanation = ai.explain_error(error_output);
    assert!(explanation.contains("Port 80"));
    assert!(explanation.contains("already in use"));
}
```

**Step 2: Implement with pattern matching + AI fallback**

```rust
pub fn explain_error(&self, error: &str) -> String {
    // First try pattern matching (fast)
    if let Some(exp) = self.pattern_explain(error) {
        return exp;
    }
    // Fallback to AI
    self.ai_explain(error).await
}
```

**Step 3: Commit**

---

## Phase 3: Learning Features

### Task 7: Learning Tracker

**Files:**
- Create: `src/shell/learning.rs` - Learning progress tracking

**Step 1: Define skill categories**

```rust
// src/shell/learning.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillCategory {
    FileOperations,
    ProcessManagement,
    NetworkDiagnostics,
    Docker,
    Nginx,
    // etc
}

pub struct Skill {
    pub category: SkillCategory,
    pub name: String,
    pub commands_learned: Vec<String>,
    pub mastery_level: u8, // 0-100
}
```

**Step 2: Implement progress tracking**

```rust
pub struct LearningTracker {
    skills: HashMap<SkillCategory, Skill>,
}

impl LearningTracker {
    pub fn record_command(&mut self, command: &str) {
        // Detect skill category from command
        // Update mastery
    }
    
    pub fn get_progress(&self) -> ProgressReport {
        // Generate progress summary
    }
}
```

**Step 3: Commit**

---

### Task 8: Progress Command

**Files:**
- Modify: `src/shell/core.rs` - Add built-in commands

**Step 1: Implement /progress command**

```rust
impl Shell {
    fn handle_builtin(&mut self, cmd: &str) -> bool {
        match cmd {
            "/progress" | "progress" => {
                let progress = self.learning.get_progress();
                println!("{}", progress.display());
                true
            }
            _ => false
        }
    }
}
```

**Step 2: Test manually**

```bash
kaido> /progress
📊 Your Progress:
  ✓ File Operations (ls, cd, cp)
  → Process Management (ps, kill) - 3/5
  ○ Network Diagnostics
```

**Step 3: Commit**

---

## Phase 4: Ecosystem Foundation

### Task 9: Skills Knowledge Base

**Files:**
- Create: `src/shell/skills/` - Skills module
- Create: `skills/` directory in project root

**Step 1: Create skills directory structure**

```
skills/
├── nginx/
│   ├── errors/
│   │   ├── 502.yaml
│   │   └── 500.yaml
│   └── commands.yaml
├── docker/
│   ├── errors/
│   └── commands.yaml
└── common/
    └── port-issues.yaml
```

**Step 2: Create example skill**

```yaml
# skills/nginx/errors/502.yaml
pattern: "502 Bad Gateway"
causes:
  - Backend service not running
  - Backend timeout
  - Network issue
diagnosis:
  - systemctl status php-fpm
  - tail /var/log/nginx/error.log
teaches:
  - "502 = upstream failed"
  - "Check backend first"
```

**Step 3: Load skills in shell startup**

```rust
impl Shell {
    fn load_skills(&self) -> SkillsRegistry {
        // Load all YAML files from skills/
        // Parse into registry
    }
}
```

**Step 4: Commit**

---

### Task 10: Plugin System (Basic)

**Files:**
- Create: `src/shell/plugin.rs` - Plugin system

**Step 1: Define plugin trait**

```rust
pub trait Plugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn on_command(&self, cmd: &str) -> Option<HookResult>;
}

pub enum HookResult {
    Modified(String),
    Suggestion(String),
    None,
}
```

**Step 2: Implement plugin loader**

```rust
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn load_plugins(&mut self) {
        // Load from ~/.kaido/plugins/
    }
}
```

**Step 3: Commit**

---

## Phase 5: Integration & Testing

### Task 11: Main Shell Loop Integration

**Files:**
- Modify: `src/shell/core.rs` - Integrate all components

**Step 1: Write integration test**

```rust
#[tokio::test]
async fn test_shell_nl_to_command() {
    let mut shell = Shell::new();
    
    // User types natural language
    shell.process_input("show me all running processes").await;
    
    // Should see translation
    // Should execute ps aux
}
```

**Step 2: Implement full loop**

```rust
impl Shell {
    pub async fn run(&mut self) -> Result<()> {
        while self.running {
            let input = self.readline()?;
            
            // Check for built-ins
            if self.handle_builtin(&input) {
                continue;
            }
            
            // Check for natural language
            if self.ai.is_natural_language(&input) {
                let translation = self.ai.translate(&input).await?;
                println!("{}", translation.to_display_string());
                
                // Ask user to confirm
                print!("Execute? [Y/n]: ");
                // ... handle confirmation
            }
            
            // Execute command
            self.executor.execute(&parsed)?;
        }
    }
}
```

**Step 3: Commit**

---

### Task 12: End-to-End Testing

**Files:**
- Create: `tests/shell/e2e_test.rs`

**Step 1: Write e2e tests**

```rust
#[tokio::test]
async fn test_full_user_flow() {
    // 1. Start shell
    // 2. Type natural language
    // 3. Confirm translation
    // 4. Execute
    // 5. See error explanation
    // 6. Check progress updated
}
```

**Step 2: Run all tests**

```bash
cargo test
```

Expected: All pass

**Step 3: Commit**

---

## Summary

**Total Tasks:** 12

**Phase 1 (Tasks 1-3):** Shell core foundation - parser, executor
**Phase 2 (Tasks 4-6):** AI integration - translation, explanation
**Phase 3 (Tasks 7-8):** Learning features - progress tracking
**Phase 4 (Tasks 9-10):** Ecosystem - skills, plugins
**Phase 5 (Tasks 11-12):** Integration and testing

---

**Plan complete and saved to `docs/plans/2026-03-14-ai-shell-design.md`. Two execution options:**

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

**Which approach?**
