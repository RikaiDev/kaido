---
description: Analyze changes and generate release documentation (CHANGELOG/RELEASE_NOTES) based on project conventions
---

## ⚠️ CRITICAL DATE REQUIREMENT

**YOU MUST USE REAL DATES - NOT PLACEHOLDERS**

Before proceeding, execute this command in your terminal to get today's date:
```bash
date +%Y-%m-%d
```

This returns the current date in YYYY-MM-DD format (e.g., 2024-01-15).

**NEVER** copy placeholder dates like `[DATE]`, `YYYY-MM-DD`, or `[RELEASE_DATE]` from templates.
**ALWAYS** run the `date` command above to get the actual current date.
**DO NOT** guess or fabricate dates - execute the command and use the real output.

## User Input

```text
Version: $VERSION (optional, will be determined if not provided)
Tag: $TAG (boolean, default: false)
Push: $PUSH (boolean, default: false)
```

## Process

### Step 1: Detect Project Conventions

Execute ProjectDetector to understand project structure:

**Actions**:
1. Check for CHANGELOG.md, RELEASE_NOTES.md in root
2. Analyze last 50 git commits for format patterns
3. Check package.json for commitlint, husky, semantic-release
4. Read .cortex/templates/constitution.md for release rules
5. Check .cortex/workflows/ for workflow documents

**Output**: ProjectConventions object with:
- Documentation files present
- Commit format (conventional vs freeform)
- Toolchain configuration
- Cortex workflow availability
- Constitution release rules

### Step 2: Determine Documentation Strategy

Based on ProjectConventions, decide what to generate:

**If hasChangelog**:
- Update CHANGELOG.md with new version section

**If hasReleaseNotes**:
- Generate/update RELEASE_NOTES.md

**If both**:
- Update both files

**If neither**:
- Ask user: "No release documentation found. Create CHANGELOG.md or RELEASE_NOTES.md?"
- Default: CHANGELOG.md
- Create file with proper format

### Step 3: Analyze Changes

Execute ChangeAnalyzer with detected strategy:

**Strategy A (Git-based)** - For traditional projects:
```bash
# Get commits since last tag
git log $(git describe --tags --abbrev=0)..HEAD --oneline
```
- Parse conventional commit types
- Extract breaking changes (BREAKING CHANGE or !)
- Categorize by type: feat, fix, docs, refactor, chore

**Strategy B (Cortex-based)** - For Cortex projects:
- Read `.cortex/workflows/*/spec.md` for completed features
- Read `.cortex/workflows/*/tasks.md` for task details
- Extract User Stories and acceptance criteria

**Strategy C (Hybrid)** - Best of both:
- Use Cortex workflows for high-level feature descriptions
- Use git commits for technical details
- Merge and deduplicate

**Collect**:
- Feature additions (feat commits or completed US-xxx)
- Bug fixes (fix commits or bug fix tasks)
- Breaking changes (! or BREAKING CHANGE)
- Documentation updates
- Technical improvements (refactor, perf, chore)

### Step 4: Generate Documentation

**For CHANGELOG.md format**:
```markdown
## [VERSION] - YYYY-MM-DD

### Added
- New feature descriptions from workflows or feat commits
- List user-visible improvements

### Fixed
- Bug fix descriptions
- Issue references if available

### Changed
- Breaking changes with migration guide
- API changes

### Technical
- Internal improvements
- Refactoring
- Performance optimizations
- Dependency updates
```

**For RELEASE_NOTES.md format**:
```markdown
# Release vVERSION - YYYY-MM-DD

## Highlights
- 2-3 major features (from workflows or most important commits)

## What's New
- Detailed feature descriptions with context
- Why this matters to users

## Bug Fixes
- Critical fixes
- Known issues resolved

## Breaking Changes
- What changed
- How to migrate
- Code examples if needed

## Technical Notes
- Architecture changes
- Performance improvements
- Internal refactoring
```

**Writing Guidelines**:
- Use active voice ("Added X" not "X was added")
- Be specific ("Added JWT authentication" not "Improved auth")
- Include user impact ("Reduces load time by 50%")
- Reference issues/PRs if conventional (e.g., "Fixes #123")

### Step 5: Validate Quality (CRITICAL - NO EXCEPTIONS)

Run ALL validations with NO ATTEMPT LIMIT:

**A. Mock/Scaffold Detection** (BLOCKER):
```bash
# Scan entire src/ for forbidden patterns
grep -rn "TODO\|FIXME\|Not implemented\|mock.*=\|MOCK_\|placeholder" src/
```

**If ANY found**:
- List ALL occurrences with file:line
- Create fix task for EACH occurrence
- BLOCK release completely
- Message: "❌ RELEASE BLOCKED: Incomplete implementation detected"
- Show: "Found N issues that must be fixed before release"
- DO NOT ask "skip these?"
- DO NOT proceed
- STOP and wait for fixes

**B. Unused Code Detection** (BLOCKER):
```bash
npm run knip
```

**If unused exports found**:
- List ALL unused exports
- For each: determine if truly unused or needed for API
- If truly unused → DELETE immediately
- If part of public API → document why in code
- Re-run knip until ZERO unused
- BLOCK release until clean

**C. Markdown Lint Check** (BLOCKER):
```bash
npx markdownlint "**/*.md" --config .markdownlint.json --ignore node_modules
```

**If ANY warnings**:
- Fix ALL automatically if possible
- Re-run check
- Repeat until ZERO warnings

**D. Linter Check** (BLOCKER):
```bash
npm run build
```

**If ANY errors or warnings**:
- Show ALL errors
- Fix them
- Re-run build
- Repeat until clean build

**E. Constitution Compliance** (BLOCKER):
- Read .cortex/templates/constitution.md
- Verify all release principles met
- Check test coverage if required
- Check documentation if required
- BLOCK if any principle violated

**CRITICAL RULES**:
- Run ALL checks (A, B, C, D, E) in sequence
- If ANY check fails → fix and re-run ALL from start
- NO LIMIT on attempts
- NO "good enough" - must be PERFECT
- NO asking user to skip
- NO proceeding with warnings

### Step 6: Generate Commit Message

Analyze changes and create professional commit message:

**Format** (Conventional Commits):
```
<type>(<scope>): <description>

<body>

- Change 1
- Change 2
- Change 3
```

**Determine type**:
- `chore(release)`: If only version/docs changes
- `feat(release)`: If includes new features
- `fix(release)`: If primarily bug fixes

**Description**: Clear, concise summary (max 72 chars)
**Body**: Detailed explanation of changes

**Example**:
```
chore(release): prepare release v1.2.0

Release includes:
- Added JWT authentication system
- Fixed memory leak in data processing
- Improved error handling in API layer
- Updated dependencies to latest versions

Breaking Changes:
- Auth API now requires JWT tokens instead of session cookies
```

### Step 7: Confirm and Execute

**Display to user**:
```
📋 RELEASE SUMMARY

Version: vX.Y.Z
Files changed:
  - CHANGELOG.md (updated)
  - package.json (if version bump)
  - README.md (if version badge update)

Changes:
  - M features added
  - N bugs fixed
  - O technical improvements

Commit message:
  <type>(<scope>): <description>
  ...

Quality gates: ✅ ALL PASSED
  ✅ No TODOs/mocks
  ✅ No unused code
  ✅ Markdown lint clean
  ✅ Build successful
  ✅ Constitution compliant

Actions to perform:
  1. git add [files]
  2. git commit -m "[message]"
  3. git tag vX.Y.Z (if --tag specified)
  4. git push && git push --tags (if --push specified)

Proceed with release? [y/N]
```

**Wait for explicit user confirmation.**

**If confirmed**:
1. `git add CHANGELOG.md` (and other changed files)
2. `git commit -m "<generated message>"`
3. If --tag: `git tag vX.Y.Z`
4. If --push: `git push && git push --tags`
5. Show success message with next steps

**If rejected**:
- Ask what to change
- Regenerate and re-confirm

### Step 8: Record Experience

Use cortex.learn to save release metadata:

**Record**:
```json
{
  "type": "release",
  "version": "X.Y.Z",
  "conventions": {
    "hasChangelog": true,
    "usesConventionalCommits": true,
    "strategy": "hybrid"
  },
  "changes": {
    "features": M,
    "fixes": N,
    "breaking": O
  },
  "qualityChecks": {
    "mocksFound": 0,
    "unusedCode": 0,
    "lintErrors": 0
  },
  "success": true,
  "notes": "All quality gates passed on first attempt"
}
```

This helps future releases learn from patterns.

## CRITICAL RULES (NON-NEGOTIABLE)

1. **MUST detect before generating** - Never assume project structure
2. **MUST adapt to conventions** - Follow project's existing patterns
3. **MUST validate until perfect** - No attempt limit, must be 100% clean
4. **MUST follow constitution** - All principles must be met
5. **MUST get user confirmation** - Never auto-commit without approval
6. **NEVER skip quality validation** - All checks must pass
7. **NEVER ask "skip these checks?"** - This is forbidden
8. **NEVER proceed with warnings** - Fix ALL issues first
9. **NEVER generate TODO comments** - Everything must be complete
10. **NEVER accept mock data** - All implementations must be real

## Error Handling

**If quality checks fail repeatedly**:
- DO NOT ask to skip
- DO NOT simplify
- INSTEAD:
  - List ALL remaining issues
  - Create fix tasks for each
  - Save current progress
  - Message: "Release paused. Created N fix tasks. Resume with cortex.release after fixes."
  - STOP and wait

**If git operations fail**:
- Show exact error
- Suggest fix
- Wait for user to resolve
- Retry after confirmation

**If unable to determine version**:
- Check last git tag
- Check package.json version
- Ask user for version number
- Validate format (semver)

