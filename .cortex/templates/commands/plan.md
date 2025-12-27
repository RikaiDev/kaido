---
description: Create implementation plan from approved specification with technical context, architecture decisions, and role assignments
---

## ⚠️ CRITICAL DATE REQUIREMENT

**YOU MUST USE REAL DATES - NOT PLACEHOLDERS**

Before proceeding, execute this command in your terminal to get today's date:
```bash
date +%Y-%m-%d
```

This returns the current date in YYYY-MM-DD format (e.g., 2024-01-15).

**NEVER** copy placeholder dates like `[DATE]`, `YYYY-MM-DD`, or `[PLAN_DATE]` from templates.
**ALWAYS** run the `date` command above to get the actual current date.
**DO NOT** guess or fabricate dates - execute the command and use the real output.

## User Input

```text
Workflow ID: $WORKFLOW_ID
```

You **MUST** use this workflow ID to load the specification.

## Outline

You are creating an implementation plan at `.cortex/workflows/[WORKFLOW_ID]/plan.md` from the template at `.cortex/templates/plan-template.md`. Follow this execution flow:

### 1. Load specification and context

- Read `.cortex/workflows/[WORKFLOW_ID]/spec.md`
- Read `.cortex/templates/plan-template.md` to identify placeholders
- Read `.cortex/constitution.md` for compliance checking
- Identify project language/framework from existing codebase

### 2. Extract from specification

**From spec.md extract:**
- Feature name and summary
- User stories with priorities
- Functional requirements
- Key entities/data structures
- Technical constraints mentioned
- Success criteria

### 3. Determine technical context

**Analyze existing project:**
- Language and version (from package.json, go.mod, requirements.txt, etc.)
- Primary dependencies (frameworks, libraries)
- Existing architecture patterns (from src/ structure)
- Storage solution (database, files, memory)
- Testing framework (from test files or package.json)
- Target platform (web, mobile, CLI, API)

**If new project:**
- Infer from feature requirements
- Follow constitution's "Simplicity & Anti-Abstraction" principle
- Choose minimal viable tech stack

### 4. Fill placeholder values

**Metadata:**
- `FEATURE_NAME`: Extract from spec.md
- `BRANCH_NAME`: Same as workflow ID
- `DATE`: Today's date (YYYY-MM-DD)
- `WORKFLOW_ID`: From user input

**Summary:**
- `SUMMARY`: 2-3 sentence summary combining:
  - Primary requirement from spec
  - Chosen technical approach
  - Key architectural decision

**Technical Context:**
- `LANGUAGE_VERSION`: e.g., "TypeScript 5.0", "Python 3.11", "Go 1.21"
- `DEPENDENCIES`: List 3-5 primary dependencies
- `ARCHITECTURE_PATTERN`: e.g., "Library-first with CLI", "MVC", "Layered"
- `STORAGE_SOLUTION`: e.g., "PostgreSQL", "File-based JSON", "In-memory", "N/A"
- `TESTING_FRAMEWORK`: e.g., "Vitest", "pytest", "Go testing"
- `TARGET_PLATFORM`: e.g., "Node.js 18+", "Web browsers", "iOS 15+"

**Constitution Compliance:**
- `CONSTITUTION_COMPLIANCE`: For each principle:
  - **Simplicity**: How this plan avoids over-engineering
  - **Test-First**: Testing strategy summary
  - **Library-First**: How feature is modularized
  - **CLI Interface**: CLI exposure plan
  - **Real Implementation**: No mocks/hardcoding commitment

Format:
```markdown
**Simplicity & Anti-Abstraction**:
- Using [approach] instead of complex [alternative]
- Maximum [N] projects/modules

**Test-First**:
- Contract tests for [X]
- Integration tests for [Y]
- Written before implementation

**Library-First**:
- Feature implemented as [library/module name]
- Can be imported independently

**CLI Interface**:
- Exposed via `[command-name]` command
- Input: [format], Output: [format]

**Real Implementation**:
- No mock data in production
- All functionality tested with real dependencies
```

**Project Structure:**
- `SOURCE_CODE_STRUCTURE`: Show actual directory tree
  - If single project: `src/`, `tests/`
  - If web app: `backend/`, `frontend/`
  - If mobile: `api/`, `ios/` or `android/`
  - Include specific directories for this feature
  - Mark new files with `(new)`

Example:
```
src/
├── features/
│   └── user-auth/          (new)
│       ├── index.ts        (new)
│       ├── cli.ts          (new)
│       ├── types.ts        (new)
│       └── __tests__/      (new)
tests/
├── integration/
│   └── user-auth.test.ts   (new)
```

- `STRUCTURE_RATIONALE`: Explain why this structure (1-2 sentences)

**Role Assignments:**
- `ROLE_ASSIGNMENTS`: Map workflow phases to Multi-Role system

Format:
```markdown
- **Architecture Designer**: Technical decisions, system design (this phase)
- **Code Assistant**: Implementation of user stories
- **Testing Specialist**: Test strategy, contract/integration tests
- **Documentation Specialist**: README, API docs
```

**Complexity Tracking:**
- `COMPLEXITY_JUSTIFICATIONS`: **ONLY if constitution violations**
  - If no violations: "No complexity violations. Plan adheres to constitution."
  - If violations: Table format

```markdown
| Violation | Why Needed | Simpler Alternative Rejected |
|-----------|------------|------------------------------|
| 4th project added | [reason] | [why 3 projects insufficient] |
| Repository pattern | [reason] | [why direct DB access insufficient] |
```

### 5. Validation before output

**MUST verify:**
- Technical context matches actual project (if existing)
- Architecture pattern follows constitution's simplicity principle
- No over-engineering (max 3 projects, no unnecessary abstractions)
- Test strategy is test-first (contract → integration → implementation)
- CLI interface planned for feature
- All placeholders filled
- Structure shows specific files for this feature

**Constitution checks:**
- [ ] Uses existing stdlib/frameworks (not adding unnecessary dependencies)
- [ ] Solves real problem NOW (not future-proofing)
- [ ] No abstractions until 3rd use case
- [ ] Feature is independently testable
- [ ] Feature can run via CLI

### 6. Write implementation plan

- Write filled template to `.cortex/workflows/[WORKFLOW_ID]/plan.md`
- Ensure valid markdown syntax
- Include code blocks for structure

### 7. Output to user

```
## Implementation Plan Generated

**Workflow ID**: [WORKFLOW_ID]
**Feature**: [FEATURE_NAME]
**Architecture**: [ARCHITECTURE_PATTERN]
**Language**: [LANGUAGE_VERSION]

### Technical Decisions
- Dependencies: [COUNT] primary
- Structure: [STRUCTURE_TYPE]
- Testing: [TESTING_FRAMEWORK]

### Constitution Compliance
[✓] Simplicity - No over-engineering
[✓] Test-First - Contract tests planned
[✓] Library-First - Independent module
[✓] CLI Interface - Command planned
[✓/✗] Complexity - [Status]

### Generated Files
- plan.md ([LINE_COUNT] lines)

---

[WAITING FOR USER CONFIRMATION]

**Next Steps**:
1. Review plan at .cortex/workflows/[WORKFLOW_ID]/plan.md
2. Request technical adjustments if needed
3. When approved, run: `cortex.tasks [WORKFLOW_ID]` to generate task breakdown
```

---

## Formatting & Style Requirements

- No emojis
- Use backticks for code/commands
- Code blocks for structure trees
- Tables for complexity tracking
- Keep paragraphs under 4 lines
- Blank line between sections

---

## Special Cases

**Existing vs New Project:**
- Existing: Match current tech stack, extend existing structure
- New: Minimal viable stack, follow constitution strictly

**Unclear Technical Requirements:**
- Make reasonable assumptions based on feature type
- Mark uncertain decisions: `[NEEDS CONFIRMATION: ...]`
- Propose alternatives in complexity tracking

**Multiple Features in Spec:**
- Address all user stories in structure
- Show phased implementation (P1 → P2 → P3)
- Note dependencies between features

---

## CRITICAL RULES

1. **Technical context MUST match existing project** (if applicable)
2. **Architecture MUST follow constitution's simplicity principle**
3. **No over-engineering** - max 3 projects, no unnecessary abstractions
4. **Test-first strategy MANDATORY** - contract tests before implementation
5. **CLI interface REQUIRED** - every feature must be CLI-accessible
6. **Constitution compliance check MANDATORY** - never skip
7. **All placeholders MUST be filled** - no `[TOKENS]` left
8. **Structure MUST show this feature's files** - not generic templates

