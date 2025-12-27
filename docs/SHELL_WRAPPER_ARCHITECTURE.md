# Kaido Shell Wrapper Architecture

## Vision

Transform Kaido from a "passive diagnosis tool" to an "active mentor shell" - a shell wrapper that observes user commands and teaches them to understand errors.

```
┌─────────────────────────────────────────────────────────────┐
│                      CURRENT MODEL                          │
├─────────────────────────────────────────────────────────────┤
│  User: "nginx won't start"                                  │
│  Kaido: Let me diagnose... (executes commands)              │
│  Kaido: Here's the answer: stop apache first                │
│                                                             │
│  Problem: User learns nothing, stays dependent              │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                      NEW MODEL                              │
├─────────────────────────────────────────────────────────────┤
│  kaido> nginx -t                                            │
│  nginx: [emerg] unknown directive "proxy_passs" in ...      │
│                                                             │
│  ┌─ MENTOR ───────────────────────────────────────────────┐ │
│  │ Key message: "unknown directive 'proxy_passs'"         │ │
│  │                                                        │ │
│  │ This tells you:                                        │ │
│  │   • Line 42 has an unrecognized directive              │ │
│  │   • "proxy_passs" looks like a typo of "proxy_pass"    │ │
│  │                                                        │ │
│  │ Search: nginx proxy_pass configuration                 │ │
│  │ Next: vim /etc/nginx/nginx.conf +42                    │ │
│  └────────────────────────────────────────────────────────┘ │
│  kaido>                                                     │
│                                                             │
│  Result: User learns to read errors, becomes independent    │
└─────────────────────────────────────────────────────────────┘
```

## Core Philosophy

| Aspect | Current | New |
|--------|---------|-----|
| Who executes | AI executes commands | User executes commands |
| AI role | Problem solver | Observer & mentor |
| When AI speaks | Always | Only when errors occur |
| Goal | Give answers | Teach understanding |
| Outcome | Dependency | Independence |

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     KAIDO SHELL                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │   Input     │───>│  Executor   │───>│   Output    │     │
│  │   Parser    │    │   (PTY)     │    │   Capture   │     │
│  └─────────────┘    └─────────────┘    └──────┬──────┘     │
│                                               │             │
│                                               v             │
│                                        ┌─────────────┐      │
│                                        │   Error     │      │
│                                        │  Detector   │      │
│                                        └──────┬──────┘      │
│                                               │             │
│                           ┌───────────────────┼─────────┐   │
│                           │ Error detected?   │         │   │
│                           │                   v         │   │
│                           │ YES        ┌─────────────┐  │   │
│                           │            │   Mentor    │  │   │
│                           │            │   Engine    │  │   │
│                           │            └──────┬──────┘  │   │
│                           │                   │         │   │
│                           │                   v         │   │
│                           │            ┌─────────────┐  │   │
│                           │            │  Learning   │  │   │
│                           │            │  Tracker    │  │   │
│                           │            └─────────────┘  │   │
│                           └─────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Component Design

### 1. Shell Core (`src/shell/`)

```rust
pub struct KaidoShell {
    config: Config,
    pty: PtySession,
    error_detector: ErrorDetector,
    mentor: MentorEngine,
    history: ShellHistory,
    learning_tracker: LearningTracker,
}

impl KaidoShell {
    /// Main shell loop
    pub async fn run(&mut self) -> Result<()> {
        self.display_welcome();

        loop {
            let input = self.read_line()?;

            if self.is_builtin(&input) {
                self.handle_builtin(&input)?;
                continue;
            }

            // Execute command and capture output
            let result = self.execute_with_pty(&input).await?;

            // Check for errors
            if let Some(error_info) = self.error_detector.analyze(&result) {
                // Mentor intervention
                let guidance = self.mentor.generate_guidance(&error_info).await?;
                self.display_mentor_block(&guidance);

                // Track learning
                self.learning_tracker.record_error(&error_info);
            }
        }
    }
}
```

### 2. PTY Executor (`src/shell/pty.rs`)

Why PTY (Pseudo-Terminal)?
- Preserves colors and formatting from commands
- Handles interactive programs (vim, less, top)
- Captures both stdout and stderr properly
- Supports signals (Ctrl+C, Ctrl+Z)

```rust
pub struct PtySession {
    master: PtyMaster,
    child: Child,
}

pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration: Duration,
    pub command: String,
}
```

### 3. Error Detector (`src/mentor/detector.rs`)

```rust
pub struct ErrorDetector {
    patterns: Vec<ErrorPattern>,
}

pub struct ErrorInfo {
    pub error_type: ErrorType,
    pub key_message: String,
    pub full_output: String,
    pub exit_code: i32,
    pub detected_patterns: Vec<PatternMatch>,
}

pub enum ErrorType {
    CommandNotFound,
    PermissionDenied,
    FileNotFound,
    SyntaxError,
    ConnectionRefused,
    ConfigurationError,
    ResourceNotFound,
    AuthenticationFailed,
    Unknown,
}
```

### 4. Mentor Engine (`src/mentor/engine.rs`)

```rust
pub struct MentorEngine {
    llm: Box<dyn LLMBackend>,
    pattern_db: PatternDatabase,
}

pub struct MentorGuidance {
    pub key_message: String,           // What to focus on
    pub explanation: String,           // What it means
    pub search_keywords: Vec<String>,  // What to Google
    pub next_steps: Vec<String>,       // What to try next
    pub related_concepts: Vec<String>, // What to learn
}

impl MentorEngine {
    pub async fn generate_guidance(&self, error: &ErrorInfo) -> Result<MentorGuidance> {
        // 1. Try pattern matching first (fast, no LLM)
        if let Some(guidance) = self.pattern_db.match_error(error) {
            return Ok(guidance);
        }

        // 2. Fall back to LLM for unknown errors
        self.generate_with_llm(error).await
    }
}
```

### 5. Learning Tracker (`src/learning/tracker.rs`)

```rust
pub struct LearningTracker {
    db: SqliteConnection,
}

pub struct LearningProgress {
    pub errors_encountered: HashMap<ErrorType, u32>,
    pub errors_resolved: HashMap<ErrorType, u32>,
    pub concepts_learned: Vec<String>,
    pub skill_level: SkillLevel,
}

pub enum SkillLevel {
    Beginner,      // Needs detailed explanations
    Intermediate,  // Shorter hints
    Advanced,      // Minimal intervention
}
```

## Error Pattern Database

Pre-built patterns for common errors (no LLM needed):

```rust
// Example patterns
ErrorPattern {
    regex: r"command not found: (\w+)",
    error_type: ErrorType::CommandNotFound,
    key_extraction: |m| format!("Command '{}' is not installed", m[1]),
    guidance: MentorGuidance {
        explanation: "The command doesn't exist on this system",
        search_keywords: vec!["install {}", "how to install {}"],
        next_steps: vec![
            "Check if it's spelled correctly",
            "Install with: brew install {} (macOS) or apt install {} (Linux)",
        ],
    }
}

ErrorPattern {
    regex: r"Permission denied",
    error_type: ErrorType::PermissionDenied,
    guidance: MentorGuidance {
        explanation: "You don't have permission to access this resource",
        search_keywords: vec!["linux file permissions", "chmod"],
        next_steps: vec![
            "Check file permissions: ls -la",
            "Try with sudo (if appropriate)",
        ],
    }
}
```

## Display Format

### Mentor Block

```
┌─ MENTOR ────────────────────────────────────────────────────┐
│                                                              │
│ Key: "nginx: [emerg] unknown directive 'proxy_passs'"       │
│      ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~               │
│                                                              │
│ This means:                                                  │
│   nginx found a directive it doesn't recognize.              │
│   "proxy_passs" is likely a typo of "proxy_pass"            │
│                                                              │
│ Location: /etc/nginx/nginx.conf:42                          │
│                                                              │
│ Search: nginx proxy_pass syntax                             │
│                                                              │
│ Next steps:                                                  │
│   1. vim /etc/nginx/nginx.conf +42                          │
│   2. Fix the typo: proxy_passs → proxy_pass                 │
│   3. Test again: nginx -t                                   │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### Skill Level Adaptation

**Beginner** (verbose):
```
┌─ MENTOR ────────────────────────────────────────────────────┐
│ Key: "Permission denied"                                     │
│                                                              │
│ What this means:                                             │
│   In Linux, every file has permissions that control who     │
│   can read, write, or execute it. This error means you      │
│   don't have the required permission.                       │
│                                                              │
│ Understanding permissions:                                   │
│   -rw-r--r--  = owner can read/write, others can read       │
│   drwxr-xr-x  = directory, owner has full access            │
│                                                              │
│ Search: "linux file permissions explained"                  │
│                                                              │
│ Try: ls -la <file>  (see current permissions)              │
└──────────────────────────────────────────────────────────────┘
```

**Advanced** (concise):
```
┌─ MENTOR ─────────────────────────┐
│ Permission denied                │
│ → Check: ls -la | Try: sudo      │
└──────────────────────────────────┘
```

## Reusable Components from Current Codebase

| Component | Status | Notes |
|-----------|--------|-------|
| `AIManager` | ✅ Reuse | LLM backend abstraction |
| `GeminiBackend` | ✅ Reuse | Cloud API |
| `OllamaBackend` | ✅ Reuse | Local inference |
| `Config` | ✅ Reuse | Configuration system |
| `AuditLogger` | ✅ Reuse | Session tracking |
| `Tool` trait | ⚠️ Adapt | Risk classification useful |
| `ErrorExplanation` | ⚠️ Adapt | Extend for mentor guidance |
| `REPL` | ❌ Replace | New shell wrapper needed |
| `AgentLoop` | ❌ Replace | No longer AI-driven |

## Configuration

```toml
# ~/.kaido/config.toml

[shell]
# When to show mentor guidance
mentor_trigger = "on_error"  # "on_error" | "always" | "never"

# Verbosity based on skill level
auto_adjust_verbosity = true

# Show mentor for exit codes
error_exit_codes = [1, 2, 126, 127, 128]

[mentor]
# Use LLM for unknown errors
use_llm_fallback = true

# Show search suggestions
show_search_keywords = true

# Show next steps
show_next_steps = true

[learning]
# Track learning progress
enabled = true

# Adapt explanations to skill level
adaptive_verbosity = true
```

## Migration Path

### Phase 1: Shell Foundation
- [ ] PTY-based command execution
- [ ] Basic shell loop with readline
- [ ] Output capture (stdout/stderr)
- [ ] Signal handling

### Phase 2: Error Detection
- [ ] Exit code detection
- [ ] Error pattern matching
- [ ] Key message extraction

### Phase 3: Mentor System
- [ ] Pattern-based guidance (no LLM)
- [ ] LLM fallback for unknown errors
- [ ] Mentor display formatting

### Phase 4: Learning Tracker
- [ ] Error history recording
- [ ] Skill level detection
- [ ] Adaptive verbosity

### Phase 5: Polish
- [ ] Shell builtins (cd, export, alias)
- [ ] History with search
- [ ] Tab completion
- [ ] Themes and customization
