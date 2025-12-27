# Developer Quickstart: Professional TUI Interface for Kaido AI Shell

**Feature**: 006-tui-interface
**Date**: 2025-10-24
**Purpose**: Get developers set up to work on TUI implementation

## Prerequisites

### System Requirements

- **Rust**: 1.75+ (verify with `rustc --version`)
- **Cargo**: Bundled with Rust
- **Terminal**: Unicode-capable (iTerm2, Alacritty, Windows Terminal, GNOME Terminal)
- **Platform**: macOS, Linux, or Windows with WSL2

### Already Installed (Project Dependencies)

These are already in the project's `Cargo.toml`:
- `tokio = { version = "1.0", features = ["full"] }`
- `crossterm = "0.27"`
- `llama-cpp-2 = { git = "https://github.com/utilityai/llama-cpp-rs" }`
- `serde_json = "1.0"`

### New Dependencies to Add

Run this command to add ratatui:

```bash
cd /Users/gloomcheng/Workspace/RikaiDev/kaido-ai
cargo add ratatui@0.27
```

Or manually add to `Cargo.toml`:

```toml
[dependencies]
ratatui = "0.27"
```

---

## Project Setup

### 1. Clone and Branch

```bash
# If not already on the branch
git checkout 006-tui-interface

# Verify you're on the right branch
git branch --show-current
# Should output: 006-tui-interface
```

### 2. Build the Project

```bash
cargo build
```

**Expected Output**: Successful compilation with zero warnings (per constitution).

If you see warnings:
- `unused import: Command` in `src/ai/mod.rs`: Will be fixed in implementation
- `method execute_command is never used` in `src/shell/executor.rs`: Will be addressed or removed
- `field error_output is never read` in `src/utils/mod.rs`: May need to be removed

### 3. Run the Current Version

```bash
cargo run
```

**Current Behavior**: Text-based REPL with `kaido> ` prompt. No TUI yet.

To exit: Press `Ctrl+C` or type `exit`.

---

## Development Workflow

### Phase 1: Log Silencing (Quick Win)

**Goal**: Silence llama.cpp verbose output

**Files to Modify**:
1. `src/main.rs`: Add `std::env::set_var("LLAMA_LOG_LEVEL", "0");` before REPL init
2. `src/ai/mod.rs`: Add same in `run_llama_cpp_inference()` as defense-in-depth

**Test**:
```bash
cargo run
# Type: "list files"
# Observe: NO llama.cpp model loading logs should appear
```

**Expected**: Clean output, only TUI or REPL prompt visible.

---

### Phase 2: Basic TUI Structure

**Goal**: Create skeleton UI module with split layout

**New Files to Create**:
1. `src/ui/mod.rs`
2. `src/ui/app.rs`
3. `src/ui/layout.rs`
4. `src/ui/spinner.rs`
5. `src/ui/modal.rs`

**File Structure**:
```bash
mkdir src/ui
touch src/ui/mod.rs src/ui/app.rs src/ui/layout.rs src/ui/spinner.rs src/ui/modal.rs
```

**Minimal `src/ui/mod.rs`**:
```rust
pub mod app;
pub mod layout;
pub mod spinner;
pub mod modal;

pub use app::KaidoApp;
pub use layout::create_layout;
pub use spinner::{get_spinner_frame, SPINNER_FRAMES};
pub use modal::ModalDialog;
```

**Test**:
```bash
# In src/main.rs, add:
mod ui;

cargo build
# Should compile with zero errors
```

---

### Phase 3: TUI Event Loop

**Goal**: Replace text REPL with ratatui event loop

**Files to Modify**:
1. `src/main.rs`: Initialize terminal guard, create TUI terminal
2. `src/shell/repl.rs`: Complete rewrite to use ratatui

**Development Steps**:

1. **Read research findings**: See `specs/006-tui-interface/research.md` for:
   - Event loop pattern (Research Task 1)
   - Terminal guard pattern (Research Task 5)
   - State machine pattern (Research Task 4)

2. **Implement TerminalGuard**:
   - Create wrapper struct for raw mode
   - Implement `Drop` trait for cleanup
   - Add custom panic hook

3. **Update KaidoREPL**:
   - Add `KaidoApp` field
   - Add `Terminal<CrosstermBackend>` field
   - Replace readline loop with `event::poll()` loop
   - Implement `render_ui()` method

**Test**:
```bash
cargo run
# Should see TUI with borders (even if content empty)
# Press Ctrl+C to exit
# Verify terminal restored (cursor visible, no raw mode)
```

---

### Phase 4: Spinner Animation

**Goal**: Show animated spinner while AI thinking

**Implementation**:

1. **Define spinner frames** in `src/ui/spinner.rs`:
```rust
pub const SPINNER_FRAMES: &[&str] = &[
    "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"
];

pub fn get_spinner_frame(index: usize) -> &'static str {
    SPINNER_FRAMES[index % SPINNER_FRAMES.len()]
}
```

2. **Update render loop** to cycle frames every 100ms

**Test**:
```bash
cargo run
# Type: "list files"
# Observe: Spinner should animate smoothly in right panel
# Should see box characters rotating
```

---

### Phase 5: Modal Dialog

**Goal**: Show confirmation dialog for dangerous commands

**Implementation**:

1. **Create ModalDialog struct** in `src/ui/modal.rs`:
   - Fields: `command`, `description`, `selected_option`
   - Method: `render()` using centered_rect helper

2. **Update event loop** to handle modal state:
   - Add `AppState` enum to `KaidoApp`
   - Route key events based on state
   - Show modal when dangerous command detected

**Test**:
```bash
cargo run
# Type: "delete test.txt" (AI should generate "rm test.txt")
# Observe: Modal should appear centered
# Press 1/2/3 to interact
# Modal should close after selection
```

---

## Testing Guide

### Manual Testing Checklist

Run `cargo run` and test each scenario:

- [ ] **Basic launch**: App starts, shows split panels, no llama logs
- [ ] **Spinner animation**: Type command, see smooth rotating spinner
- [ ] **Ctrl+T toggle**: Press Ctrl+T, see JSON output instead of spinner
- [ ] **Safe command**: Type "show files", executes without modal
- [ ] **Dangerous command**: Type "delete test", shows modal
- [ ] **Modal option 1**: Press 1, command executes once
- [ ] **Modal option 2**: Press 2, command added to allowlist
- [ ] **Modal option 3**: Press 3, command cancelled
- [ ] **Allowlist persistence**: Restart app, previously allowed command bypasses modal
- [ ] **Terminal cleanup**: Press Ctrl+C, terminal restored correctly
- [ ] **Window resize**: Resize terminal, layout adjusts

### Unit Testing

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test ui::

# Run with output
cargo test -- --nocapture
```

**Test Files to Create**:
- `tests/unit/ui/app_test.rs`: Test `KaidoApp` state transitions
- `tests/unit/ui/layout_test.rs`: Test split calculations
- `tests/unit/ui/spinner_test.rs`: Test frame cycling
- `tests/integration/tui_smoke_test.rs`: Test full TUI launch and teardown

---

## Debugging Tips

### Issue: Terminal Stuck in Raw Mode

**Symptoms**: After crash, terminal doesn't show cursor or echo input

**Solution**:
```bash
reset
# Or
stty sane
```

**Prevention**: Ensure `TerminalGuard` is always used, check panic hook is installed

---

### Issue: Spinner Not Animating

**Symptoms**: Spinner stuck on one frame

**Possible Causes**:
1. Event loop blocking on AI call (should be async)
2. `next_spinner_frame()` not called in loop
3. Render not called frequently enough

**Debug**:
```rust
// Add to event loop
eprintln!("Spinner index: {}", self.app.spinner_index);
```

---

### Issue: Modal Not Centered

**Symptoms**: Modal appears in corner or off-screen

**Possible Causes**:
1. `centered_rect()` calculation incorrect
2. Terminal size too small (< 80 columns)
3. Percentage values inverted

**Debug**:
```rust
// In modal.rs
eprintln!("Modal area: {:?}, Terminal size: {:?}", popup_area, area);
```

---

### Issue: llama.cpp Logs Still Appearing

**Symptoms**: Verbose model loading logs despite `LLAMA_LOG_LEVEL=0`

**Possible Causes**:
1. Environment variable set after `LlamaBackend::init()`
2. Variable name incorrect (check spelling)
3. llama-cpp-2 version doesn't support this variable

**Debug**:
```rust
// Before backend init
std::env::set_var("LLAMA_LOG_LEVEL", "0");
eprintln!("LLAMA_LOG_LEVEL set to: {:?}", std::env::var("LLAMA_LOG_LEVEL"));
```

**Fallback**: If environment variable doesn't work, check llama-cpp-2 docs for alternative silencing methods.

---

## Code Style Guidelines

### Per Constitution

1. **Zero warnings**: Fix all compiler warnings before committing
2. **No mock implementations**: All code must be functional
3. **Manual modifications**: No batch scripting, edit files individually
4. **No emojis**: Use only ASCII characters in code and comments
5. **Remove unused code**: Delete, don't comment out or suppress warnings

### Rust Best Practices

1. **Error handling**: Use `KaidoResult` and `?` operator, never `unwrap()` in production code
2. **Naming**: Snake_case for functions/variables, PascalCase for types
3. **Documentation**: Add doc comments for public APIs
4. **Async**: Use `async`/`await` for I/O operations, keep UI rendering sync
5. **Lifetime**: Avoid explicit lifetimes unless necessary, prefer owned types

---

## Common Pitfalls

### Pitfall 1: Blocking the Event Loop

**Wrong**:
```rust
// AI call blocks event loop, freezes spinner
let sequence = self.ai_manager.generate_response(&input).await?;
```

**Right**:
```rust
// Set thinking flag, return to event loop, spinner animates
self.app.ai_thinking = true;
// Event loop continues, spinner updates
match self.ai_manager.generate_response(&input).await {
    Ok(seq) => {
        self.app.ai_thinking = false;
        // ...
    }
    // ...
}
```

---

### Pitfall 2: Not Restoring Terminal

**Wrong**:
```rust
// Panic happens, terminal left in raw mode
enable_raw_mode()?;
// ... code that might panic
disable_raw_mode()?; // Never reached on panic
```

**Right**:
```rust
// TerminalGuard ensures cleanup even on panic
let _guard = TerminalGuard::new()?;
// ... code that might panic
// Drop guarantee restores terminal
```

---

### Pitfall 3: Modal State Leakage

**Wrong**:
```rust
// Modal never cleared, stays visible forever
self.app.modal = Some(ModalDialog { ... });
// User presses key, modal not removed
```

**Right**:
```rust
// Always clear modal after handling
match key.code {
    KeyCode::Char('1') => {
        // ... handle option
        self.app.modal = None; // Critical!
        self.app.state = AppState::Normal;
    }
}
```

---

## Resources

### Documentation

- **ratatui**: https://ratatui.rs/
- **crossterm**: https://docs.rs/crossterm/
- **tokio**: https://tokio.rs/
- **llama-cpp-2**: https://github.com/utilityai/llama-cpp-rs

### Example Projects

- **ratatui examples**: Clone ratatui repo, check `examples/` directory
- **Kaido AI Shell constitution**: `.specify/memory/constitution.md`
- **Feature spec**: `specs/006-tui-interface/spec.md`
- **Research findings**: `specs/006-tui-interface/research.md`
- **Data model**: `specs/006-tui-interface/data-model.md`

### Getting Help

1. Check research findings in `research.md` for solved problems
2. Review data model in `data-model.md` for state structure
3. Read feature spec in `spec.md` for requirements clarity
4. Consult constitution in `.specify/memory/constitution.md` for principles
5. Ask specific technical questions with context

---

## Next Steps After Implementation

Once TUI is working:

1. **Run full test suite**: `cargo test`
2. **Check for warnings**: `cargo build` should show zero warnings
3. **Manual test checklist**: Complete all items above
4. **Update this quickstart**: Document any issues encountered
5. **Ready for next feature**: TUI foundation complete, can build on top

---

## Summary

**Quick Start Command**:
```bash
cd /Users/gloomcheng/Workspace/RikaiDev/kaido-ai
git checkout 006-tui-interface
cargo add ratatui@0.27
cargo build
cargo run
```

**Key Files to Understand**:
- `research.md`: All technical decisions explained
- `data-model.md`: State structure and relationships
- `spec.md`: User requirements and acceptance criteria
- `plan.md`: Overall implementation strategy

**Development Order**:
1. Log silencing (quick win)
2. Basic TUI structure (skeleton)
3. Event loop (foundation)
4. Spinner animation (feedback)
5. Modal dialog (safety)

**Remember**: Manual process, zero warnings, no mock implementations, follow constitution!

