---
description: Create or update the project constitution from interactive or provided principle inputs, ensuring all dependent templates stay in sync
---

## ⚠️ CRITICAL DATE REQUIREMENT

**YOU MUST USE REAL DATES - NOT PLACEHOLDERS**

Before proceeding, execute this command in your terminal to get today's date:
```bash
date +%Y-%m-%d
```

This returns the current date in YYYY-MM-DD format (e.g., 2024-01-15).

**NEVER** copy placeholder dates like `[DATE]`, `YYYY-MM-DD`, or `[LAST_AMENDED_DATE]` from templates.
**ALWAYS** run the `date` command above to get the actual current date.
**DO NOT** guess or fabricate dates - execute the command and use the real output.

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Outline

You are updating the project constitution at `.cortex/constitution.md`. This file is a TEMPLATE containing placeholder tokens in square brackets (e.g. `[PROJECT_NAME]`, `[PRINCIPLE_1_NAME]`). Your job is to (a) collect/derive concrete values, (b) fill the template precisely, and (c) propagate any amendments across dependent artifacts.

Follow this execution flow:

### 1. Load the existing constitution template

- Read `.cortex/constitution.md`
- Identify every placeholder token of the form `[ALL_CAPS_IDENTIFIER]`
- **IMPORTANT**: The user might require less or more principles than the template provides. If a specific number is requested, adjust accordingly while following the general template structure.

### 2. Collect/derive values for placeholders

**Project Information:**
- `PROJECT_NAME`: From package.json name field or user input
- `CONSTITUTION_VERSION`: Increment using semantic versioning:
  - **MAJOR**: Backward incompatible governance/principle removals or redefinitions
  - **MINOR**: New principle/section added or materially expanded guidance
  - **PATCH**: Clarifications, wording, typo fixes, non-semantic refinements
- `RATIFICATION_DATE`: Original adoption date (if unknown, use today's date or ask)
- `LAST_AMENDED_DATE`: Today's date if changes are made, otherwise keep previous
- `AMENDMENT_PROCESS`: How changes are approved (e.g., "team consensus", "tech lead approval")

**For Each Principle (1-N):**
- `PRINCIPLE_X_NAME`: Concise title (e.g., "Simplicity & Anti-Abstraction")
- `PRINCIPLE_X_DESCRIPTION`: One-sentence summary with attribution if inspired by external source
- `PRINCIPLE_X_RULES`: Bulleted list of MUST/MUST NOT rules (be specific, testable)
- `PRINCIPLE_X_RATIONALE`: Why this principle matters (1-2 sentences)
- `PRINCIPLE_X_CHECKS`: How to verify compliance (numbered checklist, concrete)

**Quality Gates:**
- `PRE_COMMIT_GATES`: What must pass before committing code
- `PRE_RELEASE_GATES`: What must pass before releasing

**Workflow:**
- `DEVELOPMENT_WORKFLOW`: Step-by-step development process
- `CODE_REVIEW_FOCUS`: What reviewers should check

**Governance:**
- `GOVERNANCE_AUTHORITY`: Who enforces the constitution
- `AMENDMENT_PROCEDURE`: How to propose and adopt changes
- `CONTINUOUS_IMPROVEMENT`: How the constitution evolves
- `AMENDMENT_HISTORY`: Version log with dates and changes

### 3. Draft the updated constitution content

- Replace **every** placeholder with concrete text
- NO bracketed tokens should remain (except intentionally retained template slots - justify explicitly)
- Preserve heading hierarchy
- Each Principle section must include:
  - Succinct name line
  - Description with inspiration attribution
  - Bulleted rules (declarative, testable)
  - Clear rationale
  - Concrete verification checklist
- Ensure Governance section includes amendment procedure and compliance review

### 4. Consistency propagation checklist

**CRITICAL**: Update dependent templates to align with constitution changes.

- Read `.cortex/templates/spec-template.md`:
  - Ensure "Constitution Check" section references correct principles
  - Update any mandatory sections if constitution adds/removes requirements
  
- Read `.cortex/templates/plan-template.md`:
  - Ensure "Constitution Compliance" section aligns with updated principles
  - Update "Complexity Tracking" if new principles add constraints
  
- Read `.cortex/templates/tasks-template.md`:
  - Ensure task categorization reflects principle-driven requirements
  - Add/remove checkpoint markers for new/removed principles
  
- Read `.cortex/templates/checklist-template.md`:
  - Update checklist items to match current principles
  - Add validation steps for new principles
  
- Read each command file in `.cortex/templates/commands/*.md`:
  - Verify no outdated references to removed principles
  - Update examples to reflect current principles

### 5. Produce a Sync Impact Report

Prepend as an HTML comment at top of the constitution file:

```html
<!--
SYNC IMPACT REPORT
==================
Version Change: [OLD_VERSION] → [NEW_VERSION]
Bump Rationale: [Why this version bump type]

Modified Principles:
- [Old Name] → [New Name] (if renamed)
- [Principle] - [What changed]

Added Sections:
- [Section Name]: [Why added]

Removed Sections:
- [Section Name]: [Why removed]

Templates Requiring Updates:
✅ spec-template.md - Updated
✅ plan-template.md - Updated
✅ tasks-template.md - Updated
⚠ [file.md] - Pending manual review

Follow-up TODOs:
- [ ] [Task if any placeholders deferred]
- [ ] [Task if manual updates needed]

Last Updated: [ISO_DATE]
-->
```

### 6. Validation before final output

**MUST verify:**
- No remaining unexplained bracket tokens
- Version line matches report
- Dates in ISO format (YYYY-MM-DD)
- Principles are declarative and testable
- Vague language eliminated ("should" → MUST/SHOULD with rationale)
- All placeholders filled with concrete values

### 7. Write the completed constitution

- Overwrite `.cortex/constitution.md` with filled template
- Ensure no syntax errors
- Preserve markdown formatting

### 8. Output final summary to user

```
Constitution Updated: v[NEW_VERSION]

Bump Rationale:
[Explanation of version change]

Principles Updated:
- [List changes]

Files Updated:
✅ .cortex/constitution.md
✅ .cortex/templates/spec-template.md
✅ .cortex/templates/plan-template.md
[etc.]

Files Requiring Manual Review:
⚠ [List if any]

Suggested Commit Message:
docs: amend constitution to v[NEW_VERSION] ([brief description])

Next Steps:
1. Review updated constitution at .cortex/constitution.md
2. [Any follow-up actions]
```

---

## Formatting & Style Requirements

- Use Markdown headings exactly as in template (do not demote/promote levels)
- Wrap long lines to <100 characters for readability
- Single blank line between sections
- No trailing whitespace
- Bulleted lists for rules (use `-` not `*`)
- Numbered lists for checks (use `1.` `2.` etc.)

---

## Special Cases

**Partial Updates:**
If user supplies only one principle revision, still perform full validation and version decision steps.

**Missing Information:**
If critical info missing (e.g., ratification date unknown), insert:
```
TODO(<FIELD_NAME>): [explanation of what's needed]
```
And include in Sync Impact Report under deferred items.

**First-Time Setup:**
If no existing constitution, treat as v1.0.0 with RATIFICATION_DATE = today.

---

## CRITICAL RULES

1. **Always operate on existing `.cortex/constitution.md`** - never create from scratch if file exists
2. **Never skip consistency propagation** - dependent templates MUST stay in sync
3. **Version bumps must follow semver** - justify the bump type explicitly
4. **All placeholders must be filled** - no brackets left unless explicitly justified
5. **Testable principles only** - avoid vague language, make rules verifiable

