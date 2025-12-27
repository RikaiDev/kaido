# Quickstart: Shell Logging Improvement

**Feature**: Shell Logging Improvement  
**Date**: 2025-01-23

## Overview

This feature reduces verbose startup logging in Kaido AI Shell and adds builtin commands for logging level control, providing a cleaner, more professional user experience.

## Key Changes

### 1. Simplified Startup Output

**Before**:
```
[2025-10-23T01:08:15Z INFO  kaido] Starting Kaido AI Shell
[2025-10-23T01:08:15Z INFO  kaido] Configuration loaded: backend=Local
[2025-10-23T01:08:15Z INFO  kaido] AI manager initialized successfully
[2025-10-23T01:08:15Z INFO  kaido] Model status: Loading
[2025-10-23T01:08:15Z INFO  kaido] Model error status detected
[2025-10-23T01:08:15Z INFO  kaido] Performance metrics: 0 inferences, 0.00ms avg response time
[2025-10-23T01:08:15Z INFO  kaido] Task planner ready with 6 capabilities
[2025-10-23T01:08:15Z INFO  kaido] Testing model switching capability...
[2025-10-23T01:08:15Z INFO  kaido] Successfully switched to local model
[2025-10-23T01:08:15Z INFO  kaido] Planner capabilities updated: 6 capabilities available
[2025-10-23T01:08:15Z INFO  kaido] i18n manager initialized successfully
[2025-10-23T01:08:15Z INFO  kaido] Multilingual AI processor initialized successfully
[2025-10-23T01:08:15Z INFO  kaido::shell] Initializing Kaido AI Shell session
[2025-10-23T01:08:15Z INFO  kaido::shell] Session initialized successfully
```

**After**:
```
Welcome to Kaido AI Shell
Type your request in natural language
kaido> 
```

### 2. Builtin Logging Commands

**New Commands**:
- `set log-level quiet` - Minimal output (errors only)
- `set log-level normal` - Standard output (default)
- `set log-level verbose` - Detailed operational info
- `set log-level debug` - Full debugging information
- `status log-level` - Show current logging level
- `unset log-level` - Reset to default level

**Usage Examples**:
```bash
kaido> set log-level verbose
Logging level set to: verbose

kaido> status log-level
Current logging level: verbose

kaido> unset log-level
Logging level reset to: normal
```

### 3. Configuration Persistence

**Location**: `~/.config/kaido/config.toml`

**Example Configuration**:
```toml
[logging]
level = "normal"
startup_verbose = false
welcome_message = "Welcome to Kaido AI Shell"

[session]
auto_save_config = true
```

## Implementation Details

### Files Modified

- `src/config.rs` - Add logging configuration support
- `src/shell/repl.rs` - Integrate builtin commands
- `src/shell/state.rs` - Add session logging state
- `config/default.toml` - Add default logging settings

### New Dependencies

- None (uses existing `tracing` and `toml` crates)

### Testing Strategy

- Unit tests for logging level changes
- Integration tests for configuration persistence
- Manual testing for startup output verification

## Migration Guide

### For Users

1. **No action required** - Changes are backward compatible
2. **Optional**: Create `~/.config/kaido/config.toml` for custom settings
3. **New feature**: Use `set log-level` commands for debugging

### For Developers

1. **Update tests** - Add logging level test cases
2. **Review startup code** - Ensure minimal output by default
3. **Add builtin commands** - Implement `set`/`unset`/`status` commands

## Success Metrics

- ✅ Startup output reduced from 15+ lines to 2 lines
- ✅ Logging level changes take effect immediately
- ✅ Configuration persists across sessions
- ✅ Zero compilation warnings maintained
- ✅ All existing functionality preserved

