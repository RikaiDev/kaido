# API Contracts: Fix Build Warnings and Remove All Emojis

**Feature**: Fix Build Warnings and Remove All Emojis  
**Date**: 2024-12-19  
**Phase**: Phase 1 - Design & Contracts

## Overview

This feature involves code cleanup operations that are primarily internal to the development process. The API contracts define the interfaces for build analysis, emoji detection, and cleanup operations.

## Build Analysis API

### GET /build/warnings
Analyze the current build for warnings.

**Request**:
```json
{
  "target": "debug|release",
  "verbose": true
}
```

**Response**:
```json
{
  "success": true,
  "warnings": [
    {
      "id": "warning_001",
      "file_path": "src/main.rs",
      "line_number": 42,
      "warning_type": "unused_variable",
      "message": "unused variable `x`",
      "severity": "warning"
    }
  ],
  "total_warnings": 1,
  "build_time_ms": 1500
}
```

### POST /build/warnings/fix
Apply a fix to resolve a specific warning.

**Request**:
```json
{
  "warning_id": "warning_001",
  "fix_type": "remove_unused",
  "description": "Remove unused variable x"
}
```

**Response**:
```json
{
  "success": true,
  "fix_id": "fix_001",
  "applied_at": "2024-12-19T10:30:00Z",
  "verified": true
}
```

## Emoji Detection API

### GET /codebase/emojis
Scan the codebase for emoji characters.

**Request**:
```json
{
  "file_types": [".rs", ".toml", ".md"],
  "include_context": true
}
```

**Response**:
```json
{
  "success": true,
  "emojis": [
    {
      "id": "emoji_001",
      "file_path": "src/main.rs",
      "line_number": 15,
      "character_position": 25,
      "emoji_character": "",
      "context": "println!(\"Hello  world\");",
      "file_type": "rust_source"
    }
  ],
  "total_emojis": 1,
  "scan_time_ms": 500
}
```

### POST /codebase/emojis/remove
Remove an emoji from the codebase.

**Request**:
```json
{
  "emoji_id": "emoji_001",
  "removal_method": "replace",
  "replacement_text": "rocket"
}
```

**Response**:
```json
{
  "success": true,
  "removal_id": "removal_001",
  "removed_at": "2024-12-19T10:35:00Z",
  "verified": true
}
```

## Verification API

### GET /verification/build
Verify that the build has no warnings.

**Request**:
```json
{
  "target": "debug|release"
}
```

**Response**:
```json
{
  "success": true,
  "warnings_found": 0,
  "build_successful": true,
  "build_time_ms": 1200
}
```

### GET /verification/emojis
Verify that no emojis remain in the codebase.

**Request**:
```json
{
  "file_types": [".rs", ".toml", ".md", ".txt", ".json", ".yaml"]
}
```

**Response**:
```json
{
  "success": true,
  "emojis_found": 0,
  "scan_complete": true,
  "scan_time_ms": 300
}
```

## Constitution API

### GET /constitution/emoji-policy
Check the current emoji policy in the constitution.

**Request**: None

**Response**:
```json
{
  "success": true,
  "emoji_prohibited": true,
  "policy_section": "Professional Code Standards",
  "last_updated": "2024-12-19T10:00:00Z"
}
```

### POST /constitution/emoji-policy
Update the emoji policy in the constitution.

**Request**:
```json
{
  "action": "add_prohibition",
  "policy_text": "Emoji characters are strictly prohibited in all source code, configuration files, documentation, and user-facing content."
}
```

**Response**:
```json
{
  "success": true,
  "updated_at": "2024-12-19T10:00:00Z",
  "version": "1.2.0"
}
```

## Error Responses

All endpoints return consistent error responses:

```json
{
  "success": false,
  "error": {
    "code": "BUILD_FAILED",
    "message": "Build process failed",
    "details": "Compilation error in src/main.rs:42"
  }
}
```

## Common Error Codes

- `BUILD_FAILED`: Build process failed
- `EMOJI_NOT_FOUND`: Specified emoji not found
- `WARNING_NOT_FOUND`: Specified warning not found
- `FILE_NOT_FOUND`: File not found
- `PERMISSION_DENIED`: Insufficient permissions
- `CONSTITUTION_UPDATE_FAILED`: Failed to update constitution

## Rate Limiting

- Build analysis: 10 requests per minute
- Emoji detection: 20 requests per minute
- Verification: 30 requests per minute
- Constitution updates: 5 requests per minute
