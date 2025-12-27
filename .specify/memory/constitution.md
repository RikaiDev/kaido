<!-- 
Sync Impact Report:
Version change: 1.5.0 → 1.5.1
Modified principles: VI. Professional Code Standards - Added mandatory individual inspection for unused code
Added sections: None
Removed sections: None
Templates requiring updates: None (clarification only)
Follow-up TODOs: None
Bump rationale: PATCH version bump - Clarification added to prevent batch deletion of unused code
-->

# Kaido AI Shell Constitution

## Core Principles

### I. MVP-First Development (NON-NEGOTIABLE)

Build the simplest version that works. Start with basic natural language to command translation. Add features only when user feedback demands them. Avoid premature optimization and complex abstractions. Focus on core value: making shell commands easier through AI.

### II. Testable by Design (NON-NEGOTIABLE)

Every component MUST be independently testable. Use dependency injection for external dependencies (AI models, file system, commands). Mock external services in tests. Each user story must have clear acceptance criteria and automated tests.

### III. Simple AI Integration (NON-NEGOTIABLE)

Start with a single AI model (local GGUF). No complex routing or hybrid strategies. Simple prompt → response → command execution flow. Add cloud models only when local models prove insufficient for user needs.

### IV. Basic Safety (NON-NEGOTIABLE)

Simple confirmation for destructive commands (rm, mv, etc.). Log all executed commands. No complex sandboxing or rollback mechanisms in MVP. Safety grows with user feedback.

### V. Shell Compatibility (NON-NEGOTIABLE)

Work as a drop-in replacement for bash/zsh. Maintain environment variables, working directory, and basic shell features. No complex state management in MVP.

### VI. Professional Code Standards (NON-NEGOTIABLE)

Maintain professional code appearance and readability. Emoji characters are strictly prohibited in all source code, configuration files, documentation, and user-facing content. Use standard ASCII characters only to ensure compatibility across all systems and platforms.

**Code Quality Requirements:**
- NEVER use `#[allow(dead_code)]` or similar suppression attributes to hide warnings
- NEVER use underscore prefix for variables (e.g., `_variable`, `_result`, `_guard`)
- All variables MUST have meaningful, descriptive names indicating their purpose
- RAII guards MUST use descriptive names (e.g., `terminal_guard`, `stderr_redirect`, `lock_holder`)
- All warnings MUST be resolved by proper code implementation or removal
- Unused code warnings REQUIRE individual inspection:
  - NEVER batch-delete unused code without checking functionality
  - For EACH unused item, determine if it serves project requirements
  - If feature is required, IMPLEMENT it properly
  - If feature is not required, THEN remove it
  - Manual one-by-one verification is mandatory
- Unused variables indicate incomplete implementation or unnecessary code
- All functions, structs, and methods MUST serve a purpose or be removed
- Code must compile with zero warnings

### VII. Real Implementation Requirement (NON-NEGOTIABLE)

NO MOCK IMPLEMENTATIONS ALLOWED. NO FAKE, USELESS, OR PRETEND IMPLEMENTATIONS. Every piece of code MUST serve a real-world need and provide actual functionality. 

**Implementation Standards:**
- NEVER implement mock functions that return hardcoded strings like "not implemented yet"
- NEVER create placeholder implementations that pretend to work but do nothing
- NEVER write code just to make it compile without providing real value
- If a feature cannot be fully implemented, DO NOT implement it at all
- Every implementation MUST solve a real user problem or enable real functionality
- Remove incomplete features rather than leaving mock implementations
- Only implement what users actually need and will use

### VIII. Manual Development Process (NON-NEGOTIABLE)

ALL development work MUST be performed manually and deliberately. Automated batch operations and script-based modifications are strictly prohibited to ensure quality and intentional changes.

**Process Requirements:**
- NEVER use shell scripts to test the application
- NEVER use Python scripts to modify code
- NEVER use sed, awk, or similar tools for batch code modifications
- NEVER use automated find-and-replace across multiple files
- Testing MUST be done manually, one test case at a time
- Code changes MUST be made individually and deliberately
- Each modification MUST be reviewed and understood before implementation
- Batch operations hide mistakes and prevent careful consideration
- Manual process ensures every change is intentional and correct

### IX. Complete Execution Requirement (NON-NEGOTIABLE)

NEVER adopt a "this is good enough" mindset. Work MAY pause due to token limits or technical constraints, but MUST always resume and continue forward progress. Settling for partial completion is strictly prohibited.

**Execution Standards:**
- NEVER suggest stopping with partial implementation
- NEVER provide "this should work" or "this is sufficient" feedback when work remains
- If token limits are reached, explicitly state "pausing due to token limit, will continue"
- ALWAYS identify remaining work items before any pause
- Progress is the only acceptable direction—there is no "good enough" compromise
- When warnings remain, implementation remains incomplete
- When methods are unused, implementation remains incomplete
- Only completion of all requirements satisfies this principle
- Each resume MUST pick up exactly where previous work stopped
- "Suggestions" or "options to consider" are NOT acceptable when concrete work remains

## Technology Constraints

**Language**: Rust (performance and safety)
**REPL**: rustyline (simple, proven)
**AI Models**: GGML/GGUF only (no cloud APIs in MVP)
**Command Execution**: std::process::Command
**Configuration**: Simple TOML file
**Logging**: Basic structured logging

## Development Workflow

**MVP Phases**: Basic REPL → Simple AI → Command Execution → Basic Safety → User Testing
**Testing**: Unit tests for each component, integration tests for user workflows
**Code Review**: Focus on simplicity and testability
**Documentation**: Keep it minimal and focused

## Governance

Keep it simple. Add complexity only when users request it. Every feature must solve a real user problem. Test everything. Use implementation plans for detailed guidance.

**Version**: 1.5.1 | **Ratified**: 2025-10-22 | **Last Amended**: 2025-10-24
