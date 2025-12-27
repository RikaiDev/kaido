---
description: Execute implementation phase by coordinating role execution and task completion with constitution compliance
---

## ⚠️ CRITICAL DATE REQUIREMENT

**YOU MUST USE REAL DATES - NOT PLACEHOLDERS**

Before proceeding, execute this command in your terminal to get today's date:
```bash
date +%Y-%m-%d
```

This returns the current date in YYYY-MM-DD format (e.g., 2024-01-15).

**NEVER** copy placeholder dates like `[DATE]`, `YYYY-MM-DD`, or `[IMPLEMENTATION_DATE]` from templates.
**ALWAYS** run the `date` command above to get the actual current date.
**DO NOT** guess or fabricate dates - execute the command and use the real output.

## User Input

```text
Workflow ID: $WORKFLOW_ID
```

You **MUST** use this workflow ID to load tasks and execute implementation.

## Outline

You are NOT generating a document - you are COORDINATING IMPLEMENTATION. The implementation phase uses Cortex's Multi-Role system to execute tasks from the task breakdown. Follow this execution flow:

### 1. Load task breakdown

- Read `.cortex/workflows/[WORKFLOW_ID]/tasks.md`
- Parse all tasks with format: `[ID] [P?] [Story] Description`
- Identify current phase (Foundation, US1, US2, etc.)
- Check task completion status (`[ ]` vs `[X]`)

### 2. Determine execution strategy

**Check user preference:**
- MVP only (US1): Execute Foundation + US1 only
- Incremental: Execute one user story at a time
- Full: Execute all user stories

**Default: MVP First**
- Foundation → US1 → Stop for validation

### 2.5. Pre-Implementation Check (CRITICAL)

**BEFORE starting ANY implementation:**

1. Load task from tasks.md
2. Analyze task complexity using these criteria:
   - More than 2 file changes required?
   - More than 200 lines of code estimated?
   - Involves multiple systems/modules?
   - Has complex logic flow?

3. If task is too large:
   - PAUSE implementation
   - Break task into subtasks (T-xxx-1, T-xxx-2, etc.)
   - Update tasks.md with new subtasks
   - Inform user: "Task [ID] too large, decomposed into [N] subtasks"
   - Start with first subtask only

**Example**:
```
Task T-045 requires creating auth service + database schema + login endpoint.
This is 3 separate concerns.

DECOMPOSING:
- T-045-1: Create auth service (1 file, ~80 lines)
- T-045-2: Create database schema (1 file, ~60 lines)
- T-045-3: Implement login endpoint (1 file, ~100 lines)

Starting with T-045-1...
```

### 3. Task execution order

**For current phase:**

**Foundation Phase:**
1. Identify all tasks without dependencies
2. Execute non-parallel tasks sequentially (no [P] marker)
3. Execute parallel tasks ([P]) simultaneously using parallel tool calls
4. Validate foundation ready before proceeding

**Parallel Execution Rules:**
- Tasks marked [P] can execute simultaneously IF they modify different files
- Check task descriptions for file paths to avoid conflicts
- Group [P] tasks by file dependency - execute each group in parallel
- Example: `T001 [P] Create models/user.ts` and `T002 [P] Create models/post.ts` can run together

**User Story Phases:**
For each user story (in priority order):

**Step 1: Test Tasks First**
- Execute all test tasks marked [USX]
- Tests MUST FAIL initially (red phase)
- Do NOT proceed until tests exist and fail

**Step 2: Implementation Tasks**
- Execute implementation tasks sequentially
- Skip parallel execution unless explicitly marked [P]
- After each task: verify tests start passing (green phase)

**Step 3: Checkpoint Validation**
- When user story complete: verify all tests pass
- Run integration tests
- Validate independent functionality
- Pause for user confirmation

### 4. Role assignment

**Map tasks to roles:**
- Model/Type creation → Code Assistant
- Service implementation → Code Assistant
- Test writing → Testing Specialist
- CLI interface → Code Assistant
- Documentation → Documentation Specialist
- Architecture decisions → Architecture Designer

**Execution format:**
```
Executing: T012 [P] [US1] Create User model in src/models/user.ts

Assigned Role: Code Assistant
Task Type: Model Creation
Constitution Check: ✓ No mocks, ✓ Real implementation

[Role executes task...]

Result: ✓ Complete
- File created: src/models/user.ts
- Tests affected: tests/contract/user.test.ts
- Next task: T013
```

### 5. Constitution validation per task

**Before each task execution, verify:**
- [ ] No mock data being added (unless external service)
- [ ] No hardcoded values (use config/env)
- [ ] No placeholder implementations
- [ ] Tests exist for this functionality
- [ ] CLI interface considerations

**If violations detected:**
- STOP immediately
- Report violation to user
- Request correction before proceeding

### 5.5. During Implementation - FORBIDDEN Actions (NON-NEGOTIABLE)

**NEVER DO THESE:**
- Writing "// TODO: implement this later"
- Creating mock data structures
- Defining unused interfaces/types
- Throwing "Not implemented" errors
- Leaving placeholder functions
- Asking user "continue or simplify?"

**REQUIRED ACTIONS:**
- If running out of context → SAVE PROGRESS immediately
- Create new subtask for remaining work
- Update tasks.md with progress and new subtask
- Inform user: "Task [ID] partially complete, created subtask [ID-N] for remaining work"

**Example**:
```
Implementing T-045-1: Create auth service...
[150 lines written, context running low]

PAUSING:
- Progress saved: AuthService with login() and register() complete
- Remaining work: logout(), refreshToken(), validateToken()
- Created T-045-1-B: Complete remaining auth service methods
- Updated tasks.md

User notification: "T-045-1 partially complete. Created T-045-1-B for remaining methods."
```

### 5.6. Post-Implementation Validation (MANDATORY)

**AFTER completing each task, RUN validation:**

1. **Mock/Scaffold Detection** (BLOCKER):
```bash
# Scan for forbidden patterns
grep -r "TODO\|FIXME\|Not implemented\|mock.*=\|MOCK_" [changed files]
```

2. **Unused Code Detection** (BLOCKER):
```bash
npm run knip
```

3. **Linter Check** (BLOCKER):
```bash
npm run build
```

**If ANY validation fails:**
- BLOCK task completion
- List ALL issues found
- Create fix subtasks immediately
- DO NOT proceed to next task
- Message user: "Task [ID] blocked by validation failures. Created [N] fix tasks."

**Completion Criteria:**
A task is ONLY complete when:
1. All code written (no TODOs)
2. No mock patterns detected
3. No unused exports (knip passes)
4. Linter passes (zero warnings)
5. All tests pass
6. Constitution compliance verified

NO EXCEPTIONS.

### 6. Progress tracking

**Track execution state:**
```json
{
  "workflowId": "[WORKFLOW_ID]",
  "currentPhase": "User Story 1",
  "totalTasks": 50,
  "completedTasks": 12,
  "currentTask": "T013",
  "nextCheckpoint": "US1 complete",
  "violations": []
}
```

**Save to:** `.cortex/workflows/[WORKFLOW_ID]/execution/progress.json`

### 7. Execution log

**Log each task execution:**

Format:
```markdown
## Execution Log

### 2025-01-24 10:30

**T012** [P] [US1] Create User model in src/models/user.ts
- Role: Code Assistant
- Status: ✓ Complete
- Duration: 45s
- Files: src/models/user.ts (new)
- Tests: tests/contract/user.test.ts (affects)

### 2025-01-24 10:32

**T013** [US1] Implement AuthService in src/services/auth.ts
- Role: Code Assistant
- Status: ⚠ Partial - needs error handling
- Duration: 120s
- Files: src/services/auth.ts (new)
- Tests: tests/integration/auth.test.ts (affects)
- Notes: Added TODO for rate limiting
```

**Save to:** `.cortex/workflows/[WORKFLOW_ID]/execution/log.md`

### 8. Checkpoint handling

**When reaching checkpoint (end of user story):**

1. Run all tests for this story
2. Validate constitution compliance
3. Generate checkpoint summary
4. Pause for user confirmation

**Checkpoint Output:**
```markdown
## Checkpoint: User Story 1 Complete

**Status**: ✓ Ready for Review

### Completed Tasks
- T010-T025: All 16 tasks complete
- Tests: 5 contract, 3 integration (all passing)
- Files: 8 new files created

### Constitution Compliance
✓ No mock data used
✓ No hardcoded values
✓ All functionality tested
✓ CLI interface implemented

### Independent Test
**Test**: User can authenticate via CLI
**Command**: `auth-cli login --email test@example.com`
**Result**: ✓ Works independently

---

[WAITING FOR USER CONFIRMATION]

**Next Steps**:
1. Review implementation
2. Test US1 independently
3. When satisfied:
   - Continue to US2: `cortex.implement [WORKFLOW_ID] --continue`
   - Deploy MVP: `cortex.pr [WORKFLOW_ID]`
```

### 9. Error handling

**If task fails:**
1. Log failure with error details
2. Mark task as failed in tasks.md
3. Pause execution
4. Report to user with remediation suggestions

**If constitution violation:**
1. STOP immediately
2. Report violation clearly
3. Suggest fix
4. Do not proceed until fixed

**If tests don't pass:**
1. Mark implementation as partial
2. Log which tests failing
3. Pause for debugging
4. Do not proceed to next task

### 10. Output to user

**Startup output:**
```
## Implementation Phase Started

**Workflow ID**: [WORKFLOW_ID]
**Strategy**: MVP First (Foundation + US1)
**Total Tasks**: [COUNT]
**Estimated Duration**: [ESTIMATE]

### Phase 1: Foundation
Starting Foundation tasks...
[ Progress bar or task list ]

---

Implementation will pause at checkpoints for your confirmation.
Constitution violations will stop execution immediately.
```

**During execution:**
```
Executing T012 [US1] Create User model...
✓ Complete (45s)

Executing T013 [US1] Implement AuthService...
✓ Complete (120s)

[ Continue until checkpoint... ]
```

**At checkpoint:**
```
[Checkpoint summary as shown in step 8]
```

---

## Formatting & Style Requirements

- Clear progress indicators
- Timestamp all log entries
- Use symbols: ✓ (success), ✗ (fail), ⚠ (partial)
- Keep status updates concise
- Detailed logs in execution/log.md

---

## Special Cases

**Parallel Task Execution:**
- If multiple tasks marked [P] in sequence
- Execute them concurrently
- Wait for all to complete before next non-parallel task

**User Interruption:**
- Save current state
- Allow resume from last completed task
- Preserve all logs and progress

**Skipping User Stories:**
- Allow user to skip P2, P3, etc.
- Only execute requested priorities
- Maintain checkpoints for each

---

## CRITICAL RULES

1. **Tests BEFORE implementation** - red → green → refactor
2. **Constitution check EVERY task** - violations stop execution
3. **Checkpoint at every user story end** - mandatory pause
4. **Never fake completion** - partial is better than false success
5. **Log EVERYTHING** - execution/log.md tracks all actions
6. **User confirmation required** - at every checkpoint
7. **Fail fast on violations** - don't continue with bad code
8. **Independent testing** - each story must work standalone

---

## Integration with Multi-Role System

**This command coordinates roles - it does NOT implement directly.**

When task assigned to role:
1. Load role prompt from Multi-Role system
2. Provide task context
3. Execute role
4. Validate output
5. Record result
6. Proceed to next task

**Role Execution:**
```
Task: T012 Create User model
↓
Assign: Code Assistant role
↓
Provide Context: {spec, plan, tasks, constitution}
↓
Execute: Role generates code
↓
Validate: Check constitution compliance
↓
Save: Write to src/models/user.ts
↓
Log: Record in execution/log.md
↓
Next: T013
```

