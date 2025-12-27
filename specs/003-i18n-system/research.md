# Research: Internationalization (i18n) System

**Feature**: Internationalization (i18n) System  
**Date**: 2025-10-23  
**Purpose**: Resolve technical unknowns and establish implementation approach

## Research Tasks

### Task 1: Rust i18n Framework Selection

**Research Question**: What is the best Rust i18n framework for CLI applications with AI Agent integration?

**Findings**:
- **Decision**: fluent-rs with unic-langid
- **Rationale**: 
  - fluent-rs is the Rust implementation of Mozilla's Fluent localization system
  - Provides rich message formatting with variables and pluralization
  - unic-langid handles locale detection and parsing
  - Both are actively maintained and widely used
  - Excellent performance for CLI applications
- **Alternatives Considered**:
  - gettext-rs: Traditional but limited formatting capabilities
  - i18n-rs: Simpler but lacks advanced features needed for AI responses
  - Custom solution: Too complex for MVP

### Task 2: AI Agent Chain of Thought Implementation

**Research Question**: How to implement Chain of Thought reasoning in Rust for multi-language AI processing?

**Findings**:
- **Decision**: Structured prompt engineering with step-by-step reasoning
- **Rationale**:
  - Use structured prompts that guide AI through reasoning steps
  - Implement reasoning state tracking for multi-step tasks
  - Store intermediate reasoning results for transparency
  - Use language-specific prompt templates for cultural context
- **Alternatives Considered**:
  - External CoT libraries: Limited Rust support
  - Custom reasoning engine: Too complex for MVP
  - Simple prompt concatenation: Insufficient for complex tasks

### Task 3: Locale Detection Strategy

**Research Question**: How to detect and manage system locales across different platforms?

**Findings**:
- **Decision**: Platform-specific locale detection with fallback chain
- **Rationale**:
  - Use system APIs for locale detection (LC_ALL, LANG environment variables)
  - Implement fallback chain: user preference → system locale → English
  - Support runtime locale switching without restart
  - Cache locale information for performance
- **Alternatives Considered**:
  - Browser-style locale detection: Not applicable to CLI
  - Manual configuration only: Poor user experience
  - Single locale per session: Too restrictive

### Task 4: Translation Resource Management

**Research Question**: How to structure and manage translation resources for AI Agent responses?

**Findings**:
- **Decision**: Hierarchical TOML files with AI-specific sections
- **Rationale**:
  - Separate sections for UI strings, AI prompts, and cultural context
  - Use TOML for human-readable translation files
  - Implement hot-reloading for development
  - Support parameterized messages for AI responses
- **Alternatives Considered**:
  - JSON format: Less readable for translators
  - Database storage: Overkill for CLI application
  - Single large file: Hard to maintain

### Task 5: Cultural Context Integration

**Research Question**: How to integrate cultural context into AI Agent responses?

**Findings**:
- **Decision**: Cultural context metadata in translation resources
- **Rationale**:
  - Include cultural context as metadata in translation files
  - Provide cultural hints to AI Agent prompts
  - Support region-specific formatting and conventions
  - Allow cultural context override by users
- **Alternatives Considered**:
  - External cultural databases: Too complex for MVP
  - Hardcoded cultural rules: Not flexible
  - AI-only cultural detection: Unreliable

## Technical Decisions Summary

1. **i18n Framework**: fluent-rs + unic-langid for robust localization
2. **AI Agent Architecture**: Structured prompt engineering with reasoning state tracking
3. **Locale Detection**: Platform-specific detection with intelligent fallback
4. **Resource Management**: Hierarchical TOML files with hot-reloading
5. **Cultural Integration**: Metadata-driven cultural context in translation resources

## Implementation Approach

- Start with core languages (English, Chinese, Japanese, Spanish)
- Implement basic AI Agent CoT reasoning first
- Add cultural context gradually based on user feedback
- Maintain backward compatibility with existing shell functionality
- Focus on testability and performance from the start
