# API Contracts: Shell Logging Improvement

**Feature**: Shell Logging Improvement  
**Date**: 2025-01-23  
**Type**: Builtin Commands Interface

## Builtin Commands API

### set

**Purpose**: Set logging level or other shell options

**Syntax**: `set <option> <value>`

**Options**:
- `log-level <level>` - Set logging verbosity level
- `startup-verbose <true|false>` - Control startup message verbosity

**Examples**:
```bash
set log-level verbose
set log-level quiet
set startup-verbose false
```

**Response Format**:
```
Logging level set to: verbose
```

**Error Cases**:
- Invalid log level: `Error: Invalid log level 'invalid'. Valid levels: quiet, normal, verbose, debug`
- Missing value: `Error: Missing value for option 'log-level'`

### unset

**Purpose**: Reset options to default values

**Syntax**: `unset <option>`

**Options**:
- `log-level` - Reset to normal level
- `startup-verbose` - Reset to default startup behavior

**Examples**:
```bash
unset log-level
unset startup-verbose
```

**Response Format**:
```
Logging level reset to: normal
```

**Error Cases**:
- Invalid option: `Error: Unknown option 'invalid-option'`

### status

**Purpose**: Display current shell state and configuration

**Syntax**: `status [option]`

**Options**:
- `log-level` - Show current logging level
- `config` - Show configuration file location
- (no option) - Show all status information

**Examples**:
```bash
status log-level
status config
status
```

**Response Format**:
```
Current logging level: normal
Configuration file: ~/.config/kaido/config.toml
Session ID: 123e4567-e89b-12d3-a456-426614174000
```

## Configuration File API

### File Format: TOML

**Location**: `~/.config/kaido/config.toml`

**Schema**:
```toml
[logging]
level = "normal"                    # quiet, normal, verbose, debug
startup_verbose = false             # boolean
welcome_message = "Welcome to Kaido AI Shell"  # optional string

[session]
auto_save_config = true            # boolean
```

### Configuration Loading

**Priority Order**:
1. Command-line arguments (highest priority)
2. Configuration file (`~/.config/kaido/config.toml`)
3. Default values (lowest priority)

**Error Handling**:
- Invalid TOML syntax: Log warning, use defaults
- Missing file: Create with defaults
- Permission errors: Log error, use defaults

## Internal APIs

### LoggingConfiguration

```rust
pub struct LoggingConfiguration {
    pub log_level: LogLevel,
    pub startup_verbose: bool,
    pub welcome_message: Option<String>,
    pub last_updated: DateTime<Utc>,
}

impl LoggingConfiguration {
    pub fn load() -> Result<Self, ConfigError>;
    pub fn save(&self) -> Result<(), ConfigError>;
    pub fn set_log_level(&mut self, level: LogLevel);
    pub fn reset_to_defaults(&mut self);
}
```

### UserSession

```rust
pub struct UserSession {
    pub session_id: Uuid,
    pub log_level: LogLevel,
    pub startup_time: DateTime<Utc>,
    pub config_loaded: bool,
    pub builtin_commands: Vec<String>,
}

impl UserSession {
    pub fn new() -> Self;
    pub fn update_log_level(&mut self, level: LogLevel);
    pub fn get_status(&self) -> SessionStatus;
}
```

## Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| CONFIG_LOAD_ERROR | Failed to load configuration file | Use defaults, log warning |
| CONFIG_SAVE_ERROR | Failed to save configuration file | Log error, continue operation |
| INVALID_LOG_LEVEL | Invalid logging level specified | Show valid options |
| PERMISSION_ERROR | Insufficient permissions for config file | Use defaults, log error |
| PARSE_ERROR | Invalid TOML syntax in config file | Use defaults, log warning |

