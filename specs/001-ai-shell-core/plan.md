# Implementation Plan: Kaido AI Shell Core

**Branch**: `001-ai-shell-core` | **Date**: 2025-10-22 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-ai-shell-core/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Build an AI-powered shell that accepts natural language input and converts it to CLI commands, while maintaining full compatibility with traditional shell operations. Core value: transform CLI interaction from command memorization to natural language task description.

## Technical Context

**Language/Version**: Rust 1.75+ (performance and memory safety for CLI tool)  
**Primary Dependencies**: rustyline (REPL), candle-core (AI inference), tokio (async runtime), clap (CLI parsing)  
**Storage**: Local file system for configuration (TOML), command history, and model files  
**Testing**: cargo test with unit tests for each component, integration tests for user workflows  
**Target Platform**: Cross-platform CLI tool (Linux, macOS, Windows)  
**Project Type**: Single Rust binary with modular architecture  
**Performance Goals**: <3 second AI response time, <100ms command execution overhead  
**Constraints**: Offline-capable (local AI), <200MB memory footprint, zero-configuration setup  
**Scale/Scope**: Single-user desktop tool, handles typical CLI workflows (file operations, git, package management)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**MVP-First Development**:  PASS - Feature focuses on core natural language to command translation without complex abstractions  
**Testable by Design**:  PASS - Each component (REPL, AI inference, command execution) can be independently tested with dependency injection  
**Simple AI Integration**:  PASS - Single local GGUF model approach, no complex routing or hybrid strategies  
**Basic Safety**:  PASS - Simple confirmation for destructive commands, comprehensive command logging  
**Shell Compatibility**:  PASS - Drop-in replacement maintaining environment variables, working directory, and shell features

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

## Project Structure

### Documentation (this feature)

```text
specs/001-ai-shell-core/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── main.rs              # CLI entry point and argument parsing
├── config.rs            # Configuration management (TOML)
├── shell/               # Shell REPL and command execution
│   ├── mod.rs
│   ├── repl.rs          # rustyline-based REPL interface
│   ├── executor.rs      # Command execution and monitoring
│   └── state.rs         # Session state management
├── ai/                  # AI inference and natural language processing
│   ├── mod.rs
│   ├── model.rs         # GGUF model loading and inference
│   ├── planner.rs       # Task planning and command generation
│   └── prompts.rs       # Prompt templates for different scenarios
├── safety/              # Safety checks and dangerous command detection
│   ├── mod.rs
│   ├── detector.rs      # Dangerous command detection
│   └── confirmation.rs  # User confirmation handling
└── utils/               # Shared utilities
    ├── mod.rs
    ├── logging.rs       # Structured logging
    └── errors.rs        # Error handling and types

tests/
├── integration/         # End-to-end user workflow tests
│   ├── natural_language.rs
│   ├── shell_compatibility.rs
│   └── error_handling.rs
├── unit/               # Component-level tests
│   ├── ai/
│   ├── shell/
│   └── safety/
└── fixtures/           # Test data and mock models
    ├── sample_commands.txt
    └── test_config.toml

models/                 # GGUF model files (gitignored)
├── phi3-mini.gguf
└── README.md

config/                 # Default configuration files
└── default.toml
```

**Structure Decision**: Single Rust binary with modular architecture. Each major component (shell, AI, safety) is in its own module with clear interfaces. This supports MVP development while maintaining testability and future extensibility.

## Phase Completion Status

### Phase 0: Research  COMPLETE
- **research.md**: Technology decisions documented
- **Key Decisions**: Rust + candle-core + rustyline + tokio
- **Architecture**: Single binary with modular design
- **No clarifications needed**: All technical context resolved

### Phase 1: Design & Contracts  COMPLETE
- **data-model.md**: Core entities and relationships defined
- **contracts/api-contracts.md**: Internal API interfaces specified
- **quickstart.md**: User onboarding guide created
- **Agent context updated**: Cursor IDE context file created

### Phase 2: Task Planning ⏳ PENDING
- **tasks.md**: Implementation tasks will be generated by `/speckit.tasks`

## Generated Artifacts

1. **research.md**: Technical decisions and rationale
2. **data-model.md**: Core data structures and relationships
3. **contracts/api-contracts.md**: Internal API contracts and interfaces
4. **quickstart.md**: User quickstart guide
5. **Agent context**: Cursor IDE context file updated

## Constitution Check (Post-Design)

*Re-evaluated after Phase 1 design completion*

**MVP-First Development**:  PASS - Design maintains simple, focused approach  
**Testable by Design**:  PASS - All interfaces support dependency injection and mocking  
**Simple AI Integration**:  PASS - Single local GGUF model approach maintained  
**Basic Safety**:  PASS - Safety rules and confirmation system designed  
**Shell Compatibility**:  PASS - Drop-in replacement design preserved

## Complexity Tracking

> **No violations detected - all constitution requirements met**
