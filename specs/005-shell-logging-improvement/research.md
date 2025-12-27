# Research: Shell Logging Improvement

**Feature**: Shell Logging Improvement  
**Date**: 2025-01-23  
**Phase**: 0 - Research & Analysis

## Research Tasks Completed

### 1. XDG Base Directory Specification Analysis

**Task**: Research XDG Base Directory for configuration storage

**Decision**: Use `~/.config/kaido/config.toml` for configuration storage

**Rationale**: 
- Follows XDG Base Directory Specification standard
- `XDG_CONFIG_HOME` defaults to `~/.config` if not set
- Widely adopted by modern Linux/Unix applications
- Provides consistent user experience across platforms

**Alternatives considered**:
- `~/.kaido/config.toml` - Non-standard, clutters home directory
- `~/.config/kaido.toml` - Less organized for future expansion
- Environment variables only - Not persistent across sessions

### 2. Modern Shell Welcome Message Patterns

**Task**: Research Fish Shell, Bash, and Zsh startup message patterns

**Decision**: Follow Fish Shell's minimal approach (2 lines max)

**Rationale**:
- Fish Shell: "Welcome to fish, the friendly interactive shell" + "Type help for instructions"
- Bash/Zsh: Typically no welcome message, rely on prompt
- Minimal approach reduces cognitive load
- Clear guidance without overwhelming new users

**Alternatives considered**:
- Detailed welcome with feature list - Too verbose for shell
- Version number display - Not essential for daily use
- Status information - Belongs in prompt, not startup

### 3. Shell Builtin Command Patterns

**Task**: Research shell builtin commands for logging control

**Decision**: Implement builtin commands similar to bash `set -x` pattern

**Rationale**:
- Bash: `set -x` (debug), `set -v` (verbose), `set -e` (exit on error)
- Fish: `set -l` (local), `set -g` (global), `set -U` (universal)
- Builtin commands are immediate and session-scoped
- Standard pattern across shell implementations
- No external configuration files needed for runtime changes

**Alternatives considered**:
- Command-line flags only - Not persistent across commands
- Configuration file only - Not immediate for debugging
- Environment variables - Less discoverable than builtin commands

### 4. Natural Language Shell Interaction Analysis

**Task**: Research help system requirements for natural language shells

**Decision**: No traditional help system needed

**Rationale**:
- Natural language shells allow direct conversation ("How do I...", "Help me...")
- Traditional help commands are redundant
- AI can provide contextual assistance
- Reduces complexity and maintenance burden

**Alternatives considered**:
- Traditional help command - Contradicts natural language paradigm
- Command reference - Not needed for conversational interface
- Tutorial system - Can be handled through conversation

## Technical Decisions Summary

1. **Configuration Storage**: XDG Base Directory (`~/.config/kaido/config.toml`)
2. **Startup Messages**: Minimal Fish-style welcome (2 lines max)
3. **Logging Control**: Builtin commands following bash `set` patterns
4. **Help System**: None - rely on natural language interaction
5. **Implementation**: Integrate into existing shell and config modules

## Research Validation

All technical decisions align with:
- Constitution principles (MVP-first, simple integration)
- Modern shell standards (XDG, builtin commands)
- User experience goals (minimal startup, immediate control)
- Technical constraints (existing Rust codebase, zero warnings)

