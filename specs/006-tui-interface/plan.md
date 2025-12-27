# Implementation Plan: Professional TUI Interface for Kaido AI Shell

**Branch**: `006-tui-interface` | **Date**: 2025-10-24 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/006-tui-interface/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Transform Kaido AI Shell from a basic text REPL into a professional TUI application with split-panel layout (70/30 left/right), animated AI thinking indicator, toggleable detailed analysis view, modal safety confirmations, and completely silenced llama.cpp verbose logging. Uses ratatui framework for rendering, crossterm for terminal control, and integrates with existing AIManager and SafeExecutor components.

## Technical Context

**Language/Version**: Rust 1.75+ (already established in project)
**Primary Dependencies**: 
- ratatui 0.27 (TUI framework - NEW)
- crossterm 0.27 (already present for terminal control)
- tokio 1.0 with full features (already present for async runtime)
- llama-cpp-2 from git (already present for LLM inference)
- serde_json 1.0 (already present for JSON parsing)

**Storage**: File-based allowlist in user config directory (~/.config/kaido/allowlist.txt or similar)
**Testing**: cargo test with manual TUI testing (per constitution's Manual Development Process principle)
**Target Platform**: Terminal emulators on macOS/Linux/Windows with Unicode support (80+ column width minimum)
**Project Type**: Single project (terminal application)
**Performance Goals**: 
- <100ms UI response to key press
- ≥10 FPS spinner animation
- <50ms frame render time
- Immediate (<100ms) thinking animation start

**Constraints**: 
- No external dependencies beyond Rust crates
- Must work in standard terminal emulators
- Must maintain existing AI and safety functionality
- Zero compilation warnings (per constitution)
- No mock implementations (per constitution)

**Scale/Scope**: 
- 5 new source files (ui module with 4 sub-modules)
- 4 modified existing files (main.rs, repl.rs, ai/mod.rs, safety/executor.rs)
- ~800-1000 lines of new TUI code
- Single-user local application

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**MVP-First Development**: ✅ PASS - Split-panel layout is the simplest viable TUI approach. Spinner animation is basic Unicode cycling. Modal is standard blocking pattern. No over-engineering detected.

**Testable by Design**: ✅ PASS - Each UI component (app state, layout, spinner, modal) is independently testable. Event handling separated from rendering. State management isolated in KaidoApp struct.

**Simple AI Integration**: ✅ PASS - Feature integrates with existing AIManager without modification to AI logic. No new AI routing or complexity added.

**Basic Safety**: ✅ PASS - Feature enhances existing SafeExecutor with visual modal confirmations. Maintains simple allowlist pattern. No complex sandboxing.

**Shell Compatibility**: ✅ PASS - TUI mode is opt-in enhancement. Core shell functionality (command execution, history, environment) unchanged. Can fall back to basic REPL if TUI fails.

**Professional Code Standards**: ✅ PASS - Plan includes warning fixes (unused imports/methods). ratatui is production-quality framework. Unicode spinners are standard professional UX pattern.

**Real Implementation Requirement**: ✅ PASS - All TUI components are real ratatui widgets with actual rendering logic. Spinner uses real animation loop. Modal blocks with real event capture. llama.cpp silencing uses real environment variable mechanism.

**Manual Development Process**: ✅ PASS - Plan specifies manual testing only. No test scripts. Each file modification done individually through code tools.

## Project Structure

### Documentation (this feature)

```text
specs/006-tui-interface/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command) - N/A for TUI (no API contracts)
├── spec.md              # Feature specification (already exists)
└── checklists/
    └── requirements.md  # Specification quality checklist (already exists)
```

### Source Code (repository root)

```text
src/
├── main.rs                    # MODIFIED: Add ui module, set LLAMA_LOG_LEVEL
├── ai/
│   ├── mod.rs                 # MODIFIED: Add LLAMA_LOG_LEVEL in run_llama_cpp_inference
│   └── parser.rs              # Existing: CommandSequence/Command types used by TUI
├── config.rs                  # Existing: Config loading (extended for allowlist path)
├── shell/
│   ├── mod.rs                 # Existing: Shell module exports
│   ├── repl.rs                # MODIFIED: Complete rewrite to use TUI event loop
│   ├── executor.rs            # Existing: CommandExecutor (unchanged)
│   ├── prompt.rs              # Existing: Prompt utilities (may be deprecated)
│   └── state.rs               # Existing: Shell state (unchanged)
├── safety/
│   ├── mod.rs                 # Existing: Safety module exports
│   ├── executor.rs            # MODIFIED: Add modal dialog support, refactor for async
│   └── allowlist.rs           # Existing: Allowlist management
├── ui/                        # NEW MODULE
│   ├── mod.rs                 # NEW: UI module exports (app, layout, spinner, modal)
│   ├── app.rs                 # NEW: KaidoApp state struct and methods
│   ├── layout.rs              # NEW: Split-panel layout function (70/30)
│   ├── spinner.rs             # NEW: Spinner animation frames and helpers
│   └── modal.rs               # NEW: ModalDialog struct and rendering
└── utils/
    └── mod.rs                 # Existing: Error types (unchanged)

tests/
├── integration/
│   └── tui_smoke_test.rs      # NEW: Basic TUI startup/teardown test
└── unit/
    └── ui/                    # NEW: Unit tests for UI components
        ├── app_test.rs        # NEW: Test KaidoApp state transitions
        ├── layout_test.rs     # NEW: Test split calculations
        └── spinner_test.rs    # NEW: Test spinner frame cycling

Cargo.toml                     # MODIFIED: Add ratatui = "0.27"
```

**Structure Decision**: Single project structure (Option 1) is appropriate. This is a terminal application with no backend/frontend split or platform-specific code. The new `ui/` module sits alongside existing modules (`ai/`, `shell/`, `safety/`) at the same level. Tests organized by integration vs unit, with new `ui/` subdirectory in unit tests mirroring source structure.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations detected. All constitution principles satisfied.

## Phase 0: Research Tasks

### Research Task 1: ratatui Best Practices for Event-Driven TUI

**Question**: What is the recommended pattern for integrating async operations (AI inference) with ratatui's synchronous rendering loop?

**Research Areas**:
- Event polling strategy (blocking vs non-blocking)
- Async runtime integration (tokio with ratatui)
- State updates during long-running operations
- Spinner animation timing with event loop

**Expected Deliverable**: Code pattern showing how to maintain responsive UI (spinner animation) while awaiting AI response without blocking key events.

### Research Task 2: llama.cpp Log Control Mechanisms

**Question**: Does llama-cpp-2 Rust bindings respect LLAMA_LOG_LEVEL environment variable, and if not, what alternative silencing mechanisms exist?

**Research Areas**:
- llama-cpp-2 documentation for log control
- llama.cpp C++ library log level configuration
- Environment variables vs programmatic API
- Fallback options if LLAMA_LOG_LEVEL ineffective

**Expected Deliverable**: Concrete code snippet confirming log silencing method that works with llama-cpp-2 from git.

### Research Task 3: Allowlist Persistence Strategy

**Question**: What is the simplest file format and storage location for persisting the dangerous command allowlist across sessions?

**Research Areas**:
- XDG Base Directory Specification for config files (already used in project)
- File formats: plain text (one command per line) vs JSON vs TOML
- Atomic file writes to prevent corruption
- Error handling for missing/corrupted allowlist files

**Expected Deliverable**: Decision on file format and path, plus code example for load/save operations.

### Research Task 4: Modal Dialog Event Handling

**Question**: How should the application handle modal dialog interactions in an async event loop without blocking non-modal key events?

**Research Areas**:
- State machine pattern for modal vs normal mode
- Event filtering based on application state
- Modal lifetime management (when to create/destroy)
- Integration with SafeExecutor's async execution flow

**Expected Deliverable**: Event loop pseudocode showing state transitions between normal input mode, modal active mode, and command execution mode.

### Research Task 5: Terminal Raw Mode Cleanup

**Question**: What is the robust pattern for ensuring terminal is restored to normal mode on panic or unexpected exit?

**Research Areas**:
- crossterm's disable_raw_mode() placement
- Rust panic hooks and cleanup
- RAII pattern for terminal mode management
- Signal handling (Ctrl+C) with graceful cleanup

**Expected Deliverable**: Code pattern with proper error handling ensuring terminal always restored even on panic/interrupt.

## Phase 1: Design Artifacts (COMPLETED)

**Status**: ✅ All Phase 1 artifacts generated

### Generated Documents

- **research.md**: ✅ All 5 research tasks resolved with concrete decisions:
  1. Async + TUI integration (non-blocking event loop pattern)
  2. llama.cpp log silencing (`LLAMA_LOG_LEVEL=0`)
  3. Allowlist persistence (plain text file format)
  4. Modal dialog event handling (enum state machine)
  5. Terminal cleanup (RAII TerminalGuard pattern)

- **data-model.md**: ✅ Complete state model documentation including:
  - KaidoApp (application state container)
  - ModalDialog (safety confirmation UI)
  - AppState (state machine enum)
  - Allowlist (persistent command list)
  - CommandSequence (AI output structure)
  - Data flow diagrams and validation rules

- **quickstart.md**: ✅ Developer guide with:
  - Prerequisites and dependencies
  - Project setup instructions
  - Phase-by-phase development workflow
  - Testing guide and debugging tips
  - Code style guidelines per constitution
  - Common pitfalls and solutions

- **contracts/**: N/A (no API contracts for TUI application - this is a terminal UI, not a service)

### Agent Context Update

- ✅ Updated `.cursor/rules/specify-rules.mdc` with:
  - Language: Rust 1.75+
  - Database: File-based allowlist (~/.config/kaido/allowlist.txt)
  - Project type: Single project (terminal application)

### Re-evaluation: Constitution Check (Post-Design)

All principles remain satisfied after Phase 1 design:

✅ **MVP-First Development**: Design uses simplest patterns (plain text files, basic state machine)
✅ **Testable by Design**: All components independently testable (unit tests planned for each module)
✅ **Simple AI Integration**: Zero changes to AI logic, only UI layer added
✅ **Basic Safety**: Modal dialog is simple blocking pattern, allowlist is flat file
✅ **Shell Compatibility**: TUI is opt-in enhancement, core shell unchanged
✅ **Professional Code Standards**: Design includes warning fixes, uses production frameworks
✅ **Real Implementation Requirement**: All components have concrete implementations, no mocks
✅ **Manual Development Process**: Quickstart emphasizes manual testing, no automated scripts

**Gate Status**: ✅ PASS - Ready for Phase 2 (Task Generation via `/speckit.tasks`)
