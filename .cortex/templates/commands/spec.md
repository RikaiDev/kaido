---
description: Create feature specification from user description using structured template with placeholders
---

## ⚠️ CRITICAL DATE REQUIREMENT

**YOU MUST USE REAL DATES - NOT PLACEHOLDERS**

Before proceeding, execute this command in your terminal to get today's date:
```bash
date +%Y-%m-%d
```

This returns the current date in YYYY-MM-DD format (e.g., 2024-01-15).

**NEVER** copy placeholder dates like `[DATE]`, `YYYY-MM-DD`, or `[CREATION_DATE]` from templates.
**ALWAYS** run the `date` command above to get the actual current date.
**DO NOT** guess or fabricate dates - execute the command and use the real output.

## User Input

```text
$ARGUMENTS
```

You **MUST** use this description to generate the specification.

## Outline

You are creating a feature specification at `.cortex/workflows/[WORKFLOW_ID]/spec.md` from the template at `.cortex/templates/spec-template.md`. Follow this execution flow:

### 1. Load templates and context

- Read `.cortex/templates/spec-template.md`
- Identify all placeholder tokens: `[ALL_CAPS_IDENTIFIER]`
- Read `.cortex/constitution.md` for principle checks
- Query `.cortex/memory/` for relevant past experiences

### 2. Analyze user description

**Extract from user input:**
- Core feature purpose
- Target users
- Key functionality
- Technical constraints (if mentioned)
- Success metrics (if mentioned)

**Infer missing details:**
- Likely user stories (at least 1-3, prioritized)
- Acceptance scenarios for each story
- Functional requirements
- Key entities/data structures
- Success criteria

### 3. Fill placeholder values

**Metadata:**
- `FEATURE_NAME`: Extract concise name from description (2-5 words)
- `BRANCH_NAME`: Generate from feature name (e.g., "001-feature-name")
- `DATE`: Today's date (YYYY-MM-DD)
- `WORKFLOW_ID`: Generate UUID or use provided ID

**User Stories (P1, P2, P3...):**
For each story:
- `USX_TITLE`: Concise title (3-6 words)
- `USX_DESCRIPTION`: 1-2 paragraphs explaining the user journey
- `USX_PRIORITY_RATIONALE`: Why this priority level (1-2 sentences)
- `USX_TEST_DESCRIPTION`: How to independently test (specific actions)
- `USX_SCENARIO_X_GIVEN/WHEN/THEN`: Concrete acceptance scenarios

**Rules for User Stories:**
- MUST be prioritized (P1 = MVP, P2/P3 = enhancements)
- MUST be independently testable
- MUST use Given/When/Then format for scenarios
- MUST explain priority rationale

**Requirements:**
- `FUNCTIONAL_REQUIREMENTS`: Bulleted list starting with FR-001, FR-002...
  - Format: `- **FR-XXX**: System MUST [specific, testable requirement]`
  - Be specific, avoid vague language
  - Mark unclear items: `[NEEDS CLARIFICATION: what's unclear]`

**Entities:**
- `KEY_ENTITIES`: List data structures/objects needed
  - Format: `- **EntityName**: Description of what it represents`
  - Include key attributes WITHOUT implementation details
  - Note relationships to other entities

**Success Criteria:**
- `SUCCESS_CRITERIA`: Measurable outcomes
  - Format: `- **SC-XXX**: [Specific, measurable metric]`
  - Must be technology-agnostic
  - Must be objectively verifiable

**Constitution Check:**
- `CONSTITUTION_CHECK`: For each relevant principle from constitution:
  - Explain how this spec complies
  - Flag potential violations early
  - Format: `**[Principle Name]**: [How spec addresses it]`

**Experiences:**
- `RELEVANT_EXPERIENCES`: Query memory system
  - Search for patterns/solutions related to feature
  - Include 3-5 most relevant experiences
  - Format: `- [Experience Title]: path/to/experience.md (Relevance: [why])`

### 4. Validation before output

**MUST verify:**
- At least 1 user story (P1), preferably 2-3
- Each user story has 2+ acceptance scenarios
- 5+ functional requirements minimum
- 3+ success criteria minimum
- Constitution check covers Code Quality, Simplicity, Testing
- No placeholders remain unfilled
- All Given/When/Then scenarios are concrete (no vague language)

### 5. Write specification

- Create new workflow directory: `.cortex/workflows/[WORKFLOW_ID]/`
- Write filled template to `.cortex/workflows/[WORKFLOW_ID]/spec.md`
- Ensure valid markdown syntax

### 6. Output to user

```
## Specification Generated

**Workflow ID**: [WORKFLOW_ID]
**Feature**: [FEATURE_NAME]
**User Stories**: [COUNT] stories (P1: [COUNT], P2: [COUNT], ...)
**Requirements**: [COUNT] functional requirements
**Success Criteria**: [COUNT] criteria

### Constitution Validation
[✓/✗] Code Quality
[✓/✗] Simplicity
[✓/✗] Testing Standards

### Relevant Experiences Found
- [Count] pattern(s)
- [Count] solution(s)

### Generated Files
- spec.md ([LINE_COUNT] lines)

---

[WAITING FOR USER CONFIRMATION]

**Next Steps**:
1. Review specification at .cortex/workflows/[WORKFLOW_ID]/spec.md
2. Request clarifications or changes if needed
3. When ready, run: `cortex.plan [WORKFLOW_ID]` to create implementation plan
```

---

## Formatting & Style Requirements

- No emojis (professional tone)
- Use consistent heading levels
- Bullet points use `-` not `*`
- Code/commands in backticks
- Tables for comparisons (if needed)
- Keep paragraphs under 4 lines
- Blank line between sections

---

## Special Cases

**Vague Description:**
If user description is too vague:
- Make reasonable assumptions
- Mark uncertain areas with `[NEEDS CLARIFICATION: ...]`
- Generate at least 1 complete user story as example
- Ask clarifying questions in output

**Technical vs Feature Request:**
- Technical: Focus on system capabilities, performance, architecture
- Feature: Focus on user journeys, interactions, outcomes
- Adjust language and emphasis accordingly

**Single vs Multiple Features:**
If description contains multiple features:
- Break into separate user stories (different priorities)
- Note dependencies between stories
- Recommend splitting into multiple workflows if too large

---

## CRITICAL RULES

1. **Always generate at least 1 complete user story (P1)** - this is the MVP
2. **User stories MUST be independently testable** - no dependencies unless explicit
3. **Requirements MUST be specific and testable** - avoid "should", use "MUST"
4. **Constitution check is MANDATORY** - never skip this section
5. **Memory search is MANDATORY** - always query for relevant experiences
6. **No placeholders left unfilled** - all `[TOKENS]` must be replaced
7. **Acceptance scenarios MUST use Given/When/Then** - no exceptions
