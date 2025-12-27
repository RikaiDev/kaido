<!-- 
  CONSTITUTION TEMPLATE - DO NOT EDIT DIRECTLY
  Use: cortex constitution <principles> to generate/update
  This file contains placeholder tokens that will be filled by the command
-->

# [PROJECT_NAME] Constitution

**Version**: [CONSTITUTION_VERSION]  
**Ratified**: [RATIFICATION_DATE]  
**Last Amended**: [LAST_AMENDED_DATE]

---

## Purpose

This constitution defines the **core principles and standards** for [PROJECT_NAME]. All code, documentation, and processes **MUST** comply with these principles.

**This is not a suggestion - it is a REQUIREMENT.**

Every feature, every line of code, every decision must pass through this constitution.

---

## Core Principles

### Principle 0: Constitution Authority (NON-NEGOTIABLE)

**This constitution supersedes all other practices, preferences, or conventions.**

- Deviations MUST be explicitly documented and justified
- All PRs MUST demonstrate compliance
- Violations block merges
- Updates require [AMENDMENT_PROCESS]

### Principle 1: Complete Execution Requirement (NON-NEGOTIABLE)

NEVER adopt a "this is good enough" mindset. Work MAY pause due to token limits or technical constraints, but MUST always resume and continue forward progress. Settling for partial completion is strictly prohibited.

**Rules:**
- NEVER suggest stopping with partial implementation
- NEVER provide "this should work" or "this is sufficient" feedback when work remains
- If token limits are reached, explicitly state "pausing due to token limit, will continue"
- ALWAYS identify remaining work items before any pause
- Progress is the only acceptable direction—there is no "good enough" compromise
- When warnings remain, implementation remains incomplete
- When methods are unused, implementation remains incomplete
- Only completion of all requirements satisfies this principle
- Each resume MUST pick up exactly where previous work stopped
- "Suggestions" or "options to consider" are NOT acceptable when concrete work remains

**Rationale:** Half-done work creates technical debt, confusion, and maintenance burden. Complete execution ensures reliability and maintainability.

**How to Check:**
```bash
# Complete execution validation
[PROJECT_BUILD_COMMAND] 2>&1 | grep -i "error\|warning" && echo "INCOMPLETE" || echo "COMPLETE"
grep -r "TODO\|FIXME" [SOURCE_DIRECTORY]/ && echo "INCOMPLETE" || echo "COMPLETE"
[UNUSED_CODE_CHECK_COMMAND] && echo "COMPLETE" || echo "INCOMPLETE"
```

---

### I. [PRINCIPLE_1_NAME]

[PRINCIPLE_1_DESCRIPTION]

**Rules:**
[PRINCIPLE_1_RULES]

**Rationale:** [PRINCIPLE_1_RATIONALE]

**How to Check:**
[PRINCIPLE_1_CHECKS]

---

### II. [PRINCIPLE_2_NAME]

[PRINCIPLE_2_DESCRIPTION]

**Rules:**
[PRINCIPLE_2_RULES]

**Rationale:** [PRINCIPLE_2_RATIONALE]

**How to Check:**
[PRINCIPLE_2_CHECKS]

---

### III. [PRINCIPLE_3_NAME]

[PRINCIPLE_3_DESCRIPTION]

**Rules:**
[PRINCIPLE_3_RULES]

**Rationale:** [PRINCIPLE_3_RATIONALE]

**How to Check:**
[PRINCIPLE_3_CHECKS]

---

### IV. [PRINCIPLE_4_NAME]

[PRINCIPLE_4_DESCRIPTION]

**Rules:**
[PRINCIPLE_4_RULES]

**Rationale:** [PRINCIPLE_4_RATIONALE]

**How to Check:**
[PRINCIPLE_4_CHECKS]

---

### V. [PRINCIPLE_5_NAME]

[PRINCIPLE_5_DESCRIPTION]

**Rules:**
[PRINCIPLE_5_RULES]

**Rationale:** [PRINCIPLE_5_RATIONALE]

**How to Check:**
[PRINCIPLE_5_CHECKS]

---

## Quality Gates

**Pre-commit Requirements**
[PRE_COMMIT_GATES]

**Pre-release Requirements**
[PRE_RELEASE_GATES]

---

## Development Workflow

**Making Changes**
[DEVELOPMENT_WORKFLOW]

**Code Review Focus**
[CODE_REVIEW_FOCUS]

---

## Governance

**Constitution Authority**
[GOVERNANCE_AUTHORITY]

**Amendment Procedure**
[AMENDMENT_PROCEDURE]

**Continuous Improvement**
[CONTINUOUS_IMPROVEMENT]

---

**Amendment History**:
[AMENDMENT_HISTORY]
