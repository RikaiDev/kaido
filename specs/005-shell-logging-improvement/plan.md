# Implementation Plan: Shell Logging Improvement

**Branch**: `005-shell-logging-improvement` | **Date**: 2025-01-23 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-shell-logging-improvement/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Reduce verbose startup logging in Kaido AI Shell to provide a cleaner, more professional user experience. Implement builtin commands for logging level control similar to bash's `set -x`, while maintaining debugging capabilities. Store configuration in ~/.config/kaido/config.toml following XDG standards.

## Technical Context

**Language/Version**: Rust 1.75+ (existing project)  
**Primary Dependencies**: tokio, tracing, rustyline, toml (existing)  
**Storage**: ~/.config/kaido/config.toml (XDG Base Directory)  
**Testing**: cargo test (existing framework)  
**Target Platform**: Linux/macOS (existing shell)  
**Project Type**: CLI application (existing structure)  
**Performance Goals**: <3 seconds for logging level changes, <2 lines startup output  
**Constraints**: Must maintain shell compatibility, zero warnings compilation  
**Scale/Scope**: Single user shell session, minimal configuration

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**MVP-First Development**: ✅ Feature reduces complexity by simplifying startup output  
**Testable by Design**: ✅ Logging levels can be tested independently, configuration changes are verifiable  
**Simple AI Integration**: ✅ No impact on AI integration, only affects logging output  
**Basic Safety**: ✅ Maintains command logging for safety, adds builtin controls  
**Shell Compatibility**: ✅ Preserves shell functionality, adds standard builtin commands

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

### Source Code (repository root)

```text
src/
├── config.rs              # Configuration management (existing)
├── shell/
│   ├── mod.rs             # Shell module (existing)
│   ├── repl.rs            # REPL implementation (existing)
│   ├── executor.rs        # Command execution (existing)
│   ├── prompt.rs          # Prompt management (existing)
│   └── state.rs           # Shell state (existing)
├── safety/
│   ├── mod.rs             # Safety module (existing)
│   ├── detector.rs         # Command detection (existing)
│   └── confirmation.rs     # User confirmation (existing)
├── ai/
│   ├── mod.rs             # AI module (existing)
│   └── model.rs           # Model management (existing)
├── i18n/
│   └── mod.rs             # Internationalization (existing)
└── utils/
    └── mod.rs             # Utilities (existing)

tests/
├── integration/
│   └── shell_tests.rs     # Shell integration tests
└── unit/
    ├── config_tests.rs    # Configuration tests
    └── logging_tests.rs   # Logging level tests

config/
└── default.toml           # Default configuration (existing)
```

**Structure Decision**: Single project structure with modular organization. Logging improvements will be integrated into existing shell and config modules.

## Phase 0 & 1 Completion

### Phase 0: Research ✅
- **research.md**: Completed with XDG standards, shell patterns, and builtin command analysis
- **All NEEDS CLARIFICATION resolved**: Technical context fully specified
- **Research validation**: All decisions align with constitution principles

### Phase 1: Design ✅
- **data-model.md**: LoggingConfiguration and UserSession entities defined
- **contracts/api.md**: Builtin commands API and configuration file schema
- **quickstart.md**: Implementation overview and migration guide
- **Agent context updated**: Cursor IDE context file updated with new technologies

### Constitution Check (Post-Design) ✅
**MVP-First Development**: ✅ Feature reduces complexity by simplifying startup output  
**Testable by Design**: ✅ Logging levels can be tested independently, configuration changes are verifiable  
**Simple AI Integration**: ✅ No impact on AI integration, only affects logging output  
**Basic Safety**: ✅ Maintains command logging for safety, adds builtin controls  
**Shell Compatibility**: ✅ Preserves shell functionality, adds standard builtin commands

## Next Steps

Ready for `/speckit.tasks` to generate implementation tasks.
