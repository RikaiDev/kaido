# Data Model: Shell Logging Improvement

**Feature**: Shell Logging Improvement  
**Date**: 2025-01-23  
**Phase**: 1 - Design

## Entities

### LoggingConfiguration

**Purpose**: Stores user preferences for log verbosity and output format

**Fields**:
- `log_level: LogLevel` - Current logging level (Quiet, Normal, Verbose, Debug)
- `startup_verbose: bool` - Whether to show verbose startup messages
- `welcome_message: Option<String>` - Custom welcome message (optional)
- `last_updated: DateTime<Utc>` - Timestamp of last configuration change

**Validation Rules**:
- `log_level` must be one of: Quiet, Normal, Verbose, Debug
- `welcome_message` must be <= 200 characters if provided
- `last_updated` must be current timestamp on save

**State Transitions**:
- Default → User Modified (on first configuration change)
- User Modified → Default (on reset to defaults)

### UserSession

**Purpose**: Current shell session state including logging preferences

**Fields**:
- `session_id: Uuid` - Unique session identifier
- `log_level: LogLevel` - Current session logging level
- `startup_time: DateTime<Utc>` - Session start timestamp
- `config_loaded: bool` - Whether configuration was successfully loaded
- `builtin_commands: Vec<String>` - Available builtin commands

**Validation Rules**:
- `session_id` must be unique per session
- `log_level` must be valid LogLevel enum value
- `builtin_commands` must include at least: "set", "unset", "status"

**State Transitions**:
- Initializing → Active (on successful startup)
- Active → Terminating (on shell exit)
- Active → Error (on configuration load failure)

## Enums

### LogLevel

**Purpose**: Defines available logging verbosity levels

**Values**:
- `Quiet` - Only essential messages (errors, warnings)
- `Normal` - Standard operational messages
- `Verbose` - Detailed operational information
- `Debug` - Full debugging information including traces

**Validation**:
- Must be ordered from least to most verbose
- Each level includes all messages from previous levels

## Relationships

- `UserSession` has one `LoggingConfiguration` (loaded from file)
- `LoggingConfiguration` can be shared across multiple `UserSession` instances
- `UserSession` tracks runtime state independent of persistent configuration

## Data Flow

1. **Startup**: Load `LoggingConfiguration` from `~/.config/kaido/config.toml`
2. **Session Creation**: Create `UserSession` with loaded configuration
3. **Runtime Changes**: Update `UserSession.log_level` via builtin commands
4. **Persistence**: Save `LoggingConfiguration` changes to file
5. **Shutdown**: Clean up session state, persist any configuration changes

