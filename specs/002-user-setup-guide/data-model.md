# Data Model: User Setup Guide

**Feature**: User Setup Guide  
**Date**: 2025-10-22

## Core Entities

### ConfigurationFile
**Purpose**: Main configuration container for all user settings

**Fields**:
- `model`: ModelConfig - AI model configuration
- `cloud_api`: Option<CloudAPIConfig> - External AI service configuration
- `safety`: SafetyConfig - Safety and confirmation settings
- `shell`: ShellConfig - Shell behavior preferences
- `logging`: LoggingConfig - Logging configuration
- `ai`: AIConfig - AI behavior preferences
- `privacy`: PrivacyConfig - Privacy settings

**Validation Rules**:
- Must have valid model configuration
- Cloud API config required when using external services
- All numeric values must be positive
- File paths must be valid and accessible

### CloudAPIConfig
**Purpose**: Configuration for external AI services

**Fields**:
- `api_url`: String - API endpoint URL
- `api_key`: String - Authentication token (encrypted in storage)
- `model_name`: String - Model identifier (e.g., "gpt-3.5-turbo")
- `timeout_seconds`: u64 - Request timeout

**Validation Rules**:
- API URL must be valid HTTPS URL
- API key must be non-empty
- Timeout must be between 1-300 seconds
- Model name must be supported by the service

### UserProfile
**Purpose**: Individual user preferences and settings

**Fields**:
- `explanation_style`: String - "beginner", "intermediate", "advanced"
- `safety_level`: String - "low", "medium", "high"
- `auto_execute_plans`: bool - Whether to auto-execute AI plans
- `max_retries`: usize - Maximum retry attempts
- `response_timeout`: u64 - AI response timeout

**Validation Rules**:
- Explanation style must be valid enum value
- Safety level must be valid enum value
- Max retries must be between 0-10
- Response timeout must be between 1-120 seconds

### InstallationEnvironment
**Purpose**: System requirements and environment information

**Fields**:
- `operating_system`: String - OS identifier
- `rust_version`: String - Required Rust version
- `dependencies`: Vec<String> - Required system dependencies
- `permissions`: Vec<String> - Required file permissions
- `network_requirements`: Vec<String> - Network access requirements

**Validation Rules**:
- Rust version must meet minimum requirement
- All dependencies must be available
- Required permissions must be granted
- Network access must be available for cloud services

## State Transitions

### Configuration Loading
1. **Initial**: No configuration loaded
2. **Loading**: Reading configuration file
3. **Validating**: Checking configuration validity
4. **Ready**: Configuration loaded and valid
5. **Error**: Configuration invalid or missing

### API Key Management
1. **Missing**: No API key configured
2. **Invalid**: API key provided but invalid
3. **Valid**: API key validated and working
4. **Expired**: API key expired or revoked

## Relationships

- ConfigurationFile contains CloudAPIConfig (optional)
- ConfigurationFile contains UserProfile (embedded)
- InstallationEnvironment validates ConfigurationFile requirements
- UserProfile influences AI behavior settings
