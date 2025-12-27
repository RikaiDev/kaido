---
description: Guide new users through Cortex initialization and constitution creation via interactive Q&A
---

## Welcome Message

Display to user:
```
👋 Welcome to Cortex AI!

I'll help you set up your project with a structured development workflow.

This setup will create:
1. Project structure (.cortex/ directories)
2. Development workflow templates
3. Project constitution (your coding principles)

Estimated time: 5 minutes

Let's get started!
```

## Process

### Step 1: Check Current State

Detect project status:

**Check 1: Is .cortex/ already created?**
```bash
test -d .cortex && echo "exists" || echo "not found"
```

**Check 2: Project type detection**
- Look for package.json → Node.js/TypeScript project
- Look for requirements.txt → Python project
- Look for go.mod → Go project
- Look for Cargo.toml → Rust project
- Look for pom.xml / build.gradle → Java project

**Check 3: Existing files**
- README.md exists?
- .git/ exists?
- Tests exist?

**Output decision**:
- If .cortex/ exists → Skip to Step 3 (constitution only)
- If new project → Full setup
- If existing project → Setup + integrate

### Step 2: Initialize Structure (if needed)

**If .cortex/ doesn't exist**:

Execute initialization:
```bash
# Create directory structure
mkdir -p .cortex/{workflows,memory,templates,templates/commands}
```

**Copy templates**:
- constitution.md template
- spec-template.md
- plan-template.md
- tasks-template.md
- execution-template.md
- clarify-template.md
- review-template.md
- All command files (commands/*.md)

**Initialize memory**:
```json
{
  "version": "1.0.0",
  "initialized": "YYYY-MM-DD",
  "experiences": []
}
```

**Confirm to user**:
```
✅ Cortex workspace initialized!

Created:
  .cortex/
  ├── workflows/      (for feature development)
  ├── memory/         (for learning experiences)
  └── templates/      (for spec, plan, tasks)
      └── commands/   (AI execution guides)

Next: Let's create your project constitution...
```

### Step 3: Constitution Q&A

**Introduction**:
```
📜 Creating Your Project Constitution

The constitution defines your project's core principles.
I'll ask 5 questions to tailor it to your needs.

Answer with the letter (a, b, c...) or press Enter for default.
```

**Q1: Project Type**
```
What type of project is this?
  a) Web application (default)
  b) Library/Package
  c) CLI tool
  d) API service
  e) Mobile app
  f) Other

Your choice:
```

**Wait for response. Parse answer.**

**Q2: Code Quality Focus** (can select multiple)
```
What's your code quality priority? (select all that apply)
  a) Type safety (TypeScript strict mode, type checking) (default)
  b) Test coverage (comprehensive automated tests)
  c) Performance (optimization, profiling)
  d) Simplicity (KISS principle, avoid over-engineering)
  e) Security (input validation, auth, encryption)

Your choice (comma-separated, e.g., a,b,d):
```

**Wait for response. Parse comma-separated list.**

**Q3: Testing Approach**
```
What's your testing strategy?
  a) Test critical paths only (pragmatic) (default)
  b) High test coverage (>80% coverage target)
  c) Integration tests focused (end-to-end testing)
  d) Contract testing (API contracts, mocks for external)
  e) Manual testing primarily (QA team driven)

Your choice:
```

**Wait for response.**

**Q4: Release Process**
```
How do you want to manage releases?
  a) Manual changelog (write CHANGELOG.md manually) (default)
  b) Automated from commits (conventional commits → changelog)
  c) Formal release notes (detailed RELEASE_NOTES.md)
  d) Simple version tags (git tags only, no docs)

Your choice:
```

**Wait for response.**

**Q5: Documentation Style**
```
What documentation do you need?
  a) README + inline comments (standard) (default)
  b) Comprehensive API docs (JSDoc/Sphinx/godoc)
  c) Minimal (self-documenting code)
  d) User guides + technical docs (full documentation site)

Your choice:
```

**Wait for response.**

### Step 4: Generate Constitution

Based on answers, fill constitution template:

**Project Type mapping**:
- Web app → Focus on UX, performance, security
- Library → Focus on API stability, documentation, versioning
- CLI → Focus on simplicity, error messages, help text
- API → Focus on contracts, error handling, validation

**Code Quality mapping**:
- Type safety → Strict TypeScript config, no `any` types
- Test coverage → Minimum coverage thresholds
- Performance → Profiling requirements, benchmarks
- Simplicity → KISS principle, refactoring rules
- Security → Input validation, auth requirements

**Testing mapping**:
- Critical paths → Test user flows, edge cases
- High coverage → 80%+ coverage, pre-commit checks
- Integration → E2E tests, contract tests
- Contract → Mock external services, test contracts
- Manual → QA checklist, acceptance criteria

**Release mapping**:
- Manual changelog → Human-written, narrative style
- Automated → Conventional commits, semantic versioning
- Formal notes → Executive summaries, migration guides
- Simple tags → Version numbers, minimal docs

**Documentation mapping**:
- Standard → README, code comments, examples
- Comprehensive → Full API docs, tutorials
- Minimal → Type signatures, self-explanatory code
- Full → Docs site, guides, examples, FAQ

**Fill template at**: `.cortex/templates/constitution.md`

**Replace placeholders**:
- `[PROJECT_NAME]` → from package.json or directory name
- `[CONSTITUTION_VERSION]` → 1.0.0
- `[RATIFICATION_DATE]` → today (use `date +%Y-%m-%d`)
- `[LAST_AMENDED_DATE]` → today
- `[PRINCIPLE_X_NAME]` → based on selected priorities
- `[PRINCIPLE_X_DESCRIPTION]` → detailed guidance
- `[PRINCIPLE_X_RULES]` → specific rules
- `[PRINCIPLE_X_RATIONALE]` → why this matters
- `[PRINCIPLE_X_CHECKS]` → how to verify

**Always include** (NON-NEGOTIABLE):
- Principle VI: Real Implementation Integrity (from template)
  - No Mock, No Scaffold, No Half-Done
  - Task Decomposition Over Shortcuts
  - Zero tolerance for incomplete work

### Step 5: Confirmation

**Show summary**:
```
📋 Constitution Summary

Project: [NAME]
Type: [TYPE]
Version: 1.0.0

Core Principles:
  I. [PRINCIPLE_1_NAME]
  II. [PRINCIPLE_2_NAME]
  III. [PRINCIPLE_3_NAME]
  IV. [PRINCIPLE_4_NAME]
  V. [PRINCIPLE_5_NAME]
  VI. Real Implementation Integrity (required)

Quality Gates:
  - [PRE_COMMIT_GATES]
  - [PRE_RELEASE_GATES]

Review constitution at: .cortex/templates/constitution.md

Does this look good? (y/N)
```

**Wait for user response.**

**If 'y' or 'yes'**:
- Proceed to Step 6

**If 'n' or 'no'**:
```
What would you like to adjust?
  a) Change project type
  b) Adjust code quality focus
  c) Change testing approach
  d) Modify release process
  e) Update documentation style
  f) Start over

Your choice:
```
- Go back to relevant question
- Regenerate constitution
- Re-confirm

### Step 6: Next Steps

**Display completion message**:
```
✅ Cortex setup complete!

Your project now has:
  ✓ .cortex/ workflow structure
  ✓ Constitution with your coding principles
  ✓ Templates for spec, plan, tasks, implement
  ✓ Command guides for consistent AI behavior

📖 Quick Start:

1. Review your constitution:
   cat .cortex/templates/constitution.md

2. Start your first feature:
   cortex.spec "Add user authentication system"

3. Get help anytime:
   cortex.status   # Check current workflow
   cortex.list     # List available commands

4. Release your work:
   cortex.release  # Analyze changes and create release

🎯 Key Reminders:

- Your constitution is YOUR rules - AI follows them
- All implementations are COMPLETE (no TODOs, no mocks)
- Quality gates ensure consistency
- Workflow pauses for your confirmation

Happy coding with Cortex! 🚀
```

## CRITICAL RULES

1. **ONE question at a time** - Never batch questions
2. **MUST detect existing state** - Check before initializing
3. **MUST adapt to project type** - Different projects need different rules
4. **MUST generate complete constitution** - No placeholders left unfilled
5. **MUST get user confirmation** - Never finalize without approval
6. **NEVER skip onboarding** - First-time users need guidance
7. **ALWAYS include Real Implementation Integrity** - This principle is mandatory
8. **Wait for user input** - Don't assume answers

## Error Handling

**If directory creation fails**:
- Check permissions
- Show exact error
- Suggest fix (chmod, sudo)
- Retry

**If user provides invalid answer**:
- Show valid options again
- Don't proceed until valid input
- Provide examples

**If constitution generation fails**:
- Show error
- Offer to retry with different answers
- Don't leave partial files

**If user abandons onboarding**:
- Save progress in .cortex/.onboarding-state.json
- Offer to resume later
- Clean up if explicitly requested

