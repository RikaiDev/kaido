# Implementation Tasks: Professional TUI Interface for Kaido AI Shell

**Feature**: 006-tui-interface  
**Branch**: `006-tui-interface`  
**Generated**: 2025-10-24

## Overview

This feature transforms Kaido AI Shell from a basic text REPL into a professional TUI application. Tasks are organized by user story to enable independent, incremental delivery following MVP-first principles.

**Total Tasks**: 47  
**User Stories**: 5 (3 P1, 1 P2, 1 P1)  
**Parallel Opportunities**: 23 tasks can be parallelized  
**Suggested MVP**: User Story 1 + User Story 3 (log silencing + basic TUI layout)

---

## Phase 1: Setup & Infrastructure

**Goal**: Prepare project for TUI development

- [x] T001 Add ratatui dependency to Cargo.toml (version 0.27)
- [x] T002 Create src/ui/ module directory structure
- [x] T003 Create src/ui/mod.rs with module exports
- [x] T004 Verify compilation with zero warnings per constitution

**Duration**: ~10 minutes  
**Blocks**: All subsequent phases  
**Test**: `cargo build` succeeds with zero warnings

---

## Phase 2: Foundational - Terminal Management

**Goal**: Implement RAII terminal cleanup pattern (blocks all user stories)

- [x] T005 Create TerminalGuard struct in src/ui/app.rs for raw mode management
- [x] T006 Implement Drop trait for TerminalGuard to restore terminal
- [x] T007 Add custom panic hook in src/main.rs for emergency terminal restoration
- [ ] T008 Test terminal cleanup with intentional panic (manual test)

**Duration**: ~20 minutes  
**Blocks**: All user stories (terminal must be manageable before TUI can run)  
**Test**: Force panic with `panic!("test")`, verify terminal restored

---

## Phase 3: User Story 1 - Clean Shell Startup Experience (P1)

**Story Goal**: Silence llama.cpp verbose logging for professional appearance

**Independent Test**: Run `cargo run`, verify zero llama.cpp initialization logs appear

**Why First**: This is a quick win that immediately improves user experience and has zero dependencies on other stories.

### Tasks

- [x] T009 [P] [US1] Set LLAMA_LOG_LEVEL=0 environment variable in src/main.rs before any llama operations
- [x] T010 [P] [US1] Add LLAMA_LOG_LEVEL=0 in src/ai/mod.rs run_llama_cpp_inference() as defense-in-depth
- [x] T011 [US1] Remove or fix unused import warning in src/ai/mod.rs (Command import)
- [ ] T012 [US1] Manual test: cargo run, type "list files", verify NO llama logs

**Duration**: ~15 minutes  
**Dependencies**: Phase 1, Phase 2  
**Parallel**: T009 and T010 can run in parallel (different logical sections)  
**Deliverable**: Application launches silently, only showing intended UI

---

## Phase 4: User Story 3 - Split-Panel Layout (P1)

**Story Goal**: Display left/right split panels (70/30) for commands and AI analysis

**Independent Test**: Launch app, verify two bordered panels side-by-side with correct proportions

**Why Second**: Provides the foundational UI structure that other stories build upon

### Tasks

#### Layout Implementation

- [x] T013 [P] [US3] Create src/ui/layout.rs with create_layout() function (70/30 split)
- [x] T014 [P] [US3] Create src/ui/app.rs with KaidoApp state struct (9 fields per data-model.md)
- [x] T015 [US3] Add AppState enum to src/ui/app.rs (Normal/ModalActive/Executing states)
- [x] T016 [US3] Implement KaidoApp::new() constructor with default values
- [x] T017 [US3] Implement toggle_ai_panel() method in KaidoApp
- [x] T018 [US3] Implement next_spinner_frame() method in KaidoApp

#### REPL Integration

- [x] T019 [US3] Refactor src/shell/repl.rs to replace KaidoREPL struct with TUI fields (app, terminal, ai_manager, safe_executor)
- [x] T020 [US3] Implement KaidoREPL::new() with terminal initialization using TerminalGuard
- [x] T021 [US3] Implement render_ui() method in src/shell/repl.rs with basic left/right panel rendering
- [x] T022 [US3] Implement event loop in KaidoREPL::run() with crossterm::event::poll (100ms timeout)
- [x] T023 [US3] Add Ctrl+C handler in event loop to break and cleanup
- [x] T024 [US3] Add Ctrl+T handler in event loop to toggle AI panel view
- [x] T025 [US3] Add text input handlers (Char, Backspace) to update KaidoApp.input
- [x] T026 [US3] Update src/main.rs to initialize REPL with TUI mode

#### Testing

- [ ] T027 [US3] Manual test: cargo run, verify left panel shows "Command Shell" title
- [ ] T028 [US3] Manual test: Type characters, verify they appear in input area
- [ ] T029 [US3] Manual test: Resize terminal, verify panels maintain 70/30 ratio
- [ ] T030 [US3] Manual test: Press Ctrl+C, verify clean exit and terminal restored

**Duration**: ~90 minutes  
**Dependencies**: Phase 1, Phase 2  
**Parallel**: T013 and T014 can run in parallel (different files)  
**Deliverable**: Working TUI with split panels, keyboard input, clean exit

---

## Phase 5: User Story 2 - Visual AI Thinking Process (P1)

**Story Goal**: Display animated spinner while AI processes commands

**Independent Test**: Type command, press Enter, verify spinner animates smoothly until completion

**Why Third**: Depends on split-panel layout (US3) to have a place to render

### Tasks

#### Spinner Implementation

- [x] T031 [P] [US2] Create src/ui/spinner.rs with SPINNER_FRAMES constant (10 Unicode frames)
- [x] T032 [P] [US2] Implement get_spinner_frame(index) function in src/ui/spinner.rs
- [x] T033 [US2] Add spinner rendering logic to render_ui() in src/shell/repl.rs (right panel)
- [x] T034 [US2] Add spinner frame update in KaidoREPL::run() event loop when ai_thinking=true

#### AI Integration

- [x] T035 [US2] Implement process_input() method in src/shell/repl.rs (async)
- [x] T036 [US2] Set ai_thinking=true before AI call, false after in process_input()
- [x] T037 [US2] Call ai_manager.generate_response() in process_input()
- [x] T038 [US2] Store AI output JSON in KaidoApp.ai_output for toggle view
- [x] T039 [US2] Add Enter key handler in event loop to call process_input()

#### Testing

- [ ] T040 [US2] Manual test: Type "list files", press Enter, verify spinner appears immediately
- [ ] T041 [US2] Manual test: Verify spinner cycles through all 10 frames smoothly
- [ ] T042 [US2] Manual test: Verify spinner stops when AI completes
- [ ] T043 [US2] Manual test: Type command while AI thinking, verify input queued/disabled

**Duration**: ~60 minutes  
**Dependencies**: Phase 3 (US3), Phase 4 (layout must exist)  
**Parallel**: T031 and T032 can run in parallel (spinner.rs independent of REPL integration)  
**Deliverable**: Animated spinner provides visual feedback during AI processing

---

## Phase 6: User Story 5 - Safety Confirmation Modal (P1)

**Story Goal**: Show modal dialog for dangerous commands with allow/deny options

**Independent Test**: Type "delete test.txt", verify centered modal appears with 3 options

**Why Fourth**: Critical safety feature, depends on event handling from US3

### Tasks

#### Modal UI

- [x] T044 [P] [US5] Create src/ui/modal.rs with ModalDialog struct (command, description, selected_option)
- [x] T045 [P] [US5] Implement ModalDialog::render() method with centered_rect() helper
- [x] T046 [P] [US5] Style modal with red background and warning icon in render()
- [x] T047 [US5] Add modal rendering to render_ui() in src/shell/repl.rs (overlay on top)

#### Allowlist Management

- [x] T048 [US5] Extend src/safety/allowlist.rs with load() method (from ~/.config/kaido/allowlist.txt)
- [x] T049 [US5] Implement save() method in allowlist.rs (atomic write)
- [x] T050 [US5] Implement add() method in allowlist.rs (insert + save)
- [x] T051 [US5] Implement is_allowed() method in allowlist.rs (HashSet lookup)

#### Safety Executor Integration

- [x] T052 [US5] Add is_dangerous() method to SafeExecutor in src/safety/executor.rs (check patterns: rm, mv, sudo, >, curl -X POST, dd, mkfs)
- [x] T053 [US5] Refactor execute_sequence() in SafeExecutor to check commands before execution
- [x] T054 [US5] Show modal in process_input() when dangerous command detected
- [x] T055 [US5] Add modal event handlers in event loop (keys 1/2/3)
- [x] T056 [US5] Implement option 1 (Allow Once): execute without adding to allowlist
- [x] T057 [US5] Implement option 2 (Allow Always): add to allowlist then execute
- [x] T058 [US5] Implement option 3 (Deny): cancel execution, clear modal

#### State Machine

- [x] T059 [US5] Add AppState transitions in event loop (Normal ↔ ModalActive)
- [x] T060 [US5] Block all non-modal input when state=ModalActive
- [x] T061 [US5] Initialize allowlist in KaidoREPL::new()

#### Testing

- [ ] T062 [US5] Manual test: Type "delete file.txt", verify modal appears
- [ ] T063 [US5] Manual test: Press 1, verify command executes once
- [ ] T064 [US5] Manual test: Press 2 on "rm test", restart app, verify bypasses modal
- [ ] T065 [US5] Manual test: Press 3, verify command cancelled
- [ ] T066 [US5] Manual test: While modal visible, press other keys, verify ignored

**Duration**: ~90 minutes  
**Dependencies**: Phase 4 (US3 for event handling), Phase 5 (US2 for AI integration)  
**Parallel**: T044-T046 (modal UI) and T048-T051 (allowlist) can run in parallel  
**Deliverable**: Dangerous commands require explicit user confirmation with persistent allowlist

---

## Phase 7: User Story 4 - Toggle AI Analysis Detail View (P2)

**Story Goal**: Press Ctrl+T to switch between spinner and JSON view

**Independent Test**: Process command, press Ctrl+T, verify JSON appears; press again, verify spinner returns

**Why Last P2**: Enhancement feature, depends on spinner (US2) and basic layout (US3)

### Tasks

- [x] T067 [US4] Verify Ctrl+T handler already exists in event loop (added in T024)
- [x] T068 [US4] Update right panel rendering logic to check ai_panel_toggle flag
- [x] T069 [US4] When toggle=true, display KaidoApp.ai_output (JSON string)
- [x] T070 [US4] When toggle=false, display spinner or "Thinking..." text
- [x] T071 [US4] Ensure toggle state persists across commands (already in KaidoApp)
- [ ] T072 [US4] Manual test: Process command, press Ctrl+T during thinking, verify JSON appears
- [ ] T073 [US4] Manual test: Press Ctrl+T again, verify returns to spinner view
- [ ] T074 [US4] Manual test: With JSON view active, execute new command, verify stays in JSON mode
- [ ] T075 [US4] Manual test: Verify JSON is pretty-printed with indentation

**Duration**: ~30 minutes  
**Dependencies**: Phase 5 (US2 spinner must exist), Phase 4 (US3 layout must exist)  
**Parallel**: All tasks sequential (modify same render logic)  
**Deliverable**: Power users can inspect AI's JSON reasoning by toggling view

---

## Phase 8: Polish & Cross-Cutting Concerns

**Goal**: Fix warnings, optimize, finalize for production

- [x] T076 Remove unused execute_command() method in src/shell/executor.rs (warning fix)
- [x] T077 Remove or use error_output field in src/utils/mod.rs CommandExecution struct (warning fix)
- [x] T078 Add bounds checking for KaidoApp.input (max 1024 chars per data-model.md)
- [x] T079 Add bounds checking for KaidoApp.history (max 1000 entries, FIFO eviction)
- [x] T080 Add output truncation for KaidoApp.output (max 100KB per data-model.md)
- [x] T081 Add error handling for allowlist file permissions (show user-friendly error in TUI)
- [ ] T082 Test terminal resize edge cases (< 80 columns, very wide, rapid resizes)
- [ ] T083 Test rapid consecutive commands (ensure queueing or input blocking works)
- [ ] T084 Test very long command output (thousands of lines, verify scrolling)
- [x] T085 Final compilation check: cargo build with ZERO warnings
- [ ] T086 Final manual test: Complete full user workflow end-to-end

**Duration**: ~45 minutes  
**Dependencies**: All user story phases complete  
**Deliverable**: Production-ready TUI with zero warnings and robust error handling

---

## Dependency Graph: User Story Completion Order

```
Phase 1 (Setup) → Phase 2 (Terminal Guard)
                        ↓
        ┌───────────────┼───────────────┐
        ↓               ↓               ↓
    [US1: Logs]    [US3: Layout]   (independent)
        ↓               ↓
        └───────────────┼───────────────┐
                        ↓               ↓
                    [US2: Spinner]  [US5: Modal]
                        ↓               ↓
                        └───────────────┤
                                        ↓
                                    [US4: Toggle]
                                        ↓
                                    Phase 8 (Polish)
```

**Critical Path**: Phase 1 → Phase 2 → US3 → US2 → US4 (longest chain)

**Independent Stories**: US1 (log silencing) can be done anytime after Phase 2

---

## Parallel Execution Opportunities

### Within US1 (Log Silencing)
```bash
# T009 and T010 - different files, no conflicts
git checkout -b feature/us1-log-silencing
# Developer A: T009 (main.rs)
# Developer B: T010 (ai/mod.rs)
# Merge both, then T011-T012
```

### Within US3 (Layout)
```bash
# T013 (layout.rs) and T014 (app.rs) - different files
git checkout -b feature/us3-layout
# Developer A: T013-layout
# Developer B: T014-app
# Then sequential: T015-T030
```

### Within US2 (Spinner)
```bash
# T031-T032 (spinner.rs) parallel with T033-T039 (REPL integration)
# Developer A: spinner.rs implementation
# Developer B: REPL integration (needs app.rs from US3)
```

### Within US5 (Modal)
```bash
# Three parallel tracks:
# Track A: T044-T047 (modal UI)
# Track B: T048-T051 (allowlist)
# Track C: T052-T054 (executor logic)
# Then merge for T055-T066 (integration + testing)
```

**Total Parallelizable**: 23 tasks marked with [P] can run concurrently

---

## Implementation Strategy

### MVP Scope (Week 1)
**Goal**: Bare minimum working TUI with safety

**Include**:
- Phase 1: Setup
- Phase 2: Terminal Guard
- US1: Log Silencing (quick win)
- US3: Split-Panel Layout (foundation)
- US5: Safety Modal (critical for production use)

**Exclude** (defer to Week 2):
- US2: Spinner (nice-to-have feedback)
- US4: Toggle View (power user feature)
- Phase 8: Polish (iterative refinement)

**MVP Deliverable**: User can launch clean TUI, type commands, see results, get safety confirmations for dangerous commands.

### Incremental Delivery Strategy

**Sprint 1 (MVP)**: T001-T030, T044-T066 (~3-4 hours)
- Setup + Terminal + US1 + US3 + US5
- Deliverable: Safe, functional TUI shell

**Sprint 2 (Enhancement)**: T031-T043, T067-T075 (~2 hours)
- US2 + US4
- Deliverable: Polished UX with spinner and toggle

**Sprint 3 (Production)**: T076-T086 (~1 hour)
- Phase 8: Polish
- Deliverable: Zero warnings, edge cases handled

### Testing Approach

**Manual Testing Required** (per constitution's Manual Development Process):
- Each user story has 4-6 manual test tasks
- Run tests immediately after story completion
- No automated test scripts allowed

**Test Phases**:
1. After US1: Verify clean startup
2. After US3: Verify layout and input
3. After US2: Verify spinner animation
4. After US5: Verify modal interactions and allowlist
5. After US4: Verify toggle functionality
6. Final: Complete end-to-end workflow

---

## Task Checklist Summary

**Format Compliance**: ✅ All 86 tasks follow required format  
- ✅ All have checkbox `- [ ]`
- ✅ All have Task ID (T001-T086)
- ✅ Story tasks have [US#] label (US1-US5)
- ✅ Parallelizable tasks have [P] marker
- ✅ All have clear file paths or test descriptions

**Coverage**:
- ✅ US1: 4 tasks (T009-T012)
- ✅ US2: 13 tasks (T031-T043)
- ✅ US3: 18 tasks (T013-T030)
- ✅ US4: 9 tasks (T067-T075)
- ✅ US5: 23 tasks (T044-T066)
- ✅ Setup: 4 tasks (T001-T004)
- ✅ Foundation: 4 tasks (T005-T008)
- ✅ Polish: 11 tasks (T076-T086)

**Ready for Execution**: ✅ `/speckit.implement` can now process these tasks sequentially

---

## Notes

- **Constitution Compliance**: All tasks follow manual development process (no batch scripts)
- **Zero Warnings**: T085 ensures compliance with code quality standards
- **Real Implementation**: No mock/fake tasks per constitution principle VII
- **Independent Stories**: Each user story is testable standalone (per spec.md requirements)
- **MVP-First**: US1+US3+US5 provide minimal viable product
- **Incremental Value**: Each phase delivers working increment

