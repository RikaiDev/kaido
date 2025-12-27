# Data Model: Fix Build Warnings and Remove All Emojis

**Feature**: Fix Build Warnings and Remove All Emojis  
**Date**: 2024-12-19  
**Phase**: Phase 1 - Design & Contracts

## Entities

### BuildWarning
Represents a compiler warning that needs to be resolved.

**Fields**:
- `file_path`: String - Path to the source file containing the warning
- `line_number`: u32 - Line number where the warning occurs
- `warning_type`: String - Type of warning (e.g., "unused_variable", "dead_code")
- `message`: String - Full warning message from compiler
- `severity`: WarningSeverity - Level of warning (error, warning, note)

**Relationships**:
- Belongs to SourceFile
- Can be resolved by CodeFix

**Validation Rules**:
- file_path must be a valid file path
- line_number must be > 0
- warning_type must not be empty
- message must not be empty

**State Transitions**:
- `identified` → `analyzed` → `fixed` → `verified`

### EmojiOccurrence
Represents an emoji character found in the codebase.

**Fields**:
- `file_path`: String - Path to the file containing the emoji
- `line_number`: u32 - Line number where emoji occurs
- `character_position`: u32 - Character position within the line
- `emoji_character`: String - The actual emoji character
- `context`: String - Surrounding text context
- `file_type`: FileType - Type of file (.rs, .toml, .md, etc.)

**Relationships**:
- Belongs to SourceFile
- Can be removed by EmojiRemoval

**Validation Rules**:
- file_path must be a valid file path
- line_number must be > 0
- character_position must be >= 0
- emoji_character must be a valid Unicode emoji
- context must not be empty

**State Transitions**:
- `detected` → `analyzed` → `removed` → `verified`

### SourceFile
Represents a file in the codebase that may contain warnings or emojis.

**Fields**:
- `path`: String - File path relative to project root
- `file_type`: FileType - Type of file
- `size_bytes`: u64 - File size in bytes
- `last_modified`: DateTime - Last modification timestamp
- `encoding`: String - File encoding (UTF-8, ASCII)

**Relationships**:
- Contains multiple BuildWarnings
- Contains multiple EmojiOccurrences
- Can be processed by FileProcessor

**Validation Rules**:
- path must be a valid file path
- file_type must be a supported type
- size_bytes must be > 0
- encoding must be a valid encoding

### CodeFix
Represents a fix applied to resolve a build warning.

**Fields**:
- `warning_id`: String - ID of the warning being fixed
- `fix_type`: FixType - Type of fix applied
- `description`: String - Description of the fix
- `applied_at`: DateTime - When the fix was applied
- `verified`: bool - Whether the fix was verified

**Relationships**:
- Resolves BuildWarning
- Created by Developer

**Validation Rules**:
- warning_id must reference existing warning
- fix_type must be valid
- description must not be empty
- applied_at must be valid timestamp

### EmojiRemoval
Represents the removal of an emoji from the codebase.

**Fields**:
- `emoji_id`: String - ID of the emoji being removed
- `removal_method`: RemovalMethod - How the emoji was removed
- `replacement_text`: Option<String> - Text used to replace emoji (if any)
- `removed_at`: DateTime - When the removal was applied
- `verified`: bool - Whether the removal was verified

**Relationships**:
- Removes EmojiOccurrence
- Created by Developer

**Validation Rules**:
- emoji_id must reference existing emoji
- removal_method must be valid
- removed_at must be valid timestamp

## Enums

### WarningSeverity
```rust
enum WarningSeverity {
    Error,
    Warning,
    Note,
    Help,
}
```

### FileType
```rust
enum FileType {
    RustSource,      // .rs
    Configuration,   // .toml
    Documentation,   // .md
    Text,           // .txt
    Json,           // .json
    Yaml,           // .yaml, .yml
    Other,          // Other text files
}
```

### FixType
```rust
enum FixType {
    AddAttribute,      // #[allow(...)]
    RemoveUnused,      // Remove unused code
    AddUse,           // Add missing use statement
    FixType,          // Fix type annotation
    RenameVariable,   // Rename variable
    Other,            // Other fixes
}
```

### RemovalMethod
```rust
enum RemovalMethod {
    Delete,           // Remove emoji completely
    Replace,          // Replace with text
    Comment,          // Comment out line
    Other,            // Other methods
}
```

## Data Flow

1. **Detection Phase**:
   - Scan codebase for build warnings → BuildWarning entities
   - Scan codebase for emojis → EmojiOccurrence entities

2. **Analysis Phase**:
   - Analyze each warning → determine fix type
   - Analyze each emoji → determine removal method

3. **Resolution Phase**:
   - Apply fixes → CodeFix entities
   - Remove emojis → EmojiRemoval entities

4. **Verification Phase**:
   - Verify no warnings remain
   - Verify no emojis remain
   - Verify functionality preserved

## Constraints

- All changes must preserve existing functionality
- Build must complete without warnings
- No emoji characters in any text files
- Constitution must explicitly prohibit emoji usage
- All changes must be reversible via git
