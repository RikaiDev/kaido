# Data Model: Internationalization (i18n) System

**Feature**: Internationalization (i18n) System  
**Date**: 2025-10-23  
**Purpose**: Define core entities and their relationships for the i18n system

## Core Entities

### LanguageConfiguration

**Purpose**: Manages user language preferences and system locale detection

**Fields**:
- `current_locale: String` - Currently active locale (e.g., "zh-CN", "en-US")
- `preferred_languages: Vec<String>` - User's language preference order
- `system_locale: Option<String>` - Detected system locale
- `fallback_locale: String` - Default fallback locale (always "en")
- `auto_detect: bool` - Whether to automatically detect system locale
- `last_updated: DateTime<Utc>` - When configuration was last modified

**Validation Rules**:
- `current_locale` must be a valid BCP 47 language tag
- `preferred_languages` must contain at least one valid locale
- `fallback_locale` must always be "en" or "en-US"

**State Transitions**:
- `Initializing` → `Detected` (system locale detection)
- `Detected` → `UserSelected` (user changes language)
- `UserSelected` → `Detected` (user enables auto-detection)

### TranslationResources

**Purpose**: Manages loaded translation strings and resources

**Fields**:
- `locale: String` - Target locale for these resources
- `ui_strings: HashMap<String, String>` - Interface element translations
- `ai_prompts: HashMap<String, String>` - AI Agent prompt templates
- `cultural_context: CulturalContext` - Cultural metadata
- `last_loaded: DateTime<Utc>` - When resources were loaded
- `version: String` - Resource version for cache invalidation

**Validation Rules**:
- All keys must be non-empty strings
- AI prompts must contain valid template variables
- Cultural context must be valid for the target locale

### CulturalContext

**Purpose**: Provides cultural and regional context for AI responses

**Fields**:
- `region: String` - Geographic region (e.g., "CN", "JP", "US")
- `timezone: String` - Default timezone for the locale
- `date_format: String` - Preferred date format
- `number_format: String` - Number formatting rules
- `currency: Option<String>` - Default currency code
- `cultural_norms: HashMap<String, String>` - Cultural behavior hints
- `technical_terms: HashMap<String, String>` - Localized technical terminology

**Validation Rules**:
- `region` must be a valid ISO 3166-1 alpha-2 code
- `timezone` must be a valid IANA timezone identifier
- Date and number formats must be valid format strings

### AIAgentEngine

**Purpose**: Core AI Agent with Chain of Thought reasoning capabilities

**Fields**:
- `current_language: String` - Language for current reasoning session
- `reasoning_state: ReasoningState` - Current CoT reasoning state
- `cultural_context: CulturalContext` - Active cultural context
- `prompt_templates: HashMap<String, String>` - Language-specific prompts
- `session_id: String` - Unique session identifier
- `created_at: DateTime<Utc>` - Session creation time

**Validation Rules**:
- `current_language` must match available translation resources
- `reasoning_state` must be valid for the current task
- `prompt_templates` must contain required CoT templates

### ReasoningState

**Purpose**: Tracks Chain of Thought reasoning process

**Fields**:
- `current_step: u32` - Current reasoning step number
- `total_steps: u32` - Total expected reasoning steps
- `reasoning_steps: Vec<ReasoningStep>` - Completed reasoning steps
- `current_task: String` - Description of current task
- `context: HashMap<String, String>` - Context variables
- `status: ReasoningStatus` - Current reasoning status

**Validation Rules**:
- `current_step` must be <= `total_steps`
- All reasoning steps must be valid and complete
- Context variables must be properly typed

### ReasoningStep

**Purpose**: Individual step in Chain of Thought reasoning

**Fields**:
- `step_number: u32` - Sequential step number
- `description: String` - Human-readable step description
- `reasoning: String` - AI reasoning for this step
- `action: Option<String>` - Command or action to execute
- `result: Option<String>` - Result of the action
- `timestamp: DateTime<Utc>` - When step was completed

**Validation Rules**:
- `step_number` must be sequential
- `description` must be non-empty
- `reasoning` must contain valid reasoning content

## Entity Relationships

### Primary Relationships

1. **LanguageConfiguration** → **TranslationResources** (1:1)
   - Each configuration has one active translation resource set
   - Resources are loaded based on current locale

2. **TranslationResources** → **CulturalContext** (1:1)
   - Each translation resource includes cultural context
   - Cultural context is locale-specific

3. **AIAgentEngine** → **CulturalContext** (1:1)
   - AI Agent uses cultural context for responses
   - Context influences reasoning and output

4. **AIAgentEngine** → **ReasoningState** (1:1)
   - Each AI Agent session has one reasoning state
   - State tracks the current CoT process

5. **ReasoningState** → **ReasoningStep** (1:N)
   - Reasoning state contains multiple steps
   - Steps are executed sequentially

### Secondary Relationships

1. **LanguageConfiguration** → **AIAgentEngine** (1:N)
   - Configuration can spawn multiple AI Agent sessions
   - Each session inherits language settings

2. **TranslationResources** → **AIAgentEngine** (1:N)
   - Multiple AI Agents can use the same translation resources
   - Resources are shared across sessions

## Data Flow

1. **Initialization**: System detects locale → loads translation resources → creates cultural context
2. **Language Switch**: User changes language → reloads resources → updates AI Agent context
3. **AI Processing**: User input → AI Agent reasoning → step-by-step execution → localized response
4. **Cultural Adaptation**: AI responses incorporate cultural context and regional preferences

## Validation and Constraints

- All locale strings must be valid BCP 47 language tags
- Translation keys must follow consistent naming conventions
- Cultural context must be validated against known regions
- Reasoning steps must be logically consistent and complete
- All timestamps must be in UTC format
- Resource versions must be monotonically increasing
