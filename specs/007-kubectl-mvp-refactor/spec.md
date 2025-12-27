# Feature Specification: Kubectl-Only MVP (60-Day Reality Check)

**Feature Branch**: `007-kubectl-mvp-refactor`  
**Created**: 2025-10-25  
**Status**: Draft  
**Input**: User description: "60-Day Kubectl-Only MVP - Linus-style refactoring to focus on kubectl natural language interface only"

## Clarifications

### Session 2025-10-25

- Q: For production safety, should we require typed confirmation (e.g., typing "production" or the resource name) or is a simple yes/no sufficient? → A: Risk-based tiered approach - HIGH risk requires typed confirmation, MEDIUM risk uses yes/no, LOW risk has no confirmation. This aligns with the core value of "risk levels" similar to AI Act.
- Q: Should the audit log be queryable through the TUI interface, or is file-based logging sufficient for MVP? → A: Hybrid approach - SQLite file storage with basic TUI filters (today, last week, production only), advanced queries use external tools.
- Q: When AI translation accuracy is below user expectations, should we show confidence scores to set expectations? → A: Show confidence score only when low (<70%), warning users to carefully review the command before execution.
- Q: Should MVP use OpenAI API only or support local models? → A: **Local GGUF models are PRIMARY** (enterprise privacy requirement), OpenAI API is FALLBACK only. This is critical for enterprise adoption where data privacy is non-negotiable.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Natural Language Kubectl Commands (Priority: P1)

A DevOps engineer types `kaido> list all pods in production` and the system translates it to `kubectl get pods -n prod`, executes it, and displays results. For destructive commands like `kaido> delete all pods`, the system shows a safety confirmation modal before execution.

**Why this priority**: This is the core value proposition - kubectl operations via natural language with built-in safety. Without this, there is no product.

**Independent Test**: Can be fully tested by running 10-20 common kubectl commands via natural language prompts and verifying correct translation and execution. Delivers immediate value by reducing kubectl syntax lookup time.

**Acceptance Scenarios**:

1. **Given** user has active kubeconfig, **When** user types "show pods", **Then** system executes `kubectl get pods` and displays output without confidence score (high confidence)
2. **Given** user types "list deployments in production", **When** system detects production context, **Then** system translates to `kubectl get deployments -n production`
3. **Given** user types ambiguous command like "show logs", **When** AI confidence is <70%, **Then** system displays confidence score with warning "⚠️ Low confidence (65%) - Please review command carefully"
4. **Given** user types "delete all pods", **When** system detects high-risk command, **Then** system shows safety modal requiring explicit confirmation
5. **Given** user confirms destructive action, **When** command executes, **Then** system logs the action with timestamp, command, and user context

---

### User Story 2 - Environment-Aware Safety Controls (Priority: P2)

A DevOps engineer attempts to run a destructive command (delete, drain, scale to zero) in a production environment. The system detects the production context and enforces stricter safety checks, preventing accidental disasters.

**Why this priority**: Production safety is essential for enterprise adoption. Without it, users won't trust the tool for production use.

**Independent Test**: Can be tested by attempting various destructive commands across dev/staging/prod contexts and verifying appropriate risk levels and confirmation requirements. Delivers value by preventing costly production incidents.

**Acceptance Scenarios**:

1. **Given** current context is "prod-cluster", **When** user attempts "delete deployment" (HIGH risk), **Then** system shows "HIGH RISK" warning with cluster name and requires typed confirmation (user must type resource name or "production")
2. **Given** current context is "dev-cluster", **When** user attempts "scale to 2 replicas" (MEDIUM risk), **Then** system shows "MEDIUM RISK" with simple yes/no confirmation
3. **Given** user types read-only command like "get pods" (LOW risk), **When** any environment, **Then** system executes immediately without modal
4. **Given** HIGH risk command in production, **When** user types incorrect confirmation text, **Then** command is rejected and user must retry with correct text
5. **Given** safety modal is displayed, **When** user cancels, **Then** command is aborted and logged as "cancelled"

---

### User Story 3 - Complete Command History and Audit Trail (Priority: P2)

A team lead needs to understand what commands were executed in the last week, by whom, in which environments, and what the outcomes were. The system provides audit logs stored in SQLite with basic TUI query commands for common filters.

**Why this priority**: Compliance and debugging require complete audit trails. Critical for enterprise use cases and post-incident investigation.

**Independent Test**: Can be tested by executing various commands, then querying via TUI basic filters (today, last week, production), and verifying SQLite file is accessible via external tools. Delivers value for compliance and incident investigation.

**Acceptance Scenarios**:

1. **Given** user has executed multiple commands, **When** user types "kaido> show history today", **Then** system displays chronological list of today's commands with timestamp, command, environment, and result
2. **Given** audit log exists, **When** user types "kaido> show history production", **Then** only production environment commands are displayed
3. **Given** user types "kaido> show history last week", **When** querying, **Then** system displays all commands from the last 7 days
4. **Given** command was executed, **When** viewing audit entry, **Then** shows original natural language input, translated kubectl command, execution result, risk level, and user action
5. **Given** user needs advanced query (e.g., by command type), **When** using SQLite browser or SQL, **Then** audit database is accessible with complete schema documentation
6. **Given** user cancels a command, **When** viewing audit log, **Then** cancelled commands are included with "CANCELLED" status

---

### User Story 4 - AI Translation Feedback Loop (Priority: P3)

When the AI mistranslates a command (e.g., user types "show logs" but AI generates incomplete `kubectl logs` without pod name), the user corrects it by editing the command. The system logs this correction for future improvement.

**Why this priority**: Enables continuous improvement of translation accuracy through real user corrections. Not critical for MVP but valuable for long-term quality.

**Independent Test**: Can be tested by intentionally providing ambiguous commands, correcting the AI's translation, and verifying corrections are logged. Delivers value by improving translation accuracy over time.

**Acceptance Scenarios**:

1. **Given** AI generates ambiguous command, **When** user edits before execution, **Then** system logs both AI output and user correction
2. **Given** feedback log exists, **When** reviewing corrections, **Then** common correction patterns are identifiable for model improvement
3. **Given** user types ambiguous request like "scale app", **When** AI detects ambiguity, **Then** system prompts for clarification (replica count, which deployment)

---

### Edge Cases

- What happens when user's kubeconfig has no current context set? (System should prompt user to select context)
- How does system handle kubectl commands that require interactive input? (System should warn that interactive commands are not yet supported)
- What if the AI translation API is unavailable? (System should show graceful error and allow direct kubectl command input as fallback)
- What happens when user has insufficient kubectl permissions? (System displays kubectl's error message and suggests permission check)
- How does system handle very long kubectl output (1000+ pods)? (System paginates output or provides filtering options)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST detect and parse the user's current kubectl context (cluster, namespace) from kubeconfig
- **FR-002**: System MUST translate natural language input to valid kubectl commands with 80% accuracy for common operations (get, describe, logs, delete, scale, apply)
- **FR-003**: System MUST classify all kubectl commands into risk levels: LOW (read-only, e.g., get, describe), MEDIUM (modify, e.g., scale, restart, apply), HIGH (destructive, e.g., delete, drain)
- **FR-004**: System MUST implement risk-based confirmation strategy: HIGH risk requires typed confirmation (resource name or environment name), MEDIUM risk requires yes/no confirmation, LOW risk executes without confirmation
- **FR-005**: System MUST log all executed commands with: timestamp, natural language input, translated command, execution result, environment context, risk level, and user identifier
- **FR-006**: System MUST execute kubectl commands and display output in the TUI with proper formatting
- **FR-007**: System MUST support at least 20 common kubectl operations: get (pods, deployments, services, nodes, namespaces), describe, logs, scale, restart, apply, port-forward, exec, delete
- **FR-008**: System MUST reject HIGH risk commands if typed confirmation does not match expected text (resource name or "production" for prod environments)
- **FR-009**: System MUST handle kubectl errors gracefully and display user-friendly error messages
- **FR-010**: System MUST allow users to edit AI-generated commands before execution
- **FR-011**: System MUST log user corrections to AI translations for future model improvement
- **FR-012**: System MUST provide direct kubectl command input as fallback when AI translation fails
- **FR-013**: System MUST complete command translation and display within 5 seconds
- **FR-014**: System MUST use local GGUF models (llama.cpp) as primary translation method for privacy, with OpenAI GPT-4 API as fallback when local model unavailable or fails
- **FR-015**: System MUST work with standard kubectl installations without requiring additional cluster-side components
- **FR-016**: System MUST support basic TUI audit log queries: "show history today", "show history last week", "show history production"
- **FR-017**: System MUST store audit logs in SQLite database with documented schema accessible to external SQL tools
- **FR-018**: System MUST display audit log entries with all fields: timestamp, natural language input, kubectl command, environment, result, risk level, user action
- **FR-019**: System MUST calculate and track confidence score for each AI translation
- **FR-020**: System MUST display confidence warning when translation confidence is below 70%, showing score and advisory message
- **FR-021**: System MUST NOT display confidence score when translation confidence is 70% or above, maintaining clean UI for confident translations

### Key Entities

- **KubectlCommand**: Represents a translated kubectl command with attributes: original natural language input, translated command string, risk level (low/medium/high), confidence score (0-100%), target environment, timestamp
- **ExecutionResult**: Represents the outcome of a kubectl command execution with attributes: command reference, exit code, stdout/stderr output, execution duration, timestamp
- **AuditLogEntry**: Represents a single audit record with attributes: user identifier, timestamp, natural language input, kubectl command, environment context (cluster/namespace), execution result, risk level, confidence score, user action (executed/cancelled/edited)
- **KubernetesContext**: Represents the current kubectl context with attributes: cluster name, namespace, user, environment type (dev/staging/prod)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can execute 50 different kubectl operations via natural language within 60 days of launch
- **SC-002**: Zero production incidents caused by the tool within the first 60 days of beta testing
- **SC-003**: Command translation and execution completes in under 5 seconds for 95% of requests
- **SC-004**: At least 3 out of 5 beta users remain active (using the tool weekly) after 30 days
- **SC-005**: At least 2-3 DevOps engineers are willing to pay $50/month after 60-day trial period
- **SC-006**: 80% of natural language commands translate to correct kubectl commands without user correction
- **SC-007**: Users report reduced time for kubectl operations by at least 30% compared to manual command construction
- **SC-008**: Audit log captures 100% of executed commands with complete context information

## Assumptions

1. Target users are DevOps engineers who already use kubectl regularly and understand Kubernetes concepts
2. Users have kubectl installed and configured with valid kubeconfig files
3. Users have API keys for OpenAI GPT-4 (or organization provides them)
4. Beta testing will be conducted with 5 users from personal network within first 30 days
5. Industry-standard practice for audit log retention is 90 days (configurable by organization)
6. Production environments are identified by context names containing "prod", "production", or explicit environment labels
7. User authentication/identity comes from system user (not implementing separate user management in MVP)
8. Performance target of <5 seconds is acceptable for DevOps workflows (not real-time requirement)
9. Initial version only supports English natural language input
10. Common kubectl operations cover 80% of daily DevOps tasks for target users

## Scope Boundaries

### In Scope
- kubectl natural language interface (read and write operations)
- Safety modals for destructive commands
- Environment detection from kubeconfig
- Audit logging to SQLite
- TUI interface for command input and output
- Integration with OpenAI GPT-4 API

### Out of Scope (Deferred Post-MVP)
- Docker, Terraform, Helm, AWS CLI support
- VS Code or JetBrains IDE extensions
- Workflow marketplace and sharing
- Slack/Teams bot integration
- SSO/SAML authentication
- Pattern learning and command prediction
- Runbook automation engine
- Multi-language support (only English for MVP)
- Advanced local model fine-tuning (basic GGUF inference is MVP scope)
- Web dashboard or GUI beyond TUI
