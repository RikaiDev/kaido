---
description: Resolve specification ambiguities through structured Q&A process, scanning 11 categories and updating spec with clarifications
---

## ⚠️ CRITICAL DATE REQUIREMENT

**YOU MUST USE REAL DATES - NOT PLACEHOLDERS**

Before proceeding, execute this command in your terminal to get today's date:
```bash
date +%Y-%m-%d
```

This returns the current date in YYYY-MM-DD format (e.g., 2024-01-15).

**NEVER** copy placeholder dates like `[DATE]`, `YYYY-MM-DD`, or `[UPDATE_DATE]` from templates.
**ALWAYS** run the `date` command above to get the actual current date.
**DO NOT** guess or fabricate dates - execute the command and use the real output.

## Purpose

Identify and resolve ambiguities in the feature specification through systematic scanning and targeted questioning.

## Execution Steps

### 1. Load Specification

Load the spec.md file from the workflow directory.

### 2. Perform Ambiguity Scan

Scan the specification against these 11 categories:

#### A. Functional Scope & Behavior
- Are core user goals clear?
- Is out-of-scope explicitly declared?
- Are user roles/personas differentiated?

#### B. Domain & Data Model
- Are entities, attributes, relationships clear?
- Are identity & uniqueness rules defined?
- Are state transitions specified?
- Are data volume assumptions stated?

#### C. Interaction & UX Flow
- Are critical user journeys clear?
- Are error/empty/loading states defined?
- Are accessibility requirements noted?

#### D. Non-Functional Quality Attributes
- Are performance targets specified (latency, throughput)?
- Are scalability requirements clear?
- Are reliability & availability expectations stated?
- Are observability requirements (logging, metrics) defined?
- Are security & privacy requirements clear?
- Are compliance constraints noted?

#### E. Integration & External Dependencies
- Are external services/APIs and failure modes clear?
- Are data import/export formats specified?
- Are protocol/versioning assumptions stated?

#### F. Edge Cases & Failure Handling
- Are negative scenarios covered?
- Is rate limiting/throttling specified?
- Is conflict resolution defined (e.g., concurrent edits)?

#### G. Constraints & Tradeoffs
- Are technical constraints stated?
- Are explicit tradeoffs documented?
- Are rejected alternatives noted?

#### H. Terminology & Consistency
- Are key terms defined in a glossary?
- Are synonyms avoided/deprecated terms noted?

#### I. Completion Signals
- Are acceptance criteria testable?
- Are Definition of Done indicators measurable?

#### J. Placeholders & TODOs
- Are there TODO markers?
- Are there unresolved decisions?

#### K. Vague Language
- Are there ambiguous adjectives ("robust", "intuitive")?
- Are there unquantified requirements?

### 3. Generate Prioritized Questions

For each category with **Partial** or **Missing** status:

1. Calculate priority: **Impact × Uncertainty**
2. Consider:
   - Does answer materially change implementation?
   - Does answer affect architecture, data modeling, task decomposition, test design, UX behavior, operational readiness, or compliance?
3. Skip if:
   - Already answered
   - Trivial stylistic preference
   - Better deferred to planning phase
   - Would not reduce downstream rework risk

**Generate maximum 5 questions total.**

### 4. Sequential Questioning Loop

For each question (up to 5 total):

#### Step A: Present ONE Question

**For multiple-choice questions:**

1. Analyze all options
2. Determine the most suitable option based on:
   - Best practices for the project type
   - Common patterns in similar implementations
   - Risk reduction (security, performance, maintainability)
   - Alignment with project goals/constraints

3. Present your recommendation:

```markdown
**Recommended**: Option [X] - <brief reasoning 1-2 sentences>

| Option | Description |
|--------|-------------|
| A | <Option A description> |
| B | <Option B description> |
| C | <Option C description> |
| Short | Provide a different short answer (≤5 words) |

You can reply with the option letter (e.g., "A"), accept the recommendation by saying "yes" or "recommended", or provide your own short answer.
```

**For short-answer questions:**

1. Provide your suggested answer based on best practices and context

2. Format:

```markdown
**Suggested**: <your proposed answer> - <brief reasoning>

Format: Short answer (≤5 words). You can accept the suggestion by saying "yes" or "suggested", or provide your own answer.
```

#### Step B: Wait for User Response

- If user says "yes", "recommended", or "suggested": Use your recommendation/suggestion
- Otherwise: Validate answer maps to option or fits ≤5 word constraint
- If ambiguous: Ask for quick disambiguation (doesn't count as new question)

#### Step C: Integrate Answer Immediately

1. Record in clarifications document:
   - Question
   - Answer
   - Which spec section updated

2. Update the most appropriate spec section(s):
   - **Functional ambiguity** → Functional Requirements
   - **User interaction/actor** → User Stories or Actors
   - **Data shape/entities** → Data Model (add fields, types, relationships)
   - **Non-functional** → Quality Attributes (convert vague to measurable)
   - **Edge case** → Edge Cases / Error Handling
   - **Terminology** → Normalize across spec

3. Save spec.md immediately after each integration

4. **Validation after each write**:
   - Clarification recorded in session
   - Relevant spec section updated
   - No contradictory statements remain
   - Terminology consistent

#### Step D: Stop Conditions

Stop asking when:
- All critical ambiguities resolved early
- User signals completion ("done", "good", "no more")
- Reached 5 asked questions
- No more valid questions exist

### 5. Generate Coverage Summary

After questioning loop ends:

Create a table showing each category's status:

| Category | Status | Notes |
|----------|--------|-------|
| Functional Scope & Behavior | Resolved/Deferred/Clear/Outstanding | |
| Domain & Data Model | Resolved/Deferred/Clear/Outstanding | |
| ... | ... | ... |

### 6. Final Report

Output:
- Number of questions asked & answered
- Path to updated spec.md
- Path to clarifications document
- Sections touched (list names)
- Coverage summary
- Suggested next command: `/cortex.plan` or `/cortex.clarify` (if significant Outstanding remain)

## Important Rules

1. **Maximum 5 questions total** across entire session
2. **One question at a time** - never reveal future questions
3. **Immediate integration** - update spec after each answer
4. **Preserve structure** - don't reorder unrelated sections
5. **Atomic saves** - save after each update to minimize risk
6. **No speculation** - only ask high-impact questions
7. **Respect early termination** - user can say "stop" anytime

## Example Flow

```
1. Load spec.md
2. Scan: Found 3 categories with ambiguity
3. Generate 3 questions (prioritized by Impact × Uncertainty)
4. Ask Question 1 → User answers → Update spec.md → Save
5. Ask Question 2 → User answers → Update spec.md → Save
6. Ask Question 3 → User says "done"
7. Generate coverage summary
8. Report: 2 questions asked, 2 answered, 3 sections updated
9. Suggest: `/cortex.plan` to proceed
```

## No Critical Ambiguities Case

If scan reveals all categories are **Clear** or only **Low Impact**:

1. Output: "No critical ambiguities detected worth formal clarification."
2. Coverage summary (all Clear)
3. Suggest: Proceed to `/cortex.plan`

## High Outstanding Case

If after 5 questions, high-impact categories remain **Outstanding**:

1. Flag them explicitly in coverage summary
2. Explain why deferred (quota reached)
3. Recommend: Run `/cortex.clarify` again after `/cortex.plan` if needed

