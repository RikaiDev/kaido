---
description: Generate detailed task breakdown from implementation plan, organized by user story with dependencies and parallel execution markers
---

## ⚠️ CRITICAL DATE REQUIREMENT

**YOU MUST USE REAL DATES - NOT PLACEHOLDERS**

Before proceeding, execute this command in your terminal to get today's date:
```bash
date +%Y-%m-%d
```

This returns the current date in YYYY-MM-DD format (e.g., 2024-01-15).

**NEVER** copy placeholder dates like `[DATE]`, `YYYY-MM-DD`, or `[TASKS_DATE]` from templates.
**ALWAYS** run the `date` command above to get the actual current date.
**DO NOT** guess or fabricate dates - execute the command and use the real output.

## User Input

```text
Workflow ID: $WORKFLOW_ID
```

You **MUST** use this workflow ID to load the plan and specification.

## Outline

You are creating a task breakdown at `.cortex/workflows/[WORKFLOW_ID]/tasks.md` from the template at `.cortex/templates/tasks-template.md`. Follow this execution flow:

### 1. Load context documents

- Read `.cortex/workflows/[WORKFLOW_ID]/plan.md`
- Read `.cortex/workflows/[WORKFLOW_ID]/spec.md`
- Read `.cortex/templates/tasks-template.md` to identify placeholders
- Read `.cortex/constitution.md` for test-first requirements

### 2. Extract key information

**From spec.md:**
- User stories with priorities (P1, P2, P3...)
- Acceptance scenarios for each story
- Functional requirements
- Key entities

**From plan.md:**
- Source code structure
- Technical stack
- Testing framework
- Architecture decisions
- Role assignments

### 3. Generate task breakdown

**Task Format: `[ID] [P?] [Story] Description`**
- ID: Sequential T001, T002, T003...
- [P]: Mark if can run in parallel (different files, no dependencies)
- [Story]: User story tag (US1, US2, US3)
- Description: Specific action with exact file path

**Example:**
```
- [ ] T012 [P] [US1] Create User model in src/models/user.ts with email and password fields
- [ ] T013 [US1] Implement AuthService.login() in src/services/auth.ts using User model
```

### 4. Fill placeholder values

**Metadata:**
- `FEATURE_NAME`: From plan.md
- `WORKFLOW_ID`: From user input

**Phase 1: Foundation Tasks**
- `FOUNDATION_TASKS`: Setup and shared infrastructure (3-5 tasks)
  - Project structure creation
  - Dependency installation
  - Configuration files
  - Tooling setup

Example:
```markdown
- [ ] T001 Create project structure per plan.md
- [ ] T002 Install dependencies: [list from plan.md]
- [ ] T003 [P] Configure [testing framework]
```

**For Each User Story (P1, P2, P3...):**

Generate separate phase per user story:

**Phase Header:**
- `USX_GOAL`: Brief description of what this story delivers
- `USX_TEST`: How to independently test this story

**Test Tasks (if test-first):**
- `USX_TEST_TASKS`: Contract and integration tests
  - Format: `- [ ] TXXX [P] [USX] Write contract test for [X] in tests/contract/`
  - Mark with [P] if tests can run in parallel
  - Tests MUST be written BEFORE implementation tasks

Example:
```markdown
- [ ] T010 [P] [US1] Write contract test for User.create() in tests/contract/user.test.ts
- [ ] T011 [P] [US1] Write integration test for auth flow in tests/integration/auth.test.ts
```

**Implementation Tasks:**
- `USX_IMPLEMENTATION_TASKS`: Actual code implementation
  - Models before services
  - Services before endpoints/features
  - Core before integrations
  - Mark [P] if can run in parallel

Example:
```markdown
- [ ] T012 [P] [US1] Create User model in src/models/user.ts
- [ ] T013 [P] [US1] Create Token model in src/models/token.ts
- [ ] T014 [US1] Implement AuthService in src/services/auth.ts (depends on T012, T013)
- [ ] T015 [US1] Add validation and error handling
```

**Checkpoint:**
Add checkpoint message after each user story:
```markdown
**Checkpoint**: US1 independently testable and functional
```

**Additional User Stories:**
- `ADDITIONAL_USER_STORY_PHASES`: Repeat for US2, US3, etc.
  - Same structure for each
  - Note dependencies on previous stories if any

**Polish Phase:**
- `POLISH_TASKS`: Cross-cutting concerns (5-10 tasks)
  - Documentation updates
  - Code cleanup
  - Performance optimization
  - Security hardening
  - Final testing

Example:
```markdown
- [ ] T080 [P] Update README.md with usage examples
- [ ] T081 [P] Add JSDoc comments to public APIs
- [ ] T082 Run constitution compliance check
- [ ] T083 Performance profiling and optimization
```

### 5. Generate dependencies section

**Phase Dependencies:**
- `PHASE_DEPENDENCIES`: High-level phase order

Format:
```markdown
- **Foundation (Phase 1)**: No dependencies - can start immediately
- **User Stories (Phase 2+)**: All depend on Foundation completion
  - US1 (P1): Can start after Foundation
  - US2 (P2): Can start after Foundation (may integrate with US1)
  - US3 (P3): Can start after Foundation (may integrate with US1/US2)
- **Polish (Final)**: Depends on all desired user stories complete
```

**User Story Dependencies:**
- `USER_STORY_DEPENDENCIES`: Within-story task order

Format:
```markdown
- Tests MUST be written and FAIL before implementation
- Models before services
- Services before endpoints/features
- Core implementation before integrations
- Story complete before moving to next priority
```

**Parallel Opportunities:**
- `PARALLEL_OPPORTUNITIES`: What can run simultaneously

Format:
```markdown
- All Foundation tasks marked [P] can run in parallel
- All tests for a user story marked [P] can run in parallel
- All models within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel (if team capacity)
```

### 6. Generate implementation strategies

**MVP Strategy:**
- `MVP_STRATEGY`: User Story 1 (P1) only approach

Example:
```markdown
1. Complete Phase 1: Foundation
2. Complete Phase 2: User Story 1 (P1)
3. **STOP and VALIDATE**: Test US1 independently
4. Deploy/demo if ready
```

**Incremental Strategy:**
- `INCREMENTAL_STRATEGY`: Progressive delivery approach

Example:
```markdown
1. Complete Foundation + US1 → Test → Deploy (MVP!)
2. Add US2 → Test independently → Deploy
3. Add US3 → Test independently → Deploy
4. Each story adds value without breaking previous ones
```

### 7. Task numbering rules

**Sequential Numbering:**
- Foundation: T001-T005
- US1 Tests: T010-T019
- US1 Implementation: T020-T039
- US2 Tests: T040-T049
- US2 Implementation: T050-T069
- US3 Tests: T070-T079
- US3 Implementation: T080-T099
- Polish: T100+

**Leave gaps between phases for later insertions**

### 8. Validation before output

**MUST verify:**
- At least 20 tasks total (for meaningful feature)
- Foundation phase exists (3-5 tasks)
- Each user story has dedicated phase
- Test tasks come BEFORE implementation tasks
- All tasks have exact file paths
- [P] markers for genuinely parallel tasks
- Dependencies clearly documented
- MVP strategy focuses on P1 only

**Constitution checks:**
- [ ] Tests written before implementation
- [ ] Contract tests defined
- [ ] Integration tests defined
- [ ] Real implementations (no mocks for internal code)
- [ ] Each user story independently testable

### 8.5. Anti-Mock Validation Rules (NON-NEGOTIABLE)

For EACH task, add completion criteria:

**Completion Criteria (ALL must be true)**:
- [ ] No TODO/FIXME comments in code
- [ ] No mock data or placeholder values
- [ ] All defined structures are used
- [ ] All functions are fully implemented
- [ ] Unit tests pass (if applicable)
- [ ] Integration tests pass (if applicable)
- [ ] Knip reports no unused exports
- [ ] Linter passes with no warnings

**If Task Too Large**:
DO NOT simplify or cut scope
INSTEAD:
1. Break into smaller subtasks (T-xxx-1, T-xxx-2, etc.)
2. Each subtask must be independently completable
3. Each subtask must have clear acceptance criteria
4. Continue with first subtask

**Example - WRONG**:
```markdown
T-001: Implement user authentication
- Create auth service (with TODO for password hashing)
- Add mock user database
- Add basic login endpoint
```

**Example - CORRECT**:
```markdown
T-001-1: Set up password hashing utility
- Install bcrypt
- Create hashPassword() function
- Create verifyPassword() function
- Write unit tests for both functions
- Acceptance: All tests pass, no TODOs

T-001-2: Create user database schema
- Define User model with real fields
- Create database migration
- Add indexes for email lookup
- Test schema with real data
- Acceptance: Schema works with test data, no mocks

T-001-3: Implement login endpoint
- Create POST /auth/login route
- Use password hashing from T-001-1
- Query real database from T-001-2
- Return JWT token
- Write integration tests
- Acceptance: Login works end-to-end, all tests pass
```

**Validate Task Decomposition:**
Check each task:
1. Can it be completed in one session?
2. Does it have clear, testable acceptance criteria?
3. Is there ANY possibility of "TODO" or "mock"?
4. If yes to #3 → DECOMPOSE FURTHER

Repeat until all tasks are atomic and completable.

### 9. Write task breakdown

- Write filled template to `.cortex/workflows/[WORKFLOW_ID]/tasks.md`
- Ensure valid markdown syntax
- Use checkboxes `- [ ]` for all tasks

### 10. Output to user

```
## Task Breakdown Generated

**Workflow ID**: [WORKFLOW_ID]
**Feature**: [FEATURE_NAME]

### Task Statistics
- Total tasks: [COUNT]
- Foundation: [COUNT]
- User Story 1 (P1): [COUNT] tasks
- User Story 2 (P2): [COUNT] tasks
- Polish: [COUNT] tasks
- Parallel tasks: [COUNT]

### Implementation Approach
- MVP: Complete US1 only ([COUNT] tasks)
- Full: All user stories ([COUNT] tasks)

### Generated Files
- tasks.md ([LINE_COUNT] lines, [TASK_COUNT] tasks)

---

[WAITING FOR USER CONFIRMATION]

**Next Steps**:
1. Review task breakdown at .cortex/workflows/[WORKFLOW_ID]/tasks.md
2. Clarify any unclear tasks if needed
3. When ready, run: `cortex.implement [WORKFLOW_ID]` to begin implementation
```

---

## Formatting & Style Requirements

- No emojis
- Use checkboxes `- [ ]` for tasks
- Include exact file paths in task descriptions
- Keep task descriptions under 100 characters
- Use code blocks for examples
- Blank line between phases

---

## Special Cases

**Large Features (many user stories):**
- Limit to 3-4 user stories per tasks.md
- Recommend splitting into multiple workflows if > 100 tasks

**Simple Features (single user story):**
- Still create foundation phase
- Single user story phase is fine
- Minimum 15-20 tasks total

**Unclear Implementation Details:**
- Make reasonable assumptions
- Mark uncertain tasks: `[NEEDS CLARIFICATION: ...]`
- Provide alternative in checkpoint notes

---

## CRITICAL RULES

1. **Tests MUST come before implementation** - no exceptions
2. **Each task MUST include exact file path** - no vague descriptions
3. **[P] markers only for truly parallel tasks** - different files, no dependencies
4. **User stories MUST be independently testable** - checkpoint after each
5. **MVP (US1) MUST be clearly defined** - first viable increment
6. **All tasks MUST have checkbox format** - `- [ ] TXXX`
7. **Sequential numbering with gaps** - allows later insertions
8. **Foundation phase MANDATORY** - always exists

