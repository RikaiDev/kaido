# Tasks: Shell Logging Improvement

**Feature**: Shell Logging Improvement  
**Branch**: `005-shell-logging-improvement`  
**Created**: 2025-01-23  
**Total Tasks**: 15

## Implementation Strategy

**MVP Scope**: User Story 1 (Reduce Verbose Startup Logging) - delivers immediate user value  
**Approach**: Incremental delivery with independent user stories  
**Testing**: Unit tests for configuration and logging level changes  
**Dependencies**: Minimal - leverages existing shell and config infrastructure

## Phase 1: Setup & Infrastructure

### Project Setup Tasks

- [x] T001 Create LogLevel enum in src/config.rs
- [x] T002 Add LoggingConfiguration struct to src/config.rs
- [x] T003 Add UserSession struct to src/shell/state.rs
- [x] T004 Update default.toml with logging configuration schema

## Phase 2: Foundational Tasks

### Core Configuration Infrastructure

- [x] T005 [P] Implement LoggingConfiguration::load() method in src/config.rs
- [x] T006 [P] Implement LoggingConfiguration::save() method in src/config.rs
- [x] T007 [P] Implement UserSession::new() method in src/shell/state.rs
- [x] T008 [P] Add XDG config directory support to src/config.rs

## Phase 3: User Story 1 - Reduce Verbose Startup Logging (P1)

**Goal**: Reduce startup output from 15+ lines to 2 lines maximum  
**Independent Test**: Shell startup displays no more than 2 lines of output by default

### Implementation Tasks

- [x] T009 [US1] Modify startup logging in src/main.rs to use LogLevel::Normal
- [x] T010 [US1] Update shell initialization in src/shell/repl.rs to suppress verbose messages
- [x] T011 [US1] Implement welcome message display in src/shell/prompt.rs
- [x] T012 [US1] Add startup message configuration loading in src/shell/repl.rs

## Phase 4: User Story 2 - Provide Clear User Prompt (P1)

**Goal**: Display concise welcome message with usage guidance  
**Independent Test**: 95% of new users can understand how to interact within 5 seconds of startup

### Implementation Tasks

- [x] T013 [US2] Create welcome message template in src/shell/prompt.rs
- [x] T014 [US2] Integrate welcome message with shell startup in src/shell/repl.rs

## Phase 5: User Story 3 - Configurable Logging Levels (P2)

**Goal**: Implement builtin commands for logging level control  
**Independent Test**: Users can access detailed logging information in under 3 seconds when needed

### Implementation Tasks

- [x] T015 [US3] Implement `set` builtin command in src/shell/executor.rs
- [x] T016 [US3] Implement `unset` builtin command in src/shell/executor.rs
- [x] T017 [US3] Implement `status` builtin command in src/shell/executor.rs
- [x] T018 [US3] Add builtin command registration in src/shell/mod.rs

## Phase 6: Polish & Cross-Cutting Concerns

### Testing & Validation

- [x] T019 Create unit tests for LoggingConfiguration in tests/unit/config_tests.rs
- [x] T020 Create unit tests for UserSession in tests/unit/logging_tests.rs
- [x] T021 Create integration tests for builtin commands in tests/integration/shell_tests.rs
- [x] T022 Add configuration file validation tests in tests/unit/config_tests.rs

### Documentation & Cleanup

- [x] T023 Update README.md with new logging configuration options
- [x] T024 Add builtin command documentation to help system
- [x] T025 Verify zero compilation warnings across all modified files

## Dependencies

### User Story Completion Order

1. **User Story 1** (P1) - Can be implemented independently after Phase 2
2. **User Story 2** (P1) - Depends on User Story 1 for welcome message integration
3. **User Story 3** (P2) - Depends on Phase 2 configuration infrastructure

### Parallel Execution Opportunities

**Phase 2**: Tasks T005-T008 can be implemented in parallel (different files)  
**Phase 3**: Tasks T009-T012 can be implemented in parallel (different components)  
**Phase 5**: Tasks T015-T017 can be implemented in parallel (different commands)

## Success Criteria Validation

- **SC-001**: ✅ T009-T012 ensure startup output ≤ 2 lines
- **SC-002**: ✅ T013-T014 provide clear user guidance
- **SC-003**: ✅ T015-T017 enable quick logging level access
- **SC-004**: ✅ T005-T006 ensure immediate configuration changes
- **SC-005**: ✅ All tasks maintain debugging capability while reducing default verbosity

## MVP Delivery Plan

**Phase 1-3**: Deliver User Story 1 (clean startup) - immediate user value  
**Phase 4**: Add User Story 2 (clear prompt) - enhanced usability  
**Phase 5**: Add User Story 3 (logging control) - advanced features  
**Phase 6**: Polish and testing - production readiness
