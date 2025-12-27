# Configuration API Contract

**Feature**: User Setup Guide  
**Date**: 2025-10-22  
**Version**: 1.0

## Configuration Management API

### Load Configuration
**Endpoint**: `Config::load() -> Result<Config, ConfigError>`

**Description**: Load configuration from default or specified file path

**Parameters**: None (uses default paths)

**Returns**: 
- `Ok(Config)` - Configuration loaded successfully
- `Err(ConfigError)` - Configuration file missing, invalid, or inaccessible

**Error Types**:
- `ConfigNotFound` - Configuration file does not exist
- `ConfigInvalid` - Configuration file contains invalid data
- `ConfigPermissionDenied` - Insufficient permissions to read file

### Validate Configuration
**Endpoint**: `Config::validate() -> Result<(), ValidationError>`

**Description**: Validate all configuration settings

**Parameters**: None

**Returns**:
- `Ok(())` - Configuration is valid
- `Err(ValidationError)` - Configuration contains invalid values

**Error Types**:
- `InvalidApiKey` - API key format is invalid
- `InvalidUrl` - API URL is malformed
- `InvalidTimeout` - Timeout value out of range
- `MissingRequiredField` - Required configuration field is missing

### Save Configuration
**Endpoint**: `Config::save(path: Option<PathBuf>) -> Result<(), ConfigError>`

**Description**: Save configuration to file

**Parameters**:
- `path`: Optional path to save to (uses default if None)

**Returns**:
- `Ok(())` - Configuration saved successfully
- `Err(ConfigError)` - Failed to save configuration

## Cloud API Integration Contract

### Validate API Key
**Endpoint**: `CloudAPIClient::validate_key(api_key: &str) -> Result<bool, ApiError>`

**Description**: Validate API key with external service

**Parameters**:
- `api_key`: API key to validate

**Returns**:
- `Ok(true)` - API key is valid
- `Ok(false)` - API key is invalid
- `Err(ApiError)` - Network or service error

### Test Connection
**Endpoint**: `CloudAPIClient::test_connection(config: &CloudAPIConfig) -> Result<(), ApiError>`

**Description**: Test connection to external AI service

**Parameters**:
- `config`: Cloud API configuration

**Returns**:
- `Ok(())` - Connection successful
- `Err(ApiError)` - Connection failed

## Configuration File Format

### TOML Structure
```toml
[model]
name = "gpt-3.5-turbo"
type = "cloud"
max_tokens = 2048
temperature = 0.7

[cloud_api]
api_url = "https://api.openai.com/v1/chat/completions"
api_key = "sk-..."
model_name = "gpt-3.5-turbo"
timeout_seconds = 30

[safety]
require_confirmation_for = ["rm -rf", "sudo"]
auto_confirm_dangerous = false
log_all_commands = true

[shell]
default_prompt = "kaido> "
history_size = 1000
auto_complete = true

[ai]
explanation_style = "beginner"
safety_level = "medium"
auto_execute_plans = false
```

## Error Handling

### Configuration Errors
- All configuration errors must provide clear, actionable messages
- Missing files should suggest default configuration creation
- Invalid values should specify valid ranges or options
- Permission errors should suggest solutions

### API Errors
- Network errors should provide retry suggestions
- Authentication errors should guide users to check API keys
- Rate limit errors should suggest waiting or upgrading plans
- Service errors should provide fallback options
