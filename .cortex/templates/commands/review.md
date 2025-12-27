---
description: Perform technical review of implementation plan covering architecture, security, performance, maintainability, data model, and integration
---

## ⚠️ CRITICAL DATE REQUIREMENT

**YOU MUST USE REAL DATES - NOT PLACEHOLDERS**

Before proceeding, execute this command in your terminal to get today's date:
```bash
date +%Y-%m-%d
```

This returns the current date in YYYY-MM-DD format (e.g., 2024-01-15).

**NEVER** copy placeholder dates like `[DATE]`, `YYYY-MM-DD`, or `[REVIEW_DATE]` from templates.
**ALWAYS** run the `date` command above to get the actual current date.
**DO NOT** guess or fabricate dates - execute the command and use the real output.

## Purpose

Conduct a comprehensive technical review of the implementation plan before proceeding to task breakdown. This ensures quality, identifies risks, and prevents costly issues during implementation.

## Execution Steps

### 1. Load Context

Load the following documents from the workflow directory:

- `spec.md` - Feature requirements and user stories
- `plan.md` - Technical implementation plan
- `data-model.md` (if exists) - Data structure design
- `contracts/` (if exists) - API contracts
- `.cortex/constitution.md` - Project principles and standards

### 2. Perform 6-Category Review

For each category, analyze the plan and provide:
- **Strengths**: What works well
- **Concerns**: Potential issues or risks
- **Recommendations**: Specific improvements with rationale

#### A. Architecture Review

**Focus Areas**:
- Component organization and separation of concerns
- System boundaries and integration points
- Scalability of the proposed architecture
- Complexity vs. requirements fit
- Adherence to established patterns

**Key Questions**:
- Is the architecture appropriate for the problem scale?
- Are components loosely coupled and highly cohesive?
- Are there any over-engineered or under-designed areas?
- Does it follow SOLID principles?

#### B. Security Review

**Focus Areas**:
- Authentication and authorization mechanisms
- Data protection (encryption, sanitization)
- Input validation and injection prevention
- Secrets management
- Attack surface analysis

**Key Questions**:
- Are user inputs validated and sanitized?
- Is sensitive data properly encrypted?
- Are authentication/authorization patterns secure?
- Are there any OWASP Top 10 vulnerabilities?

#### C. Performance Review

**Focus Areas**:
- Database query efficiency
- API response time considerations
- Resource usage (memory, CPU, network)
- Caching strategies
- Potential bottlenecks

**Key Questions**:
- Will this meet the performance targets in plan.md?
- Are there N+1 query problems?
- Is pagination/lazy loading considered?
- Are expensive operations optimized?

#### D. Maintainability Review

**Focus Areas**:
- Code organization and structure
- Testability of design
- Documentation completeness
- Technical debt assessment
- Long-term evolution considerations

**Key Questions**:
- Is the code structure easy to understand?
- Can components be tested in isolation?
- Is documentation sufficient for future developers?
- Are there any quick wins that create long-term pain?

#### E. Data Model Review

**Focus Areas**:
- Entity design and normalization
- Relationship modeling
- Constraints and validation
- Migration strategy
- Data integrity

**Key Questions**:
- Is the data model properly normalized?
- Are relationships correctly defined?
- Are constraints enforced at the right layer?
- Is the migration path clear?

#### F. Integration Review

**Focus Areas**:
- External service dependencies
- API design and versioning
- Error handling for external failures
- Fallback and retry strategies
- Service level agreements

**Key Questions**:
- What happens if external services fail?
- Are integrations properly abstracted?
- Is API versioning considered?
- Are there single points of failure?

### 3. Constitution Compliance Check

Load `.cortex/constitution.md` and check plan.md against each principle:

1. For each principle, evaluate:
   - ✅ **Compliant**: Plan follows this principle
   - ⚠️ **Violation**: Plan violates this principle
   - ℹ️ **N/A**: Principle doesn't apply to this feature

2. For each violation:
   - Check if it's justified in plan.md's "Complexity Tracking" table
   - If justified: Verify the justification is reasonable
   - If not justified: Flag as "Must Fix"

3. Generate compliance summary:
   - Total principles checked
   - Compliant count
   - Violations count
   - Justified violations count
   - Unjustified violations (blockers)

### 4. Categorize Action Items

Based on review findings, categorize action items:

**Must Fix (Blocking)**:
- Unjustified constitution violations
- Critical security vulnerabilities
- Architectural flaws that prevent implementation
- Performance issues that violate requirements

**Should Fix (Important)**:
- Technical debt that will compound
- Maintainability concerns
- Minor security improvements
- Performance optimizations

**Consider (Optional)**:
- Nice-to-have refactorings
- Future enhancements
- Alternative approaches to consider

### 5. Make Overall Decision

Based on the review, decide:

**APPROVED**:
- All "Must Fix" items are zero
- Architecture is sound
- Security is adequate
- Performance meets targets
- → Proceed to `cortex.tasks`

**APPROVED WITH CHANGES**:
- Few "Must Fix" items (1-3 minor ones)
- Generally sound design
- Changes can be made quickly
- → Address items, update plan.md, then proceed

**NEEDS MAJOR REVISION**:
- Multiple "Must Fix" items
- Architectural concerns
- Significant security issues
- → Requires substantial plan.md updates and re-review

### 6. Generate Review Document

Fill the review-template.md with:
- All category findings (strengths, concerns, recommendations)
- Constitution compliance results
- Categorized action items
- Overall decision with rationale
- Next steps

Save to: `.cortex/workflows/[WORKFLOW_ID]/review.md`

### 7. Generate Review Summary

Output a concise summary:

```markdown
## Technical Review Complete

**Status**: [APPROVED / APPROVED WITH CHANGES / NEEDS MAJOR REVISION]

**Summary**:
- ✅ [N] categories approved
- ⚠️ [N] categories need changes
- ❌ [N] categories need major revision

**Constitution**: [N]/[Total] principles compliant

**Action Items**:
- Must Fix: [N]
- Should Fix: [N]
- Consider: [N]

**Next Step**: [Specific instruction]
```

## Review Guidelines

### Be Constructive
- Focus on issues that matter
- Explain why something is a concern
- Suggest concrete improvements
- Acknowledge good design decisions

### Be Specific
- Quote relevant plan.md sections
- Reference specific files or components
- Provide examples of issues
- Suggest specific alternatives

### Be Pragmatic
- Consider project constraints
- Balance ideal vs. practical
- Recognize appropriate complexity
- Don't demand perfection

### Be Thorough but Efficient
- Focus on high-impact areas
- Don't nitpick minor style issues
- Flag systemic issues, not all instances
- Aim for 20-30 minute review time

## Common Red Flags

### Architecture
- God objects or classes
- Tight coupling between layers
- Missing abstraction layers
- Over-engineered simple features

### Security
- No input validation
- Passwords in plain text
- No rate limiting on APIs
- Missing authentication on endpoints

### Performance
- No pagination on large datasets
- N+1 queries in loops
- Unbounded memory usage
- Blocking operations on main thread

### Maintainability
- No tests planned
- Unclear naming conventions
- Missing error handling
- No logging strategy

## Example Review Flow

```
1. Load plan.md → Extract tech stack, architecture
2. Review Architecture → Check component design
   - Strength: Clean separation of concerns
   - Concern: Tight coupling between User and Order services
   - Recommendation: Introduce events for decoupling
3. Review Security → Check auth patterns
   - Strength: JWT auth with refresh tokens
   - Concern: No rate limiting on login endpoint
   - Recommendation: Add rate limiting middleware
4. ... (continue for all categories)
5. Check Constitution → 8/10 principles compliant
   - Violation: Repository pattern adds unnecessary complexity
   - Not justified in Complexity Tracking table
   - Action: Add to "Must Fix"
6. Decision: APPROVED WITH CHANGES
   - 2 Must Fix items
   - Update plan.md to address concerns
   - Then proceed to tasks
```

## After Review

If **APPROVED** or **APPROVED WITH CHANGES**:
- User addresses action items
- plan.md is updated
- Review document is marked complete
- Proceed to `cortex.tasks [WORKFLOW_ID]`

If **NEEDS MAJOR REVISION**:
- User makes substantial changes to plan.md
- Re-run `cortex.review [WORKFLOW_ID]`
- Continue until approved

