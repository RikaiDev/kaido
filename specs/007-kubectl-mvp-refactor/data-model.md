# Data Model

**Feature**: Kubectl-Only MVP (60-Day Reality Check)
**Branch**: `007-kubectl-mvp-refactor`
**Date**: 2025-10-25

## Overview

This document defines the core data structures and their relationships for the kubectl natural language interface with risk-based safety controls and audit logging.

---

## Core Entities

### 1. KubectlContext

Represents the active kubectl context from `~/.kube/config`.

**Attributes**:
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `name` | `String` | Required, non-empty | Context name from kubeconfig (e.g., "prod-cluster") |
| `cluster` | `String` | Required, non-empty | Cluster name from kubeconfig |
| `namespace` | `Option<String>` | Optional | Default namespace for commands (from context) |
| `user` | `String` | Required, non-empty | Username from kubeconfig |
| `environment_type` | `EnvironmentType` | Required | Detected environment: Development/Staging/Production/Unknown |

**Validation Rules**:
- `name` must match a context defined in kubeconfig
- `environment_type` determined by regex pattern on `name`:
  - `prod|production` → Production
  - `stag|staging` → Staging
  - `dev|development` → Development
  - Otherwise → Unknown (treated as Staging for risk calculation)

**Relationships**:
- One KubectlContext is active per REPL session
- Referenced by TranslationRequest for context-aware prompts
- Referenced by AuditLogEntry to record execution environment

**Source**: Parsed from `~/.kube/config` YAML file

---

### 2. TranslationRequest

Input to the AI translation service (OpenAI GPT-4).

**Attributes**:
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `natural_language_input` | `String` | Required, 1-500 chars | User's natural language command |
| `context` | `KubectlContext` | Required | Current kubectl context for prompt construction |
| `api_key` | `String` | Required, non-empty | OpenAI API key from config |

**Validation Rules**:
- `natural_language_input` length: 1-500 characters (prevent API abuse)
- `api_key` format: starts with "sk-" (OpenAI key format)

**Lifecycle**: Created per user command, discarded after translation

---

### 3. TranslationResult

Output from the AI translation service.

**Attributes**:
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `kubectl_command` | `String` | Required, non-empty | Generated kubectl command |
| `confidence_score` | `u8` | Required, 0-100 | AI self-assessed confidence |
| `reasoning` | `String` | Required | Explanation from AI (may include NEEDS_CLARIFICATION) |
| `risk_level` | `RiskLevel` | Derived | Calculated risk: Low/Medium/High |

**Derived Fields**:
- `risk_level`: Calculated by RiskClassifier based on `kubectl_command` and `context.environment_type`
- `requires_confirmation`: `bool` = `risk_level != Low`
- `requires_typed_confirmation`: `bool` = `risk_level == High && context.environment_type == Production`

**Validation Rules**:
- `kubectl_command` must start with "kubectl " (sanitized by AI)
- `confidence_score` range: 0-100
- `reasoning` max length: 1000 characters

**State Transitions**:
```
TranslationResult (pending)
    ↓
[User reviews command]
    ↓
→ EXECUTED (if confirmed/no confirmation needed)
→ EDITED (if user modifies command)
→ CANCELLED (if user rejects)
```

---

### 4. RiskLevel

Enumeration for command risk classification.

**Values**:
| Level | Definition | Confirmation Required | Examples |
|-------|------------|----------------------|----------|
| `Low` | Read-only operations | None | `get`, `describe`, `logs`, `top` |
| `Medium` | State-modifying operations | Yes/No prompt | `apply`, `scale`, `create`, `patch` |
| `High` | Destructive operations | Typed confirmation | `delete`, `drain`, `scale --replicas=0` |

**Classification Algorithm**:
```rust
fn classify_risk(command: &str, env: EnvironmentType) -> RiskLevel {
    if contains_verb(command, ["delete", "drain"]) 
        || (contains_verb(command, ["scale"]) && command.contains("--replicas=0")) {
        return RiskLevel::High;
    }
    
    if contains_verb(command, ["apply", "create", "patch", "edit", "scale", "rollout"]) {
        return RiskLevel::Medium;
    }
    
    RiskLevel::Low  // Default for read operations
}
```

**Environment Impact**:
- High risk in Production → Requires typed confirmation (resource name or "production")
- High risk in Dev/Staging → Requires yes/no confirmation
- Medium risk → Always yes/no confirmation
- Low risk → No confirmation regardless of environment

---

### 5. AuditLogEntry

Permanent record of command execution stored in SQLite.

**Attributes**:
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | `i64` | Primary key, auto-increment | Unique log entry ID |
| `timestamp` | `i64` | Required, Unix timestamp | Command execution time |
| `user_id` | `String` | Required, non-empty | System username (from `users` crate) |
| `natural_language_input` | `String` | Required | Original user input |
| `kubectl_command` | `String` | Required | Executed kubectl command |
| `confidence_score` | `Option<u8>` | Optional, 0-100 | AI confidence (null if direct kubectl input) |
| `risk_level` | `String` | Required, enum serialized | "LOW", "MEDIUM", or "HIGH" |
| `environment` | `String` | Required | Context name (e.g., "prod-cluster") |
| `cluster` | `String` | Required | Cluster name from kubeconfig |
| `namespace` | `Option<String>` | Optional | Target namespace |
| `exit_code` | `Option<i32>` | Optional | Command exit code (null if cancelled) |
| `stdout` | `Option<String>` | Optional, max 10KB | Truncated command output |
| `stderr` | `Option<String>` | Optional, max 10KB | Truncated error output |
| `user_action` | `String` | Required, enum serialized | "EXECUTED", "CANCELLED", or "EDITED" |
| `execution_duration_ms` | `Option<i64>` | Optional | Command execution time in milliseconds |

**Validation Rules**:
- `timestamp` must be <= current time
- `exit_code` range: -255 to 255 (valid exit codes)
- `stdout`/`stderr` truncated to 10KB (10,240 bytes) to prevent database bloat
- `user_action` must be one of: EXECUTED, CANCELLED, EDITED

**Indexes**:
- Primary key: `id`
- Index on `timestamp` (for date range queries: "today", "last week")
- Index on `environment` (for filtering: "production only")
- Index on `user_action` (for analytics: cancelled command rate)

**Retention Policy**:
- Records older than 90 days are deleted on application startup
- Configurable via `audit_retention_days` in config.toml

**Query Patterns**:
```sql
-- "show history today"
SELECT * FROM audit_log 
WHERE timestamp >= strftime('%s', 'now', 'start of day')
ORDER BY timestamp DESC;

-- "show history last week"
SELECT * FROM audit_log 
WHERE timestamp >= strftime('%s', 'now', '-7 days')
ORDER BY timestamp DESC;

-- "show history production"
SELECT * FROM audit_log 
WHERE environment LIKE '%prod%'
ORDER BY timestamp DESC;
```

---

### 6. ConfirmationModal

Ephemeral UI state for risk-based confirmation dialogs.

**Attributes**:
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `translation_result` | `TranslationResult` | Required | Command awaiting confirmation |
| `confirmation_type` | `ConfirmationType` | Derived | None/YesNo/Typed |
| `expected_text` | `Option<String>` | Required if Typed | Text user must type to confirm |
| `low_confidence_warning` | `bool` | Derived | Show warning if confidence < 70% |

**Derived Fields**:
- `confirmation_type`:
  - `None` if `risk_level == Low`
  - `YesNo` if `risk_level == Medium` or `(risk_level == High && env != Production)`
  - `Typed` if `risk_level == High && env == Production`
- `expected_text`:
  - For Typed confirmation: Extract resource name from command or use "production" as fallback
  - Example: "delete deployment nginx" → expected_text = "nginx"
- `low_confidence_warning`: `confidence_score < 70`

**UI Rendering**:
```text
# Low confidence warning (if applicable)
⚠️  Low confidence (62%) - Please review command carefully

# Command display
Translated Command:
  kubectl delete deployment nginx

# Risk warning
⚠️  HIGH RISK: Destructive operation in PRODUCTION
Cluster: prod-cluster
Namespace: default

# Confirmation prompt (Typed)
Type "nginx" to confirm deletion:
[_________________]

[Cancel]  [Confirm]
```

---

### 7. EnvironmentType

Enumeration for kubectl environment classification.

**Values**:
| Type | Detection Pattern | Risk Multiplier | Description |
|------|------------------|-----------------|-------------|
| `Development` | `dev|development` (case-insensitive) | 0.5x | Lowest risk, minimal confirmation |
| `Staging` | `stag|staging` | 1.0x | Medium risk, standard confirmation |
| `Production` | `prod|production` | 2.0x | Highest risk, typed confirmation for HIGH |
| `Unknown` | No pattern match | 1.0x | Treat as Staging by default |

**Purpose**: Influences confirmation requirements (see RiskLevel environment impact)

---

## Entity Relationships

```text
┌─────────────────┐
│ KubectlContext  │  (Parsed from kubeconfig)
│  - name         │
│  - cluster      │
│  - namespace    │
│  - env_type     │
└────────┬────────┘
         │ 1
         │ used by
         │ 1
┌────────▼────────────┐
│ TranslationRequest  │  (User input + context)
│  - nl_input         │
│  - context          │
└────────┬────────────┘
         │ 1
         │ translates to
         │ 1
┌────────▼────────────┐
│ TranslationResult   │  (AI output)
│  - kubectl_command  │
│  - confidence       │
│  - reasoning        │
│  - risk_level       │───┐
└────────┬────────────┘   │
         │ 1               │ evaluated by
         │ displayed in    │ 1
         │ 0..1           ┌▼─────────────┐
┌────────▼────────────┐  │ RiskClassifier│
│ ConfirmationModal   │  │  (Stateless)  │
│  - result           │  └───────────────┘
│  - confirmation_type│
│  - expected_text    │
└────────┬────────────┘
         │ 1
         │ results in
         │ 1
┌────────▼────────────┐
│ AuditLogEntry       │  (Permanent storage)
│  - timestamp        │
│  - nl_input         │
│  - kubectl_command  │
│  - confidence       │
│  - risk_level       │
│  - environment      │
│  - user_action      │
│  - exit_code        │
│  - stdout/stderr    │
└─────────────────────┘
     (SQLite table)
```

---

## Data Flow

### Primary Flow: Natural Language → Kubectl Execution

```text
1. User Input
   ↓
2. Parse KubectlContext (from ~/.kube/config)
   ↓
3. Create TranslationRequest (input + context)
   ↓
4. Call OpenAI API
   ↓
5. Receive TranslationResult (command + confidence + reasoning)
   ↓
6. Classify RiskLevel (based on command verbs)
   ↓
7. Determine Confirmation Type (based on risk + environment)
   ↓
8. Display ConfirmationModal (if needed)
   │
   ├─ User Confirms ─────────┐
   │                         ↓
   │                    Execute kubectl
   │                         ↓
   │                    Capture result
   │                         ↓
   │                    Log to AuditLogEntry
   │                         ↓
   │                    Display output
   │
   ├─ User Cancels ──────────┐
   │                         ↓
   │                    Log to AuditLogEntry (user_action=CANCELLED)
   │                         ↓
   │                    Show "Command cancelled"
   │
   └─ User Edits ────────────┐
                             ↓
                        Allow manual editing
                             ↓
                        Re-classify risk
                             ↓
                        Log to AuditLogEntry (user_action=EDITED)
                             ↓
                        Resume at step 8
```

### Secondary Flow: Audit Log Query

```text
1. User types: "kaido> show history [filter]"
   ↓
2. Parse filter: "today" | "last week" | "production" | none
   ↓
3. Query SQLite with appropriate WHERE clause
   ↓
4. Format results as table in TUI
   ↓
5. Display (paginated if >20 entries)
```

---

## Storage

### SQLite Schema

See `/contracts/audit-log-schema.sql` for complete schema definition.

**Database Location**: `~/.kaido/audit.db`

**Migrations**: None for MVP (fresh schema on initialization)

---

## Validation Summary

| Entity | Key Validations |
|--------|----------------|
| KubectlContext | Name matches kubeconfig contexts, environment pattern matching |
| TranslationRequest | Input length 1-500 chars, API key format "sk-*" |
| TranslationResult | Command starts with "kubectl ", confidence 0-100 |
| RiskLevel | Verb pattern matching logic correct, environment multiplier applied |
| AuditLogEntry | Timestamp <= now, stdout/stderr truncated to 10KB, valid exit code |
| ConfirmationModal | Expected text matches resource name from command |

**Error Handling**:
- Invalid kubeconfig → Error: "kubectl context not configured. Run 'kubectl config get-contexts'"
- OpenAI API failure → Fallback: "Enter kubectl command manually:"
- SQLite write failure → Warning logged, command proceeds (audit is non-blocking)

---

## Open Questions Resolved

✅ All data model decisions resolved in Phase 0 research. No pending clarifications.


