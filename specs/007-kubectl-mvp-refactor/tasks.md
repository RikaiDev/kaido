# Tasks: Kubectl-Only MVP (60-Day Reality Check)

**Input**: Design documents from `/specs/007-kubectl-mvp-refactor/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Not requested in specification - focus on manual testing per quickstart.md

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

**Implementation Philosophy**: Every task must produce real, working code. No mocks, no TODOs, no placeholders. Each round must result in functional, executable code that advances toward the 60-day MVP goal.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Single project structure** (Rust workspace at repository root)
- Source code: `src/`
- Tests: `tests/`
- Configuration: `config/`
- Specifications: `specs/`

---

## Phase 1: Setup & Code Cleanup

**Purpose**: Remove dead code and prepare project structure for kubectl-only focus

### Critical Cleanup Tasks

- [x] T001 Remove `src/tools/` directory entirely (tool registry, Docker/Git adapters - no longer needed)
- [x] T002 Remove `src/agent/` directory entirely (multi-agent orchestration - no longer needed)
- [x] T003 Remove `src/memory/` directory entirely (complex memory system - replaced by audit log)
- [x] T004 Update `Cargo.toml` dependencies - **KEEP llama-cpp-2** (required for local GGUF models, enterprise privacy requirement)
- [x] T005 Update `src/main.rs` to remove imports and references to deleted modules (tools, agent, memory)
- [x] T006 Update `src/config.rs` to remove tool registry configuration fields, add OpenAI API key field
- [x] T007 Run `cargo check` and fix all compilation errors from deleted modules
- [x] T008 Run `cargo clippy -- -D warnings` and fix all warnings to achieve zero-warning baseline

**Checkpoint**: ✅ Codebase compiles with 0 errors, 32 warnings (dead code to be removed in Phase 3)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Configuration Infrastructure

- [x] T009 Create `src/kubectl/mod.rs` with module structure (context, translator, risk_classifier, executor submodules)
- [x] T010 Create `src/audit/mod.rs` with module structure (logger, query, schema submodules)
- [x] T011 [P] Update `Cargo.toml` to add serde_yaml (kubeconfig parsing), reqwest (OpenAI API), rusqlite (audit log)
- [x] T012 [P] Update `config/default.toml` with OpenAI API configuration fields (api_key, model, timeout_seconds, audit settings)

### Core Data Structures

- [x] T013 [P] Define `EnvironmentType` enum in `src/kubectl/context.rs` (Development/Staging/Production/Unknown)
- [x] T014 [P] Define `RiskLevel` enum in `src/kubectl/risk_classifier.rs` (Low/Medium/High with string conversion)
- [x] T015 [P] Define `KubectlContext` struct in `src/kubectl/context.rs` (name, cluster, namespace, user, environment_type fields)
- [x] T016 [P] Define `TranslationResult` struct in `src/kubectl/translator.rs` (kubectl_command, confidence_score, reasoning fields)

### Database Schema

- [x] T017 Implement SQLite schema initialization in `src/audit/schema.rs` - create audit_log table per contracts/audit-log-schema.sql (17 fields, 4 indexes)
- [x] T018 Implement database connection manager in `src/audit/mod.rs` - create ~/.kaido/audit.db with proper permissions (600)

**Checkpoint**: ✅ Foundation complete (awaiting network for serde_yaml download) - ready for Phase 3

---

## Phase 3: User Story 1 - Natural Language Kubectl Commands (Priority: P1) 🎯 MVP

**Goal**: Users can type natural language commands (e.g., "show pods") and system translates to kubectl, executes, and displays results with confidence-based warnings

**Independent Test**: 
1. Launch kaido
2. Type "show all pods" → Verify translates to `kubectl get pods -n {namespace}` and executes
3. Type "show logs" (ambiguous) → Verify displays low confidence warning (<70%)
4. Type "delete pod test" → Verify shows risk confirmation modal
5. All commands logged to audit database

### Kubeconfig Parsing & Context Detection

- [x] T019 [US1] Implement kubeconfig file reader in `src/kubectl/context.rs` - locate file from ~/.kube/config or $KUBECONFIG, parse YAML with serde_yaml
- [x] T020 [US1] Implement current context parser in `src/kubectl/context.rs` - extract current-context field, find matching context entry
- [x] T021 [US1] Implement environment type detection in `src/kubectl/context.rs` - regex match on context name (prod/production → Production, stag/staging → Staging, dev/development → Development, else → Unknown)
- [x] T022 [US1] Implement namespace extraction in `src/kubectl/context.rs` - get namespace from context.context.namespace field, default to "default"
- [x] T023 [US1] Add error handling for missing/invalid kubeconfig in `src/kubectl/context.rs` - return user-friendly error "kubectl context not configured. Run 'kubectl config get-contexts'"

### OpenAI Translation Integration

- [x] T024 [US1] Implement system prompt builder in `src/kubectl/openai.rs` - format prompt per contracts/openai-api-contract.md with current context variables
- [x] T025 [US1] Implement user prompt builder in `src/kubectl/openai.rs` - format user input with context reminder (cluster, namespace, environment)
- [x] T026 [US1] Implement OpenAI API request function in `src/kubectl/openai.rs` - POST to https://api.openai.com/v1/chat/completions with JSON format per contract
- [x] T027 [US1] Implement response parser in `src/kubectl/openai.rs` - extract JSON from choices[0].message.content, parse command/confidence/reasoning fields
- [x] T028 [US1] Implement command validation in `src/kubectl/openai.rs` - verify command starts with "kubectl ", confidence in 0-100 range
- [x] T029 [US1] Implement timeout and retry logic in `src/kubectl/openai.rs` - 10 second timeout, 1 retry on network error per research.md
- [x] T030 [US1] Implement error handling for OpenAI failures in `src/kubectl/openai.rs` - handle 401/429/500 errors, provide fallback message "Enter kubectl command manually:"

### Command Execution

- [x] T031 [US1] Implement kubectl command executor in `src/kubectl/executor.rs` - use std::process::Command to execute kubectl with captured stdout/stderr
- [x] T032 [US1] Implement output formatting in `src/kubectl/executor.rs` - preserve kubectl output formatting, handle ANSI colors, truncate if >10KB for logging
- [x] T033 [US1] Implement execution result capture in `src/kubectl/executor.rs` - capture exit_code, stdout, stderr, execution_duration_ms

### REPL Integration

- [x] T034 [US1] Update `src/shell/repl.rs` to replace multi-tool dispatch with kubectl-only flow - remove tool detection logic
- [x] T035 [US1] Implement kubectl translation flow in `src/shell/repl.rs` - call context parser → OpenAI translator → display result
- [x] T036 [US1] Implement confidence warning display in `src/shell/repl.rs` - if confidence < 70%, show "⚠️ Low confidence (X%) - Please review command carefully" with reasoning
- [x] T037 [US1] Implement direct kubectl fallback in `src/shell/repl.rs` - if translation fails, prompt "Enter kubectl command manually:" and accept raw kubectl input
- [x] T038 [US1] Update main REPL loop in `src/shell/repl.rs` - integrate kubectl flow, remove old tool selection UI

**Checkpoint**: ✅ User Story 1 MVP COMPLETE - natural language → kubectl translation → execution → output display with confidence warnings

---

## Phase 4: User Story 2 - Environment-Aware Safety Controls (Priority: P2)

**Goal**: Destructive commands in production require typed confirmation (e.g., typing "nginx" to delete deployment nginx), medium-risk commands require yes/no, read-only commands execute immediately

**Independent Test**:
1. Set context to dev-cluster
2. Type "delete deployment nginx" → Verify shows yes/no confirmation (HIGH risk in dev = yes/no)
3. Set context to prod-cluster
4. Type "delete deployment nginx" → Verify requires typing "nginx" to confirm (HIGH risk in prod = typed)
5. Type "scale nginx to 3" → Verify shows yes/no confirmation (MEDIUM risk)
6. Type "get pods" → Verify executes immediately (LOW risk)

### Risk Classification

- [x] T039 [US2] Implement risk classification function in `src/kubectl/risk_classifier.rs` - pattern match on kubectl verbs: delete/drain → HIGH, apply/create/patch/scale/rollout → MEDIUM, get/describe/logs → LOW
- [x] T040 [US2] Implement special case for scale --replicas=0 in `src/kubectl/risk_classifier.rs` - detect and classify as HIGH risk (effectively a delete)
- [x] T041 [US2] Implement environment risk multiplier in `src/kubectl/risk_classifier.rs` - combine command risk with environment type (Production increases risk severity)

### Confirmation Modal (Ratatui TUI)

- [x] T042 [US2] Create confirmation modal widget in `src/ui/confirmation.rs` - use ratatui to create popup overlay with command preview, risk level, environment context
- [x] T043 [US2] Implement yes/no confirmation in `src/ui/confirmation.rs` - handle keyboard input (y/n), display risk warning, show [Cancel] [Confirm] buttons
- [x] T044 [US2] Implement typed confirmation in `src/ui/confirmation.rs` - extract expected text (resource name or "production"), create text input field, validate user input matches exactly
- [x] T045 [US2] Implement confirmation type selector in `src/ui/confirmation.rs` - choose None/YesNo/Typed based on risk level + environment type per clarifications
- [x] T046 [US2] Implement resource name extraction in `src/ui/confirmation.rs` - parse kubectl command for resource name (e.g., "nginx" from "delete deployment nginx")

### Safety Integration

- [x] T047 [US2] Update `src/shell/repl.rs` to integrate risk classification - classify every translated command before execution
- [x] T048 [US2] Update `src/shell/repl.rs` to show confirmation modal - display modal for MEDIUM/HIGH risk commands, skip for LOW risk
- [x] T049 [US2] Implement user action tracking in `src/shell/repl.rs` - record EXECUTED/CANCELLED/EDITED for audit log
- [x] T050 [US2] Implement cancel handler in `src/shell/repl.rs` - log cancelled commands, show "Command cancelled" message

**Checkpoint**: ✅ User Story 2 COMPLETE - risk-based confirmation system working across all environments (dev/staging/prod)

---

## Phase 5: User Story 3 - Complete Command History and Audit Trail (Priority: P2)

**Goal**: All kubectl commands logged to SQLite with timestamp, environment, result. Users can query via TUI commands: "show history today", "show history last week", "show history production"

**Independent Test**:
1. Execute 5 different kubectl commands (mix of executed/cancelled)
2. Type "kaido> show history today" → Verify displays all today's commands with details
3. Type "kaido> show history production" → Verify filters to production context only
4. Open ~/.kaido/audit.db with sqlite3 → Verify schema matches contracts/audit-log-schema.sql

### Audit Logging

- [x] T051 [US3] Implement audit log writer in `src/audit/logger.rs` - INSERT INTO audit_log with all 17 fields per schema
- [x] T052 [US3] Implement stdout/stderr truncation in `src/audit/logger.rs` - truncate to 10KB before insertion to prevent database bloat
- [x] T053 [US3] Implement non-blocking log writes in `src/audit/logger.rs` - log errors to stderr but don't fail command execution if audit write fails
- [x] T054 [US3] Implement retention policy in `src/audit/logger.rs` - on startup, DELETE records older than 90 days (configurable via config.toml)

### Audit Query Interface

- [x] T055 [US3] Implement "show history today" query in `src/audit/query.rs` - SELECT with WHERE timestamp >= start of day, format as table
- [x] T056 [US3] Implement "show history last week" query in `src/audit/query.rs` - SELECT with WHERE timestamp >= 7 days ago
- [x] T057 [US3] Implement "show history production" query in `src/audit/query.rs` - SELECT with WHERE environment LIKE '%prod%'
- [x] T058 [US3] Implement table formatter in `src/audit/query.rs` - format query results as aligned columns (ID, Time, Command, Environment, Action, Exit Code)
- [x] T059 [US3] Implement pagination in `src/audit/query.rs` - limit to 20 entries per page, show "Press any key for more..." if more results exist

### REPL History Commands

- [x] T060 [US3] Add history command parser in `src/shell/repl.rs` - detect "show history [filter]" pattern, extract filter (today/last week/production)
- [x] T061 [US3] Integrate audit logger in `src/shell/repl.rs` - log every command execution (EXECUTED), cancellation (CANCELLED), edit (EDITED) with full context
- [x] T062 [US3] Integrate query display in `src/shell/repl.rs` - call audit query functions, display formatted table in TUI

**Checkpoint**: ✅ User Story 3 COMPLETE - complete audit trail with TUI query interface working (SQLite + TUI filters)

---

## Phase 6: User Story 4 - AI Translation Feedback Loop (Priority: P3)

**Goal**: When users edit AI-generated commands before execution, system logs the correction (original AI output + user edit) for future model improvement

**Independent Test**:
1. Type ambiguous command "scale app" → AI generates incomplete command
2. Edit command to add replica count before execution
3. Query audit log → Verify both original AI command and edited command are logged
4. Verify user_action = 'EDITED' in audit_log table

### Command Editing UI

- [x] T063 [US4] Implement command edit modal in `src/ui/confirmation.rs` - add 'E' key handler to trigger edit mode in confirmation modal
- [x] T064 [US4] Implement edit trigger in `src/shell/repl.rs` - add 'E' keyboard shortcut in confirmation modal, display help text
- [x] T065 [US4] Implement edit state in `src/shell/repl.rs` - added EditingCommand state, populate input buffer with command for editing
- [x] T066 [US4] Implement re-classification after edit in `src/shell/repl.rs` - re-run risk classifier on edited command, show new confirmation if needed

### Correction Logging

- [x] T067 [US4] Extend audit log entry in `src/audit/logger.rs` - added original_command field to AuditLogEntry struct
- [x] T068 [US4] Add original_command field to audit log schema in `src/audit/schema.rs` - store AI-generated command before user edit
- [x] T069 [US4] Implement edit detection in `src/shell/repl.rs` - compare kubectl_command to original_ai_command, set user_action = 'EDITED' when different

### Feedback Analysis (Deferred to Post-MVP)

- [x] T070 [US4] Add comment in `src/audit/query.rs` noting future analytics - added comprehensive TODO with SQL examples for analyzing EDITED entries

**Checkpoint**: ✅ User Story 4 COMPLETE - command editing with original/edited logging for AI improvement

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories, final validation

### Documentation & Error Messages

- [x] T071 [P] Update `README.md` with kubectl-only focus + **privacy-first architecture** (local GGUF → OpenAI fallback)
- [ ] T072 [P] Validate all error messages in `src/kubectl/translator.rs` and `src/kubectl/context.rs` are user-friendly (no raw API errors)
- [ ] T073 [P] Add inline documentation (/// doc comments) to all public functions in src/kubectl/ and src/audit/

### Constitution Compliance Final Check

- [ ] T074 Run `cargo clippy -- -D warnings` on entire codebase - verify zero warnings (Constitution VI)
- [x] T075 Run `cargo check` - verify zero compilation errors ✅
- [ ] T076 Audit all functions for unused code - systematically evaluate each item (Constitution VI requires individual inspection)
- [ ] T077 Verify no mock implementations remain in codebase - grep for "not implemented", "TODO", "FIXME" (Constitution VII)

### End-to-End Validation

- [ ] T078 Manual test: Follow quickstart.md installation steps - verify completes in 5 minutes
- [ ] T079 Manual test: Execute 20 common kubectl operations per FR-007 - verify 80% translate correctly (SC-006)
- [ ] T080 Manual test: Test all 3 risk levels across dev/staging/prod contexts - verify correct confirmation behavior
- [ ] T081 Manual test: Execute "show history" commands - verify audit log queries work
- [ ] T082 Performance test: Measure translation + execution time for 20 commands - verify <5 seconds for 95% (SC-003, FR-013)

### Configuration & Deployment Prep

- [x] T083 [P] Create example config file at `config/example.toml` with local GGUF + OpenAI fallback configuration ✅
- [x] T084 [P] Update `.gitignore` to exclude `~/.kaido/` directory from version control ✅
- [ ] T085 [P] Add security check to `src/config.rs` - verify ~/.kaido/config.toml has permissions 600, warn if not

**Checkpoint**: All user stories complete (1-4), privacy-first architecture restored, constitution compliance in progress

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Cleanup)**: No dependencies - start immediately
  - CRITICAL: Must complete before any new code to avoid merge conflicts
- **Phase 2 (Foundational)**: Depends on Phase 1 completion - BLOCKS all user stories
  - Must have clean codebase before building new modules
- **Phase 3 (US1)**: Depends on Phase 2 completion
  - Core translation flow required for all other stories
- **Phase 4 (US2)**: Depends on Phase 3 completion
  - Needs working translation to add safety controls
- **Phase 5 (US3)**: Can start after Phase 2, ideally after Phase 3
  - Independent of US2, but benefits from having commands to log
- **Phase 6 (US4)**: Depends on Phase 3 and Phase 5 completion
  - Needs both translation and audit logging working
- **Phase 7 (Polish)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: BLOCKING for all other stories - must complete first
  - Core translation and execution functionality
- **User Story 2 (P2)**: Depends on US1 - adds safety layer
  - Independent once US1 is complete
- **User Story 3 (P2)**: Depends on US1 - can develop in parallel with US2
  - Logs the commands from US1, works independently
- **User Story 4 (P3)**: Depends on US1 and US3
  - Needs translation (US1) and logging (US3) working

### Within Each User Story

- **US1**: Context parsing → OpenAI integration → Execution → REPL integration (sequential)
- **US2**: Risk classification → Modal UI → Safety integration (sequential within, but tasks T042-T046 can parallel)
- **US3**: Logger implementation → Query interface → REPL commands (sequential)
- **US4**: Edit UI → Correction logging (sequential)

### Parallel Opportunities

**Phase 1 Cleanup** (Tasks T001-T008):
- T001, T002, T003 can run in parallel (different directories)
- T004-T008 must be sequential (each fixes errors from previous)

**Phase 2 Foundational** (Tasks T009-T018):
- T009-T012 (Configuration) can run in parallel
- T013-T016 (Data structures) can ALL run in parallel (different files)
- T017-T018 (Database) sequential

**Phase 3 US1** (Tasks T019-T038):
- Context tasks (T019-T023) sequential within group
- Translation tasks (T024-T030) sequential within group
- Execution tasks (T031-T033) can parallel with translation
- REPL tasks (T034-T038) must be sequential and last

**Phase 4 US2** (Tasks T039-T050):
- T039-T041 (Risk classification) sequential
- T042-T046 (Modal UI) can run in parallel after T039 (different UI components)
- T047-T050 (Integration) sequential

**Phase 5 US3** (Tasks T051-T062):
- T051-T054 (Logging) sequential
- T055-T059 (Queries) can run in parallel after T051 (different query types)
- T060-T062 (REPL integration) sequential

**Phase 6 US4** (Tasks T063-T070):
- T063-T066 (Edit UI) sequential
- T067-T069 (Logging) sequential but can parallel with UI if using different branches
- T070 (Comment) independent

**Phase 7 Polish** (Tasks T071-T085):
- T071-T073 (Documentation) can ALL run in parallel
- T074-T077 (Constitution checks) sequential (each builds on previous)
- T078-T082 (Testing) sequential (follow quickstart order)
- T083-T085 (Config) can run in parallel

---

## Parallel Example: Phase 2 Foundational

```bash
# Launch all data structure definitions in parallel (different files):
Task T013: "Define EnvironmentType enum in src/kubectl/context.rs"
Task T014: "Define RiskLevel enum in src/kubectl/risk_classifier.rs"
Task T015: "Define KubectlContext struct in src/kubectl/context.rs"
Task T016: "Define TranslationResult struct in src/kubectl/translator.rs"

# These can all be worked on simultaneously by AI or different developers
# because they touch different files and have no inter-dependencies
```

---

## Implementation Strategy

### MVP First (User Story 1 Only) - 60 Days

**Week 1-2**: Phase 1 + Phase 2
1. Complete Phase 1: Code cleanup (T001-T008) - 2 days
2. Complete Phase 2: Foundational infrastructure (T009-T018) - 5 days
3. Validate: Codebase compiles, modules structured

**Week 3-5**: Phase 3 (User Story 1)
4. Context parsing (T019-T023) - 3 days
5. OpenAI integration (T024-T030) - 5 days
6. Execution (T031-T033) - 2 days
7. REPL integration (T034-T038) - 4 days
8. **STOP and VALIDATE**: Test natural language → kubectl flow end-to-end

**Week 6-7**: Phase 4 (User Story 2)
9. Risk classification (T039-T041) - 2 days
10. Confirmation UI (T042-T046) - 5 days
11. Safety integration (T047-T050) - 3 days
12. **STOP and VALIDATE**: Test risk-based confirmations in dev/staging/prod

**Week 8**: Phase 5 (User Story 3)
13. Audit logging (T051-T054) - 3 days
14. Query interface (T055-T059) - 2 days
15. REPL history commands (T060-T062) - 2 days
16. **STOP and VALIDATE**: Test audit log queries

**Week 9 (Optional)**: Phase 6 (User Story 4)
17. Edit UI (T063-T066) - 2 days
18. Correction logging (T067-T070) - 2 days
19. **STOP and VALIDATE**: Test edit workflow

**Week 10**: Phase 7 (Polish)
20. Documentation and validation (T071-T085) - 5 days
21. **FINAL VALIDATION**: Complete quickstart.md test, beta deployment

**60-Day Checkpoint**: MVP ready for beta testing with 5 users

### Incremental Delivery (Recommended)

1. **Day 14**: Foundation Ready (Phase 1 + 2)
   - Codebase clean, modules structured
   - Can demonstrate architecture

2. **Day 35**: MVP (Phase 1 + 2 + 3)
   - User Story 1 working: natural language → kubectl
   - Independently testable
   - **Deploy to 2 beta users for feedback**

3. **Day 49**: Safety Layer (Add Phase 4)
   - User Story 2 added: risk-based confirmations
   - Test in production contexts
   - **Deploy to 5 beta users**

4. **Day 56**: Audit Trail (Add Phase 5)
   - User Story 3 added: complete history
   - Compliance-ready
   - **Deploy to all beta users**

5. **Day 60**: Full Release (Add Phase 6 + 7)
   - User Story 4 added: edit workflow
   - Polish complete
   - **Public beta launch**

### Parallel Team Strategy

With 2 developers:

**Week 1-2**: Both on Phase 1 + Phase 2 (pair programming)
**Week 3-5**: 
- Developer A: Phase 3 (US1)
- Developer B: Start Phase 5 (US3 audit schema, can work independently)

**Week 6-7**:
- Developer A: Phase 4 (US2)
- Developer B: Finish Phase 5 (US3)

**Week 8**:
- Both: Integration testing and Phase 7 polish

This parallelization can compress timeline from 10 weeks to 6-7 weeks.

---

## Task Count Summary

- **Phase 1 (Cleanup)**: 8 tasks
- **Phase 2 (Foundational)**: 10 tasks (18 total)
- **Phase 3 (US1 - MVP)**: 20 tasks (38 total) ⭐ MVP milestone
- **Phase 4 (US2 - Safety)**: 12 tasks (50 total)
- **Phase 5 (US3 - Audit)**: 12 tasks (62 total)
- **Phase 6 (US4 - Feedback)**: 8 tasks (70 total)
- **Phase 7 (Polish)**: 15 tasks (85 total)

**Total**: 85 tasks

**Parallel Opportunities**: 25 tasks marked [P] can run in parallel within their phases

**Independent Test Checkpoints**: 5 checkpoints (after each phase)

**MVP Scope**: Phase 1 + Phase 2 + Phase 3 = 38 tasks (45% of total)

---

## Notes

- **[P] tasks** = Different files, no dependencies, can run in parallel
- **[Story] label** = Maps task to specific user story for traceability
- **Each user story** = Independently completable and testable
- **No mocks allowed**: Every task must produce real, functional code (Constitution VII)
- **Zero warnings target**: Constitution VI compliance checked in Phase 7
- **Commit strategy**: Commit after each task or logical group (e.g., after all T013-T016 parallel tasks)
- **Token awareness**: Each task is sized to be completable in a single AI conversation round (estimate 500-2000 tokens per task explanation + implementation)
- **Validation gates**: Stop at each checkpoint to validate story independently before proceeding

---

## Success Criteria Mapping

This task list directly implements the success criteria from spec.md:

- **SC-001** (50 kubectl operations): US1 (T019-T038) enables natural language for 20+ kubectl verbs
- **SC-002** (Zero prod incidents): US2 (T039-T050) provides production safety controls
- **SC-003** (<5 second latency): US1 (T026 OpenAI timeout), validated in T082
- **SC-004** (3/5 users remain active): Enabled by complete US1+US2+US3 functionality
- **SC-005** (2-3 willing to pay): Validated through 60-day beta test after T085
- **SC-006** (80% accuracy): US1 (T024-T028 OpenAI prompts), validated in T079
- **SC-007** (30% time reduction): Measured through US3 audit log analytics
- **SC-008** (100% command capture): US3 (T051-T062) ensures complete audit trail

---

**Generated**: 2025-10-25  
**Total Tasks**: 85  
**MVP Tasks**: 38 (Phase 1-3)  
**Estimated Timeline**: 60 days (1 developer) or 40 days (2 developers)  
**Next Step**: Begin Phase 1, Task T001

