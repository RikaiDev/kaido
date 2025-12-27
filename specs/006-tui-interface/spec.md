# Feature Specification: Professional TUI Interface for Kaido AI Shell

**Feature Branch**: `006-tui-interface`  
**Created**: 2025-10-24  
**Status**: Draft  
**Input**: User description: "TUI Interface Implementation with ratatui framework, left-right split layout, AI thinking animation, modal dialogs for safety confirmation, and silenced llama.cpp logs"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Clean Shell Startup Experience (Priority: P1)

As a user launching Kaido AI Shell, I want to see a clean, professional terminal interface without verbose technical logs, so that I can immediately focus on my tasks without information overload.

**Why this priority**: This is the first impression users get when launching the application. Excessive logging noise (specifically llama.cpp's verbose output) damages user experience and professional appearance. This must work before any UI enhancements can be appreciated.

**Independent Test**: Launch the application and verify no llama.cpp model loading logs appear. Only the clean TUI interface should be visible.

**Acceptance Scenarios**:

1. **Given** the application is not running, **When** I execute `cargo run`, **Then** I see only the TUI interface without any llama.cpp initialization logs
2. **Given** the application is starting up, **When** the LLM model is loading, **Then** the user sees only a clean loading indicator without verbose backend messages
3. **Given** llama.cpp is performing inference, **When** generating responses, **Then** no internal model processing logs are displayed to the user

---

### User Story 2 - Visual AI Thinking Process (Priority: P1)

As a user entering natural language commands, I want to see a visual indication that the AI is processing my request with an animated spinner, so that I know the system is working and not frozen.

**Why this priority**: Without visual feedback, users cannot distinguish between processing and hanging. This is critical for user confidence and prevents premature termination of valid operations.

**Independent Test**: Enter any command and verify the spinner animation appears immediately and continuously updates until processing completes.

**Acceptance Scenarios**:

1. **Given** I have typed a command and pressed Enter, **When** the AI begins processing, **Then** I see an animated spinner (rotating box characters) in the right panel with "Thinking..." text
2. **Given** the spinner is animating, **When** AI processing is in progress, **Then** the spinner continuously cycles through its animation frames at a visible rate
3. **Given** the AI has completed processing, **When** the response is ready, **Then** the spinner disappears and is replaced by the command sequence or result

---

### User Story 3 - Split-Panel Layout with Command and Analysis Views (Priority: P1)

As a user working with the AI shell, I want to see my commands and outputs in the left panel while the AI analysis appears in the right panel, so that I can maintain context of both my interactions and the AI's interpretation simultaneously.

**Why this priority**: The split-panel layout is the foundational UI structure that all other features depend on. Without it, there is no framework for displaying AI analysis, spinners, or command history.

**Independent Test**: Launch the application and verify two distinct panels appear side-by-side with clear visual separation. Type commands in the left panel and observe they remain there while AI output appears on the right.

**Acceptance Scenarios**:

1. **Given** the application is running, **When** I view the interface, **Then** I see two distinct panels: left (70% width) for commands and output, right (30% width) for AI analysis
2. **Given** I am typing a command, **When** I press keys, **Then** the input appears in the left panel's command area
3. **Given** I execute a command, **When** output is generated, **Then** command results appear in the left panel while AI processing information appears in the right panel
4. **Given** the terminal window is resized, **When** dimensions change, **Then** both panels adjust proportionally maintaining their relative sizes

---

### User Story 4 - Toggle AI Analysis Detail View (Priority: P2)

As a power user or developer, I want to toggle the right panel between showing the simple spinner animation and the detailed JSON analysis of the AI's command generation, so that I can understand or debug how the AI interpreted my request when needed.

**Why this priority**: This provides transparency for advanced users and debugging capabilities without overwhelming casual users. It's secondary to the basic functionality but valuable for trust and troubleshooting.

**Independent Test**: Press Ctrl+T while a command is processing or after completion and verify the right panel switches between showing the spinner/"Thinking..." text and the full JSON command sequence.

**Acceptance Scenarios**:

1. **Given** the AI is processing a command with spinner visible, **When** I press Ctrl+T, **Then** the right panel switches to show the detailed JSON output of the AI's command generation
2. **Given** the detailed JSON view is displayed, **When** I press Ctrl+T again, **Then** the panel returns to the simple spinner/status view
3. **Given** I toggle the view mode, **When** I execute subsequent commands, **Then** the panel maintains my selected mode (JSON or simple)
4. **Given** the JSON view is active, **When** the AI generates a multi-step command sequence, **Then** I see the complete structured JSON with task description and command array

---

### User Story 5 - Safety Confirmation Modal for Dangerous Commands (Priority: P1)

As a user whose AI-generated command includes potentially destructive operations, I want to see a clear modal dialog requiring my explicit confirmation with options to allow once, allow always, or deny, so that I can prevent accidental data loss while maintaining workflow efficiency for trusted operations.

**Why this priority**: Safety is non-negotiable. Without explicit confirmation for dangerous commands (rm, mv, sudo, etc.), users risk catastrophic data loss. This must be in place before the shell can be trusted for production use.

**Independent Test**: Request the AI to perform a file deletion (e.g., "delete test.txt"). Verify a centered modal dialog appears requiring user choice before execution proceeds.

**Acceptance Scenarios**:

1. **Given** the AI generates a command containing dangerous patterns (rm, mv, sudo, >, curl -X POST, dd, mkfs), **When** execution is about to proceed, **Then** a modal dialog appears blocking the screen with the dangerous command highlighted
2. **Given** the safety modal is displayed, **When** I review the command and press 1, **Then** the command executes once and the modal disappears
3. **Given** the safety modal is displayed, **When** I press 2, **Then** the command is added to the permanent allowlist and executes immediately, and future identical commands bypass the modal
4. **Given** the safety modal is displayed, **When** I press 3, **Then** the command is denied, execution is cancelled, and I return to the command prompt
5. **Given** a command is in the allowlist, **When** the same command is generated again, **Then** it executes immediately without showing the modal
6. **Given** the modal is visible, **When** I attempt to type other commands or use the shell, **Then** all input is captured only by the modal (modal is truly blocking)

---

### Edge Cases

- **What happens when the terminal window is very small (< 80 columns)?** The split panels should gracefully degrade to a minimum size or switch to a stacked layout to maintain usability
- **How does the system handle rapid consecutive commands while AI is still processing?** Commands should be queued with visual indication, or the input should be disabled until the current command completes
- **What happens if llama.cpp fails to initialize but the log silencing is active?** The application must still surface critical errors to the user through the TUI's error display mechanism, not through raw logs
- **What happens when a command generates very long output (thousands of lines)?** The output area should scroll and provide navigation controls (scrollbar, Page Up/Down) without breaking the layout
- **How does the modal handle commands that are partially dangerous?** (e.g., `ls && rm test.txt`) The modal should appear for the dangerous component and allow users to review the entire compound command
- **What happens if the user forcefully resizes the terminal during modal display?** The modal should remain centered and readable, adjusting its size proportionally

## Requirements *(mandatory)*

### Functional Requirements

#### TUI Framework and Layout

- **FR-001**: System MUST use the ratatui framework (version 0.27) for all terminal UI rendering
- **FR-002**: System MUST display a split-panel layout with left panel occupying 70% width and right panel occupying 30% width
- **FR-003**: System MUST clearly label the left panel as "Command Shell" and the right panel as "AI Analysis" with visual borders
- **FR-004**: System MUST render both panels simultaneously on every frame update
- **FR-005**: System MUST maintain the split-panel layout across terminal resize events

#### Log Silencing

- **FR-006**: System MUST set the environment variable `LLAMA_LOG_LEVEL=0` before initializing any llama.cpp components
- **FR-007**: System MUST suppress all llama.cpp backend initialization logs (model loading, tensor allocation, Metal/GPU messages)
- **FR-008**: System MUST ensure no llama.cpp logs appear during model inference operations
- **FR-009**: System MUST still log critical errors from the AI subsystem through the application's own logging mechanism

#### AI Thinking Animation

- **FR-010**: System MUST display an animated spinner in the right panel when AI processing begins
- **FR-011**: Spinner MUST use Unicode box-drawing characters that cycle through at least 10 distinct frames (⠋, ⠙, ⠹, ⠸, ⠼, ⠴, ⠦, ⠧, ⠇, ⠏)
- **FR-012**: Spinner animation MUST update at a rate of at least 10 frames per second to appear smoothly animated
- **FR-013**: Spinner MUST be accompanied by the text "Thinking..." or similar status message
- **FR-014**: Spinner MUST disappear immediately when AI processing completes

#### AI Analysis Toggle

- **FR-015**: System MUST support toggling the right panel view via the Ctrl+T keyboard shortcut
- **FR-016**: In simple mode (default), the right panel MUST show only the spinner/status or remain empty when idle
- **FR-017**: In detailed mode (toggled), the right panel MUST display the full JSON output from the AI's command generation
- **FR-018**: The toggle state MUST persist across multiple command executions within the same session
- **FR-019**: JSON display MUST be pretty-printed with proper indentation for readability

#### Safety Modal Dialog

- **FR-020**: System MUST detect dangerous command patterns: `rm`, `mv`, `sudo`, `>`, `curl -X POST`, `dd`, `mkfs`
- **FR-021**: System MUST display a centered modal dialog when a dangerous command is detected and not in the allowlist
- **FR-022**: Modal MUST be visually distinct (e.g., red background, warning icon) to indicate danger
- **FR-023**: Modal MUST display the complete dangerous command and its description
- **FR-024**: Modal MUST provide exactly three options: [1] Allow Once, [2] Allow Always, [3] Deny
- **FR-025**: Modal MUST block all other user input until a choice is made
- **FR-026**: System MUST persist the allowlist across application restarts
- **FR-027**: Selecting "Allow Once" MUST execute the command immediately and close the modal without modifying the allowlist
- **FR-028**: Selecting "Allow Always" MUST add the command to the persistent allowlist and execute it immediately
- **FR-029**: Selecting "Deny" MUST cancel execution, close the modal, and return to the command prompt

#### Input and Navigation

- **FR-030**: System MUST accept text input in the left panel's command area
- **FR-031**: System MUST execute the command when the user presses Enter
- **FR-032**: System MUST allow Backspace to delete characters from the input
- **FR-033**: System MUST exit the application cleanly when Ctrl+C is pressed
- **FR-034**: System MUST maintain command history visible in the left panel
- **FR-035**: Output area MUST support scrolling for outputs longer than the visible area

### Key Entities

- **TUI Application State**: Represents the entire application's UI state including current input, command history, output buffer, AI analysis content, spinner state, modal state, and toggle preferences

- **Modal Dialog**: Represents the safety confirmation dialog including the dangerous command text, description, selected option index, and visibility state

- **Allowlist**: Represents the persistent storage of user-approved dangerous commands, stored as a collection of command strings that bypass safety confirmations

- **Command Sequence**: Represents the AI-generated structured output containing task description and array of commands with descriptions (already exists in the codebase as `CommandSequence` type)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users launching the application see zero llama.cpp initialization logs in the terminal output (100% elimination of verbose logging)

- **SC-002**: Users entering natural language commands see the thinking animation begin within 100ms of pressing Enter (immediate visual feedback)

- **SC-003**: The spinner animation cycles smoothly at a rate perceivable as continuous motion (minimum 10 FPS, target 30 FPS)

- **SC-004**: Users executing dangerous commands receive a safety confirmation dialog 100% of the time for non-allowlisted commands (zero bypasses without confirmation)

- **SC-005**: Power users can access detailed AI analysis within 1 second by pressing Ctrl+T (toggle responds instantly)

- **SC-006**: The split-panel layout remains stable and readable across terminal resize operations with no visual artifacts or crashes

- **SC-007**: Users can add commands to the allowlist and see them bypass confirmation on subsequent executions within the same session and across application restarts (persistent allowlist)

- **SC-008**: The modal dialog remains centered and readable for terminal widths from 80 to 200 columns

- **SC-009**: Users can complete the full workflow (enter command → see thinking animation → review safety modal → execute) within 10 seconds for typical operations

- **SC-010**: System maintains responsive UI interactions (key press to visual update) with latency under 50ms even during AI processing

## Assumptions

- The ratatui framework (v0.27) is compatible with the current Rust toolchain and crossterm version already in the project
- The llama.cpp library respects the `LLAMA_LOG_LEVEL` environment variable for log control
- Terminal emulators support the required Unicode box-drawing characters for the spinner animation
- Users have terminals with at least 80 columns width for minimum viable layout
- The existing `CommandSequence` structure from the parser module is sufficient for JSON display in the detailed view
- The allowlist can be stored in a simple file-based format (e.g., JSON or line-delimited text) in the user's config directory
- Command detection patterns (rm, mv, sudo, etc.) are sufficient to catch the majority of dangerous operations without false positives
- The existing `SafeExecutor` can be refactored to support asynchronous modal interaction without blocking the event loop

## Out of Scope

- Command history navigation with arrow keys (Up/Down to recall previous commands)
- Syntax highlighting or color coding of command output
- Mouse interaction support (clicking, scrolling, selecting text)
- Multiple concurrent command execution or task management
- Custom user themes or color schemes for the TUI
- Integration with external terminal multiplexers (tmux, screen)
- Network-based remote TUI access
- Accessibility features (screen reader support, high contrast modes)
- Internationalization of UI labels and messages
- Export or logging of command history to files
- Search functionality within command history or output

## Dependencies

- Existing `AIManager` with `generate_response` method that returns `CommandSequence`
- Existing `SafeExecutor` with `execute_sequence` method for command execution
- Existing `CommandSequence` and `Command` types from the parser module
- Existing configuration system for loading settings (to be extended for allowlist storage)
- ratatui crate version 0.27 (to be added to Cargo.toml)
- crossterm crate version 0.27 (already present)
- tokio async runtime (already present)

## Clarifications

*No clarifications required - all aspects have reasonable defaults based on the detailed implementation plan provided in the source document.*
