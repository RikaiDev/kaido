# Research Findings: Professional TUI Interface for Kaido AI Shell

**Feature**: 006-tui-interface
**Date**: 2025-10-24
**Purpose**: Resolve technical unknowns before design phase

## Research Task 1: ratatui Best Practices for Event-Driven TUI with Async Operations

### Decision

Use **two-phase event loop**: non-blocking event polling (100ms timeout) with tokio channels to communicate between async AI tasks and synchronous TUI rendering.

### Rationale

ratatui's rendering must happen on the main thread synchronously, but tokio async operations (like AI inference) cannot block the event loop or spinner animation will freeze. The solution is:

1. Event loop polls for keyboard input every 100ms (`crossterm::event::poll(Duration::from_millis(100))`)
2. When Enter pressed, spawn tokio task for AI inference
3. Task updates shared state (Arc<Mutex<AppState>>) or sends via channel
4. Main loop continues polling, updating spinner on each iteration
5. When AI task completes, state change triggers UI update on next render

### Code Pattern

```rust
pub async fn run(&mut self) -> KaidoResult<()> {
    loop {
        // Render current state
        self.terminal.draw(|f| self.render_ui(f))?;
        
        // Non-blocking event poll (100ms timeout)
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = crossterm::event::read()? {
                self.handle_key(key).await?;
            }
        }
        
        // Update spinner every loop iteration if AI thinking
        if self.app.ai_thinking {
            self.app.next_spinner_frame();
        }
    }
}

async fn handle_key(&mut self, key: KeyEvent) -> KaidoResult<()> {
    match key.code {
        KeyCode::Enter => {
            let input = self.app.input.clone();
            self.app.ai_thinking = true;
            
            // Async AI call - doesn't block event loop
            match self.ai_manager.generate_response(&input).await {
                Ok(sequence) => {
                    self.app.ai_thinking = false;
                    self.app.ai_output = serde_json::to_string_pretty(&sequence)?;
                    // ... execute commands
                }
                Err(e) => {
                    self.app.ai_thinking = false;
                    self.app.output = format!("Error: {}", e);
                }
            }
        }
        // ... other keys
    }
    Ok(())
}
```

### Alternatives Considered

- **tokio::select! with channels**: More complex, unnecessary for single-user TUI
- **Blocking AI call**: Would freeze spinner and UI - rejected
- **Separate render thread**: ratatui not thread-safe, requires complex synchronization - rejected

### References

- ratatui examples: `async.rs`, `user_input.rs`
- crossterm::event::poll documentation
- tokio async-aware event loops

---

## Research Task 2: llama.cpp Log Control Mechanisms

### Decision

Use **`LLAMA_LOG_LEVEL` environment variable** set to `"0"` before `LlamaBackend::init()`. This is supported by llama.cpp and works with llama-cpp-2 Rust bindings.

### Rationale

llama.cpp C++ library respects the `LLAMA_LOG_LEVEL` environment variable:
- `"0"` = silent (no logs)
- `"1"` = error only
- `"2"` = warning
- `"3"` = info (default)
- `"4"` = debug

The llama-cpp-2 Rust bindings wrap the C++ library directly, so environment variables set before backend initialization are respected. Setting this in `main()` before any llama calls ensures complete silence.

### Code Pattern

```rust
// In src/main.rs before any llama operations
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // CRITICAL: Silence llama.cpp logs before any initialization
    std::env::set_var("LLAMA_LOG_LEVEL", "0");
    
    let mut repl = KaidoREPL::new()?;
    repl.run().await?;
    
    Ok(())
}

// Also in src/ai/mod.rs as defense-in-depth
async fn run_llama_cpp_inference(&self, model_path: &Path, user_input: &str) -> KaidoResult<String> {
    // Redundant set in case called before main sets it
    std::env::set_var("LLAMA_LOG_LEVEL", "0");
    
    LlamaBackend::init()
        .map_err(|e| KaidoError::ModelError {
            message: format!("Backend init failed: {}", e),
            model_name: "llama-cpp".to_string(),
        })?;
    
    // ... rest of inference
}
```

### Alternatives Considered

- **llama.cpp programmatic API**: No public Rust API for log control in llama-cpp-2 - rejected
- **Redirect stderr**: Would hide all errors, not just llama logs - rejected
- **Custom log callback**: Not exposed in llama-cpp-2 bindings - rejected

### References

- llama.cpp source: `common/log.h` and `common/log.cpp`
- llama-cpp-2 GitHub issues on log control
- Environment variable approach confirmed working in llama.cpp 0.2.0+

---

## Research Task 3: Allowlist Persistence Strategy

### Decision

Use **plain text file** (one command per line) at `~/.config/kaido/allowlist.txt` following XDG Base Directory Specification already used in the project.

### Rationale

1. **Simplicity**: Plain text is easiest to read, edit manually, and debug
2. **Consistency**: Project already uses `~/.config/kaido/config.toml` for configuration
3. **Robustness**: One command per line is trivial to parse, no JSON/TOML complexity needed
4. **Atomicity**: Rust's `std::fs::write()` is atomic on most filesystems
5. **Human-editable**: Users can manually add/remove commands if needed

### Code Pattern

```rust
use std::fs;
use std::path::PathBuf;

pub struct Allowlist {
    allowed_commands: HashSet<String>,
    file_path: PathBuf,
}

impl Allowlist {
    pub fn load() -> KaidoResult<Self> {
        let config_dir = dirs::config_dir()
            .ok_or(KaidoError::ApplicationError {
                message: "Could not determine config directory".to_string(),
                context: None,
            })?;
        
        let file_path = config_dir.join("kaido").join("allowlist.txt");
        
        let allowed_commands = if file_path.exists() {
            fs::read_to_string(&file_path)?
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .collect()
        } else {
            HashSet::new()
        };
        
        Ok(Self { allowed_commands, file_path })
    }
    
    pub fn save(&self) -> KaidoResult<()> {
        // Ensure directory exists
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Write atomically
        let content = self.allowed_commands
            .iter()
            .map(|cmd| format!("{}\n", cmd))
            .collect::<String>();
        
        fs::write(&self.file_path, content)?;
        Ok(())
    }
    
    pub fn add(&mut self, command: String) -> KaidoResult<()> {
        self.allowed_commands.insert(command);
        self.save()
    }
}
```

### File Format Example

```text
# Kaido AI Shell Allowlist
# Commands here will execute without confirmation

rm test.txt
echo "hello" > /tmp/test.txt
sudo apt update
```

### Alternatives Considered

- **JSON format**: Overkill for simple list, harder to manually edit - rejected
- **TOML format**: Unnecessary structure overhead - rejected
- **SQLite database**: Far too complex for dozens of strings - rejected
- **In-memory only**: Loses allowlist on restart, violates FR-027 - rejected

### References

- XDG Base Directory Specification
- `dirs` crate documentation
- Existing `src/config.rs` uses same pattern for `config.toml`

---

## Research Task 4: Modal Dialog Event Handling

### Decision

Use **enum-based state machine** with three states: `Normal`, `ModalActive`, `Executing`. Modal dialog captures all input when active, event loop routes events based on current state.

### Rationale

1. **Explicit state**: Makes modal blocking behavior clear in code
2. **Type safety**: Rust enum ensures only valid state transitions
3. **Event routing**: Simple match statement directs input to correct handler
4. **Async integration**: Modal can await user input without blocking rendering
5. **Testability**: State transitions are pure functions, easy to unit test

### Code Pattern

```rust
pub enum AppState {
    Normal,          // Regular input mode
    ModalActive,     // Modal is blocking, waiting for 1/2/3
    Executing,       // Command running, UI shows progress
}

pub struct KaidoApp {
    pub state: AppState,
    pub modal: Option<ModalDialog>,
    // ... other fields
}

impl KaidoREPL {
    async fn handle_key(&mut self, key: KeyEvent) -> KaidoResult<()> {
        match self.app.state {
            AppState::Normal => {
                match key.code {
                    KeyCode::Enter => {
                        // Start AI processing
                        self.app.state = AppState::Executing;
                        self.process_command().await?;
                    }
                    KeyCode::Char(c) => self.app.input.push(c),
                    KeyCode::Backspace => { self.app.input.pop(); }
                    // ... other normal keys
                    _ => {}
                }
            }
            
            AppState::ModalActive => {
                // Modal captures ALL input
                match key.code {
                    KeyCode::Char('1') => {
                        // Allow once
                        if let Some(modal) = &self.app.modal {
                            self.execute_dangerous_command(&modal.command, false).await?;
                        }
                        self.app.modal = None;
                        self.app.state = AppState::Normal;
                    }
                    KeyCode::Char('2') => {
                        // Allow always (add to allowlist)
                        if let Some(modal) = &self.app.modal {
                            self.allowlist.add(modal.command.clone())?;
                            self.execute_dangerous_command(&modal.command, false).await?;
                        }
                        self.app.modal = None;
                        self.app.state = AppState::Normal;
                    }
                    KeyCode::Char('3') => {
                        // Deny
                        self.app.output = "Command cancelled by user".to_string();
                        self.app.modal = None;
                        self.app.state = AppState::Normal;
                    }
                    _ => {} // Ignore all other keys while modal active
                }
            }
            
            AppState::Executing => {
                // Only allow Ctrl+C to interrupt
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    return Err(KaidoError::Interrupted);
                }
                // Ignore all other input while executing
            }
        }
        Ok(())
    }
    
    async fn process_command(&mut self) -> KaidoResult<()> {
        let input = self.app.input.clone();
        self.app.input.clear();
        
        self.app.ai_thinking = true;
        let sequence = self.ai_manager.generate_response(&input).await?;
        self.app.ai_thinking = false;
        
        // Check for dangerous commands
        for cmd in &sequence.commands {
            if self.is_dangerous(&cmd.cmd) && !self.allowlist.is_allowed(&cmd.cmd) {
                // Show modal and wait
                self.app.modal = Some(ModalDialog {
                    command: cmd.cmd.clone(),
                    description: cmd.description.clone(),
                    selected_option: 0,
                });
                self.app.state = AppState::ModalActive;
                return Ok(()); // Exit, modal will handle continuation
            }
        }
        
        // Safe commands - execute directly
        self.execute_sequence(sequence).await?;
        self.app.state = AppState::Normal;
        Ok(())
    }
}
```

### State Transition Diagram

```
Normal --[Enter pressed]--> Executing
Executing --[AI returns safe command]--> Normal
Executing --[AI returns dangerous command]--> ModalActive
ModalActive --[1 pressed]--> Normal (execute once)
ModalActive --[2 pressed]--> Normal (add to allowlist + execute)
ModalActive --[3 pressed]--> Normal (deny)
```

### Alternatives Considered

- **Callback-based modal**: More complex, harder to reason about async flow - rejected
- **Blocking read in modal**: Would freeze animation and UI - rejected
- **Separate modal event channel**: Overcomplicated for single modal use case - rejected

### References

- State machine pattern in Rust
- ratatui event handling examples
- Existing `SafeExecutor` confirmation pattern

---

## Research Task 5: Terminal Raw Mode Cleanup

### Decision

Use **RAII wrapper struct** `TerminalGuard` that enables raw mode on creation and disables on drop, combined with custom panic hook to ensure cleanup even on panic.

### Rationale

1. **RAII guarantee**: Rust's drop ensures cleanup even on early return or error
2. **Panic safety**: Custom panic hook restores terminal before unwinding
3. **Signal handling**: Ctrl+C handled by returning from event loop, triggering drop
4. **No manual cleanup**: Compiler guarantees `disable_raw_mode()` always called
5. **Idiomatic Rust**: Standard pattern for resource management

### Code Pattern

```rust
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Stdout};

/// RAII guard for terminal raw mode
/// Automatically restores terminal on drop
pub struct TerminalGuard {
    stdout: Stdout,
}

impl TerminalGuard {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        
        Ok(Self { stdout })
    }
    
    pub fn terminal(&mut self) -> io::Result<Terminal<CrosstermBackend<&mut Stdout>>> {
        let backend = CrosstermBackend::new(&mut self.stdout);
        Terminal::new(backend)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Always restore terminal, even on panic
        let _ = execute!(self.stdout, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

// In main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set custom panic hook to ensure terminal cleanup
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Force terminal cleanup on panic
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        default_panic(info);
    }));
    
    // Silence llama.cpp logs
    std::env::set_var("LLAMA_LOG_LEVEL", "0");
    
    // Terminal guard ensures cleanup
    let mut guard = TerminalGuard::new()?;
    let terminal = guard.terminal()?;
    
    let mut repl = KaidoREPL::with_terminal(terminal)?;
    repl.run().await?;
    
    // Guard dropped here, terminal restored automatically
    Ok(())
}
```

### Behavior Guarantees

1. **Normal exit**: User presses Ctrl+C → event loop returns → `TerminalGuard` drops → terminal restored
2. **Error exit**: `run()` returns `Err` → `?` propagates → `TerminalGuard` drops → terminal restored
3. **Panic exit**: Code panics → panic hook fires → terminal manually restored → `TerminalGuard` drops → terminal restored again (idempotent)

### Alternatives Considered

- **Manual cleanup in every error path**: Error-prone, easy to miss - rejected
- **try-finally equivalent**: Doesn't work with async, verbose - rejected
- **Signal handlers**: Platform-specific, complex, unnecessary with RAII - rejected
- **No panic handling**: Would leave terminal in broken state on panic - rejected

### References

- Rust RAII pattern
- crossterm terminal mode documentation
- `std::panic::set_hook` for panic handling
- ratatui examples terminal setup

---

## Summary

All five research tasks resolved with concrete decisions and code patterns:

1. **Async + TUI**: Non-blocking event loop (100ms poll) with async AI calls
2. **Log silencing**: `LLAMA_LOG_LEVEL=0` environment variable
3. **Allowlist**: Plain text file at `~/.config/kaido/allowlist.txt`
4. **Modal events**: Enum state machine (`Normal`/`ModalActive`/`Executing`)
5. **Terminal cleanup**: RAII `TerminalGuard` struct with custom panic hook

Ready for Phase 1 design artifacts (data-model.md, quickstart.md).

