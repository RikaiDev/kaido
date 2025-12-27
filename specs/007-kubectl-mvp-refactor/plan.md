# Implementation Plan: Kubectl-Only MVP (60-Day Reality Check)

**Branch**: `007-kubectl-mvp-refactor` | **Date**: 2025-10-25 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/007-kubectl-mvp-refactor/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Refactor Kaido AI Shell to focus exclusively on kubectl natural language interface with risk-based safety controls. Remove all non-kubectl tool adapters (Docker, Git, Terraform, etc.) to achieve 60-day MVP. Core value: risk-graded confirmation system (LOW/MEDIUM/HIGH) for kubectl operations with audit logging and AI translation confidence tracking.

**Key Changes**:
- Remove: Tool registry system, Docker/Git adapters, multi-tool detection
- Add: Kubectl-specific risk classification, typed confirmation for HIGH risk, SQLite audit log with TUI queries
- Refine: OpenAI GPT-4 integration for kubectl translation, confidence score display (<70% threshold)

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: 
- rustyline 14.0 (REPL)
- tokio 1.0 (async runtime)
- reqwest 0.11 (OpenAI API client)
- rusqlite 0.31 (audit log storage)
- ratatui 0.27 (TUI for safety modals and history display)
- serde/serde_json 1.0 (JSON serialization)

**Storage**: SQLite database for audit logs (location: `~/.kaido/audit.db`)
**Testing**: cargo test (unit tests for risk classification, integration tests for kubectl execution flow)
**Target Platform**: Linux/macOS with kubectl installed and configured kubeconfig
**Project Type**: Single project (CLI tool)
**Performance Goals**: <5 seconds for AI translation + kubectl execution (95th percentile)
**Constraints**: 
- Must work with existing kubectl installations (no cluster-side components)
- Kubeconfig parsing using standard Rust YAML libraries
- OpenAI API calls must include timeout (10 seconds) and retry (1 retry)

**Scale/Scope**: 
- 5 beta users during 60-day trial
- Support 20 common kubectl operations
- Audit log retention: 90 days (configurable)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

✅ **MVP-First Development**: Removing multi-tool support to focus on single kubectl interface - simplest version that works
✅ **Testable by Design**: Risk classifier, kubectl command builder, audit logger are independently testable components
✅ **Privacy-First AI Integration**: Local GGUF models (llama.cpp) as primary translation method, OpenAI GPT-4 API as fallback (enterprise requirement)
✅ **Basic Safety**: Risk-based confirmation (HIGH=typed, MEDIUM=yes/no, LOW=none) with full command logging
✅ **Shell Compatibility**: Not applicable - tool is kubectl-specific, not general shell replacement
⚠️ **Professional Code Standards**: MUST remove all existing dead code from removed features (tool registry, Docker/Git adapters)
✅ **Real Implementation Requirement**: All risk classification rules must have real kubectl command analysis, no mock patterns

**Constitution Compliance Notes**:
- Shell Compatibility exception justified: This is kubectl assistant, not general shell
- Will audit codebase for dead code from removed features before implementation begins

## Project Structure

### Documentation (this feature)

```text
specs/007-kubectl-mvp-refactor/
├── plan.md              # This file
├── research.md          # Phase 0: kubectl context detection, OpenAI prompt engineering
├── data-model.md        # Phase 1: Risk levels, audit log schema, kubeconfig parsing
├── quickstart.md        # Phase 1: Installation and first kubectl command
├── contracts/           # Phase 1: OpenAI API contract, audit log SQL schema
└── tasks.md             # Phase 2: NOT created by this command
```

### Source Code (repository root)

```text
src/
├── kubectl/                    # NEW: Kubectl-specific functionality
│   ├── mod.rs
│   ├── context.rs              # Parse kubeconfig, detect current context
│   ├── translator.rs           # Natural language → kubectl command via OpenAI
│   ├── risk_classifier.rs      # Classify commands into LOW/MEDIUM/HIGH
│   └── executor.rs             # Execute kubectl commands
├── audit/                      # NEW: Audit logging
│   ├── mod.rs
│   ├── logger.rs               # Write to SQLite
│   ├── schema.sql              # Database schema
│   └── query.rs                # TUI query interface (today, last week, production)
├── ui/
│   ├── modal.rs                # EXISTING: Reuse for risk-based confirmation
│   ├── app.rs                  # UPDATE: Remove multi-tool UI, add history view
│   └── confidence.rs           # NEW: Display confidence warnings (<70%)
├── ai/
│   ├── mod.rs                  # UPDATE: Remove local GGUF, keep OpenAI only
│   └── prompt_builder.rs       # UPDATE: Kubectl-specific prompts
├── config.rs                   # UPDATE: Remove tool registry config
├── shell/
│   ├── repl.rs                 # UPDATE: Kubectl-only REPL loop
│   └── executor.rs             # KEEP: Generic command execution
├── safety/
│   ├── mod.rs                  # REMOVE: Old allowlist system
│   └── confirmation.rs         # NEW: Risk-based confirmation logic
└── main.rs                     # UPDATE: Simplified main for kubectl-only mode

# REMOVE THESE DIRECTORIES
src/tools/                      # Tool registry, Docker/Git adapters - DELETE
src/agent/                      # Multi-agent orchestration - DELETE
src/memory/                     # Complex memory system - DELETE (use audit log instead)

tests/
├── unit/
│   ├── kubectl/                # NEW: Risk classifier, context parser tests
│   ├── audit/                  # NEW: SQLite logger tests
│   └── ai/                     # UPDATE: OpenAI mock tests
└── integration/
    └── kubectl_flow_test.rs    # NEW: End-to-end kubectl translation + execution
```

**Structure Decision**: Single project structure maintained. Removed complexity from multi-tool architecture (src/tools/, src/agent/). New kubectl/ and audit/ modules encapsulate core MVP functionality. Existing ui/ and shell/ modules reused with simplifications.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | Constitution fully satisfied | No violations requiring justification |

**Justification for Shell Compatibility Exception**: 
The constitution states "Work as a drop-in replacement for bash/zsh" but this feature explicitly focuses on kubectl operations only. This is acceptable because:
1. MVP principle takes precedence - simplest version that works is kubectl-only
2. User feedback will determine if general shell features are needed
3. Kubectl context is sufficient to deliver core value (natural language kubectl)
