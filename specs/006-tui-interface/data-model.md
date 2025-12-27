# Data Model: Professional TUI Interface for Kaido AI Shell

**Feature**: 006-tui-interface
**Date**: 2025-10-24
**Purpose**: Define state structures and relationships for TUI implementation

## Core Entities

### 1. KaidoApp (Application State)

**Purpose**: Root state container for the entire TUI application, managing user input, command history, AI output, UI mode, and modal dialogs.

**Fields**:

| Field Name | Type | Purpose | Validation/Constraints |
|------------|------|---------|------------------------|
| `input` | `String` | Current user input being typed | Max 1024 chars to prevent memory issues |
| `history` | `Vec<String>` | Command history (past inputs) | Max 1000 entries, FIFO eviction |
| `output` | `String` | Last command execution output | Max 100KB, truncate if exceeded |
| `ai_panel_toggle` | `bool` | Toggle state for AI analysis view | `false` = spinner, `true` = JSON |
| `ai_thinking` | `bool` | Whether AI is currently processing | `true` shows spinner animation |
| `ai_output` | `String` | JSON output from AI (CommandSequence) | Valid JSON string |
| `spinner_index` | `usize` | Current spinner animation frame | 0-9 (cycles through 10 frames) |
| `modal` | `Option<ModalDialog>` | Active modal dialog if any | `None` = no modal, `Some` = modal blocking |
| `state` | `AppState` | Current application state | Enum: Normal, ModalActive, Executing |

**Methods**:
- `new()` → `Self`: Initialize with empty/default values
- `toggle_ai_panel(&mut self)`: Flip `ai_panel_toggle` boolean
- `next_spinner_frame(&mut self)`: Increment `spinner_index` mod 10
- `clear_input(&mut self)`: Reset `input` to empty string
- `add_to_history(&mut self, cmd: String)`: Push to `history`, enforce max size

**Lifecycle**:
1. Created on application start with `new()`
2. Mutated by event handlers throughout application runtime
3. Dropped on application exit (no persistence needed)

**Relationships**:
- Contains zero or one `ModalDialog` (composition via `Option`)
- Owns command history (owned `Vec`)
- Holds references to UI state (owned primitives)

---

### 2. ModalDialog (Safety Confirmation Dialog)

**Purpose**: Represents the blocking confirmation dialog shown when a dangerous command is detected, capturing user's choice to allow/deny execution.

**Fields**:

| Field Name | Type | Purpose | Validation/Constraints |
|------------|------|---------|------------------------|
| `command` | `String` | The dangerous command to be confirmed | Non-empty, contains dangerous pattern |
| `description` | `String` | Human-readable description of command | Non-empty |
| `selected_option` | `usize` | Currently highlighted option | 0 = Allow Once, 1 = Allow Always, 2 = Deny |

**Methods**:
- `new(command: String, description: String)` → `Self`: Create with default selection (0)
- `render(&self, frame: &mut Frame, area: Rect)`: Render centered modal dialog
- `select_option(&mut self, option: usize)`: Update `selected_option` if valid (0-2)

**Lifecycle**:
1. Created when `SafeExecutor` detects dangerous command
2. Stored in `KaidoApp.modal` as `Some(ModalDialog)`
3. Destroyed when user selects option (set to `None`)

**Relationships**:
- Owned by `KaidoApp` via `Option<ModalDialog>`
- References command from `CommandSequence` (owned copy)
- No persistence (ephemeral per command)

---

### 3. AppState (Application State Machine)

**Purpose**: Enum representing the current mode of the application for event routing and UI rendering.

**Variants**:

| Variant | Purpose | Valid Transitions |
|---------|---------|-------------------|
| `Normal` | Regular input mode, accepting user commands | → `Executing` (on Enter press) |
| `ModalActive` | Modal dialog is blocking, waiting for 1/2/3 | → `Normal` (on option selected) |
| `Executing` | Command is running, UI shows progress | → `Normal` (on completion), → `ModalActive` (on dangerous cmd) |

**State Transitions**:

```
Normal --[Enter + safe command]--> Executing --[completion]--> Normal
Normal --[Enter + dangerous command]--> Executing --[safety check]--> ModalActive
ModalActive --[1/2/3 pressed]--> Normal
```

**Lifecycle**:
- Starts in `Normal` state on app launch
- Transitions driven by user input and command execution
- No persistence needed

**Relationships**:
- Owned by `KaidoApp` as direct field
- Controls event routing in `handle_key()`
- Determines which UI elements render

---

### 4. Allowlist (Persistent Command Allowlist)

**Purpose**: Manages the list of dangerous commands that users have permanently approved, stored persistently across sessions.

**Fields**:

| Field Name | Type | Purpose | Validation/Constraints |
|------------|------|---------|------------------------|
| `allowed_commands` | `HashSet<String>` | Set of approved command strings | Exact match comparison |
| `file_path` | `PathBuf` | Path to allowlist file | `~/.config/kaido/allowlist.txt` |

**Methods**:
- `load()` → `KaidoResult<Self>`: Load from file, create empty if missing
- `save(&self)` → `KaidoResult<()>`: Write to file atomically
- `add(&mut self, command: String)` → `KaidoResult<()>`: Add command and save
- `is_allowed(&self, command: &str)` → `bool`: Check if command in set
- `remove(&mut self, command: &str)` → `KaidoResult<()>`: Remove command and save

**File Format** (Plain Text):
```text
# One command per line
# Lines starting with # are comments
rm test.txt
echo "hello" > /tmp/test.txt
sudo apt update
```

**Lifecycle**:
1. Loaded on application start from `~/.config/kaido/allowlist.txt`
2. Modified when user selects "Allow Always" in modal (option 2)
3. Saved to disk on every modification
4. Persists across application restarts

**Relationships**:
- Used by `SafeExecutor` to check commands before showing modal
- Independent of `KaidoApp` (loaded separately)
- No foreign key relationships (flat list)

---

### 5. CommandSequence (AI Output Structure)

**Purpose**: Represents the AI's generated task plan as a sequence of shell commands with descriptions. (Already exists in `src/ai/parser.rs` - included here for completeness)

**Fields**:

| Field Name | Type | Purpose | Validation/Constraints |
|------------|------|---------|------------------------|
| `task` | `String` | High-level task description | Non-empty |
| `commands` | `Vec<Command>` | Ordered list of commands to execute | Min 1 command |

**Sub-Entity: Command**:

| Field Name | Type | Purpose | Validation/Constraints |
|------------|------|---------|------------------------|
| `cmd` | `String` | Shell command to execute | Non-empty, valid shell syntax |
| `description` | `String` | Human-readable explanation | Non-empty |

**JSON Schema Example**:
```json
{
  "task": "List files and show current directory",
  "commands": [
    {
      "cmd": "ls -la",
      "description": "List all files with details"
    },
    {
      "cmd": "pwd",
      "description": "Show current directory path"
    }
  ]
}
```

**Lifecycle**:
1. Created by AI (`AIManager.generate_response()`)
2. Serialized to JSON and stored in `KaidoApp.ai_output` (if toggle enabled)
3. Passed to `SafeExecutor` for validation and execution
4. Ephemeral (not persisted)

**Relationships**:
- Generated by `AIManager` (producer)
- Consumed by `SafeExecutor` (consumer)
- Displayed in TUI right panel when `ai_panel_toggle = true`
- Commands checked against `Allowlist` before execution

---

## Data Flow Diagram

```
User Input
    ↓
KaidoApp.input (String)
    ↓ [Enter pressed]
AIManager.generate_response()
    ↓
CommandSequence {task, commands[]}
    ↓
KaidoApp.ai_output (JSON String) ← [if ai_panel_toggle]
    ↓
SafeExecutor.execute_sequence()
    ↓
For each Command:
    ↓
    Is dangerous? → NO → Execute directly
    ↓ YES
    Allowlist.is_allowed()
    ↓
    In allowlist? → YES → Execute directly
    ↓ NO
    ModalDialog {command, description}
    ↓
    KaidoApp.modal = Some(ModalDialog)
    KaidoApp.state = ModalActive
    ↓
User selects option:
    1 → Execute once
    2 → Allowlist.add() + Execute
    3 → Cancel
    ↓
KaidoApp.modal = None
KaidoApp.state = Normal
```

---

## State Transitions

### Normal Input Flow

```
AppState::Normal
    ↓ [user types "查詢資料夾"]
KaidoApp.input = "查詢資料夾"
    ↓ [user presses Enter]
AppState::Executing
KaidoApp.ai_thinking = true
    ↓ [AIManager processing]
spinner_index cycles 0→1→2→...
    ↓ [AI returns CommandSequence]
KaidoApp.ai_thinking = false
KaidoApp.ai_output = "{\"task\":\"...\"}" (if toggled)
    ↓ [SafeExecutor checks commands]
All safe → Execute → Output displayed
    ↓
AppState::Normal
```

### Dangerous Command Flow

```
AppState::Normal
    ↓ [user types "刪除測試檔案"]
KaidoApp.input = "刪除測試檔案"
    ↓ [user presses Enter]
AppState::Executing
    ↓ [AI returns "rm test.txt"]
SafeExecutor detects "rm" pattern
    ↓
Allowlist.is_allowed("rm test.txt") → false
    ↓
ModalDialog { command: "rm test.txt", description: "..." }
KaidoApp.modal = Some(...)
AppState::ModalActive
    ↓ [user presses "2"]
Allowlist.add("rm test.txt")
Allowlist.save()
Execute command
KaidoApp.modal = None
AppState::Normal
```

---

## Validation Rules

### KaidoApp

- `input`: Maximum 1024 characters to prevent denial-of-service via paste
- `history`: Maximum 1000 entries, evict oldest when exceeded
- `output`: Maximum 100KB text, truncate with "... (output truncated)" if exceeded
- `spinner_index`: Always 0 ≤ index < 10 (enforced by modulo in `next_spinner_frame()`)
- `ai_output`: Must be valid JSON or empty string

### ModalDialog

- `command`: Must be non-empty and contain at least one dangerous pattern
- `description`: Must be non-empty
- `selected_option`: Must be 0, 1, or 2 (validated before use)

### Allowlist

- `file_path`: Must be writable, directory must exist (created if missing)
- `allowed_commands`: Each command must be exact match (no wildcards)
- File format: Lines starting with `#` are comments, empty lines ignored

### CommandSequence

- `task`: Must be non-empty string
- `commands`: Must contain at least 1 command
- Each `Command.cmd`: Must be non-empty, should be valid shell syntax (validated by execution layer)
- Each `Command.description`: Must be non-empty

---

## Error Handling

### KaidoApp

- `input` overflow → Reject additional characters, show "Input limit reached" message
- `history` overflow → Silently evict oldest entry (FIFO)
- `output` overflow → Truncate with marker message
- Invalid `spinner_index` → Impossible (enforced by modulo arithmetic)

### ModalDialog

- Invalid `selected_option` → Ignore input, remain in modal state
- Empty `command` → Should never occur (validated before ModalDialog creation)
- Empty `description` → Should never occur (AI always generates description)

### Allowlist

- File not found → Create empty allowlist
- File permission denied → Return error, show message to user
- File corrupted → Log warning, use empty allowlist, overwrite on next save
- Directory not exists → Create directory tree
- Disk full → Return error, modal shows "Could not save allowlist"

### CommandSequence

- Missing `task` field → Return parse error, show "AI generated invalid output"
- Empty `commands` array → Return parse error, show "No commands generated"
- Invalid JSON → Return parse error, show raw AI output for debugging
- Command execution failure → Capture stderr, display in `KaidoApp.output`

---

## Persistence

### Session-Scoped (In-Memory Only)

- `KaidoApp` entire state
- `ModalDialog` instances
- `AppState` transitions
- Command history (`KaidoApp.history`)
- Current input (`KaidoApp.input`)
- Spinner animation state (`spinner_index`)

### Persistent (Across Restarts)

- `Allowlist` → `~/.config/kaido/allowlist.txt` (plain text)

**Rationale for Limited Persistence**:
- Command history persistence deferred to future enhancement
- AI output caching deferred to future enhancement
- UI state (toggle, panel sizes) uses defaults on restart
- Focus on MVP: Only essential safety data (allowlist) persists

---

## Testing Considerations

### Unit Tests

- `KaidoApp::new()`: Verify all fields initialized correctly
- `KaidoApp::next_spinner_frame()`: Verify modulo cycling (9 → 0)
- `KaidoApp::toggle_ai_panel()`: Verify boolean flip
- `ModalDialog::new()`: Verify default `selected_option = 0`
- `Allowlist::load()`: Test empty file, missing file, corrupted file
- `Allowlist::save()`: Verify atomicity, file format correctness
- `AppState` transitions: Verify all valid transitions, reject invalid ones

### Integration Tests

- Full flow: Input → AI → Safe execution → Output display
- Full flow: Input → AI → Dangerous command → Modal → Allowlist → Execution
- Edge case: Rapid input while AI thinking (queue or block?)
- Edge case: Modal displayed, user presses Ctrl+C (should cancel gracefully)
- Edge case: Allowlist file deleted while app running (should handle on next save)

### Manual TUI Tests

- Verify spinner animation smooth at ≥10 FPS
- Verify modal centered and readable at various terminal sizes
- Verify Ctrl+T toggle switches between spinner and JSON view
- Verify Ctrl+C exits cleanly with terminal restored
- Verify command history scrolls correctly when > screen height

