# API Contracts: Internationalization (i18n) System

**Feature**: Internationalization (i18n) System  
**Date**: 2025-10-23  
**Purpose**: Define API contracts for i18n system components

## Core API Contracts

### Locale Management API

#### Detect System Locale
```rust
pub async fn detect_system_locale() -> Result<Option<String>, LocaleError>
```

**Purpose**: Automatically detect the user's system locale

**Returns**: 
- `Ok(Some(locale))` - Detected locale (e.g., "zh-CN")
- `Ok(None)` - No locale detected, use fallback
- `Err(LocaleError)` - Detection failed

**Error Types**:
- `LocaleError::DetectionFailed` - System API call failed
- `LocaleError::InvalidLocale` - Detected locale is invalid

#### Set User Locale
```rust
pub async fn set_user_locale(locale: String) -> Result<(), LocaleError>
```

**Purpose**: Set the user's preferred locale

**Parameters**:
- `locale: String` - BCP 47 language tag (e.g., "zh-CN", "ja-JP")

**Returns**:
- `Ok(())` - Locale set successfully
- `Err(LocaleError)` - Invalid locale or loading failed

#### Get Current Locale
```rust
pub fn get_current_locale() -> String
```

**Purpose**: Get the currently active locale

**Returns**: Current locale string (never empty, fallback to "en")

### Translation API

#### Load Translation Resources
```rust
pub async fn load_translation_resources(locale: String) -> Result<TranslationResources, TranslationError>
```

**Purpose**: Load translation resources for a specific locale

**Parameters**:
- `locale: String` - Target locale

**Returns**:
- `Ok(TranslationResources)` - Loaded resources
- `Err(TranslationError)` - Loading failed

**Error Types**:
- `TranslationError::FileNotFound` - Translation file missing
- `TranslationError::ParseError` - Invalid translation format
- `TranslationError::MissingKeys` - Required keys missing

#### Translate String
```rust
pub fn translate(key: String, params: Option<HashMap<String, String>>) -> Result<String, TranslationError>
```

**Purpose**: Translate a string key with optional parameters

**Parameters**:
- `key: String` - Translation key
- `params: Option<HashMap<String, String>>` - Template parameters

**Returns**:
- `Ok(String)` - Translated string
- `Err(TranslationError)` - Translation failed

#### Get Cultural Context
```rust
pub fn get_cultural_context(locale: String) -> Result<CulturalContext, CulturalError>
```

**Purpose**: Get cultural context for a locale

**Parameters**:
- `locale: String` - Target locale

**Returns**:
- `Ok(CulturalContext)` - Cultural context
- `Err(CulturalError)` - Context not available

### AI Agent API

#### Process Natural Language Request
```rust
pub async fn process_natural_language(
    input: String,
    language: String,
    cultural_context: CulturalContext
) -> Result<AIResponse, AIError>
```

**Purpose**: Process natural language input with Chain of Thought reasoning

**Parameters**:
- `input: String` - User's natural language input
- `language: String` - Target language for response
- `cultural_context: CulturalContext` - Cultural context

**Returns**:
- `Ok(AIResponse)` - AI response with reasoning
- `Err(AIError)` - Processing failed

**AIResponse Structure**:
```rust
pub struct AIResponse {
    pub reasoning_steps: Vec<ReasoningStep>,
    pub generated_commands: Vec<String>,
    pub explanation: String,
    pub cultural_adaptations: Vec<String>,
    pub confidence_score: f32,
}
```

#### Get Reasoning Process
```rust
pub fn get_reasoning_process(session_id: String) -> Result<Vec<ReasoningStep>, AIError>
```

**Purpose**: Get the reasoning process for transparency

**Parameters**:
- `session_id: String` - AI Agent session ID

**Returns**:
- `Ok(Vec<ReasoningStep>)` - Reasoning steps
- `Err(AIError)` - Session not found

#### Execute Command Sequence
```rust
pub async fn execute_command_sequence(
    commands: Vec<String>,
    language: String
) -> Result<ExecutionResult, ExecutionError>
```

**Purpose**: Execute a sequence of commands with localized output

**Parameters**:
- `commands: Vec<String>` - Commands to execute
- `language: String` - Language for output formatting

**Returns**:
- `Ok(ExecutionResult)` - Execution results
- `Err(ExecutionError)` - Execution failed

**ExecutionResult Structure**:
```rust
pub struct ExecutionResult {
    pub successful_commands: Vec<CommandResult>,
    pub failed_commands: Vec<CommandError>,
    pub total_execution_time: Duration,
    pub localized_output: String,
}
```

### Configuration API

#### Update i18n Configuration
```rust
pub async fn update_i18n_config(config: I18nConfig) -> Result<(), ConfigError>
```

**Purpose**: Update i18n system configuration

**Parameters**:
- `config: I18nConfig` - New configuration

**Returns**:
- `Ok(())` - Configuration updated
- `Err(ConfigError)` - Invalid configuration

**I18nConfig Structure**:
```rust
pub struct I18nConfig {
    pub default_locale: String,
    pub supported_locales: Vec<String>,
    pub auto_detect_locale: bool,
    pub fallback_locale: String,
    pub cultural_context_enabled: bool,
    pub reasoning_transparency: bool,
}
```

#### Get i18n Status
```rust
pub fn get_i18n_status() -> I18nStatus
```

**Purpose**: Get current i18n system status

**Returns**: Current status information

**I18nStatus Structure**:
```rust
pub struct I18nStatus {
    pub current_locale: String,
    pub loaded_resources: Vec<String>,
    pub active_sessions: u32,
    pub last_locale_change: DateTime<Utc>,
    pub cultural_context_active: bool,
}
```

## Error Handling

### Common Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum I18nError {
    #[error("Locale error: {0}")]
    Locale(LocaleError),
    
    #[error("Translation error: {0}")]
    Translation(TranslationError),
    
    #[error("Cultural context error: {0}")]
    Cultural(CulturalError),
    
    #[error("AI processing error: {0}")]
    AI(AIError),
    
    #[error("Configuration error: {0}")]
    Config(ConfigError),
}
```

### Error Recovery Strategies

1. **Locale Detection Failure**: Fallback to English
2. **Translation Missing**: Use key as fallback, log warning
3. **Cultural Context Missing**: Use default cultural context
4. **AI Processing Failure**: Return error with suggestions
5. **Configuration Invalid**: Revert to last valid configuration

## Performance Requirements

- Locale detection: < 100ms
- Translation loading: < 500ms
- AI processing: < 5s for complex requests
- Language switching: < 2s
- Cultural context lookup: < 50ms

## Security Considerations

- Validate all locale strings to prevent injection
- Sanitize translation parameters
- Limit AI reasoning steps to prevent infinite loops
- Validate command sequences before execution
- Log all i18n operations for audit
