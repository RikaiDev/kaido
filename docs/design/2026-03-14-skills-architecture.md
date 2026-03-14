# Kaido Skills Architecture: Knowledge-as-Data Design

> **Goal:** Make Kaido thin, smart, extensible, and evolvable through data-driven skills.

---

## Problem Statement

Current Kaido has:
- ✅ ReAct reasoning loop
- ✅ Hardcoded tools (docker, kubectl, nginx...)
- ✅ MCP integration
- ✅ Basic context collection (local only)

Missing:
- ❌ SSH remote host support
- ❌ Structured knowledge base (skills)
- ❌ Wide domain coverage (security, network, database...)
- ❌ Extensible without code changes

---

## Core Philosophy: Simplicity is the Ultimate Sophistication

```
┌─────────────────────────────────────────────────────┐
│                    Kaido Core                       │
│  (CLI, JSON output, ReAct loop, Tool execution)    │
└─────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────┐
│              Skills Layer (YAML/JSON)              │
│  - Domain knowledge                                │
│  - Error patterns                                  │
│  - Diagnosis flows                                 │
│  - Solutions                                       │
└─────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────┐
│            Context Collectors                       │
│  - Local collector                                 │
│  - SSH collector                                   │
└─────────────────────────────────────────────────────┘
```

**Principle:** Core = thin, Skills = smart, Data = extensible

---

## Architecture

### 1. Skill System (Knowledge-as-Data)

```
skills/
├── _meta/
│   └── skill.schema.yaml        # JSON Schema for validation
├── kubernetes/
│   ├── skill.yaml               # Domain definition
│   ├── patterns/                # Error patterns
│   │   ├── pod-crash.yaml
│   │   └── image-pull.yaml
│   └── solutions/               # Solution templates
│       └── restart-pod.yaml
├── network/
│   ├── skill.yaml
│   └── patterns/
│       ├── port-conflict.yaml
│       └── dns-failure.yaml
├── security/
│   ├── skill.yaml
│   └── patterns/
│       ├── ssh-brute.yaml
│       └── cert-expired.yaml
└── database/
    ├── skill.yaml
    └── patterns/
        ├── connection-pool.yaml
        └── deadlock.yaml
```

#### Skill YAML Structure

```yaml
# skills/kubernetes/skill.yaml
name: kubernetes
domain: infrastructure
description: "Kubernetes cluster operations and troubleshooting"

triggers:
  - "pod"
  - "deployment"
  - "kubernetes"
  - "k8s"

diagnosis_flow:
  - name: check_pod_status
    command: "kubectl get pod -n {namespace} {pod_name} -o wide"
    parse:
      status: "STATUS"
      restarts: "RESTARTS"
      
  - name: get_events
    command: "kubectl get events -n {namespace} --field-selector involvedObject.name={pod_name} --sort-by='.lastTimestamp'"
    parse:
      reason: "reason"
      message: "message"

patterns:
  - name: CrashLoopBackOff
    triggers:
      - "CrashLoopBackOff"
    root_cause: "Application exits immediately after starting"
    solution:
      - "Check logs: kubectl logs {pod_name} -n {namespace} --previous"
      - "Verify Dockerfile CMD/ENTRYPOINT"
      - "Check application startup time"
      
  - name: ImagePullBackOff
    triggers:
      - "ImagePullBackOff"
    root_cause: "Cannot pull container image"
    solution:
      - "Verify image name and tag exist"
      - "Check image pull secrets"
      - "Verify registry access"
```

### 2. Context Collector System

```rust
// src/context/mod.rs

/// Target host for context collection
#[derive(Debug, Clone)]
pub enum HostTarget {
    /// Local machine
    Local,
    /// Remote host via SSH
    Remote {
        host: String,
        user: Option<String>,
        port: u16,
    },
}

/// Collected context from a host
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostContext {
    pub target: HostTarget,
    pub hostname: String,
    pub os: String,
    pub kernel: String,
    pub memory_total: u64,
    pub disk_usage: Vec<DiskInfo>,
    pub running_services: Vec<String>,
    pub network_connections: Vec<NetworkConnection>,
    pub logs: LogSummary,
}

/// Trait for context collectors
#[async_trait]
pub trait ContextCollector: Send + Sync {
    async fn collect(&self, target: &HostTarget) -> Result<HostContext>;
}

// Collectors
pub struct LocalCollector;
pub struct SSHCollector;

// Usage in Kaido
let context = match target {
    HostTarget::Local => LocalCollector.collect(&HostTarget::Local).await?,
    HostTarget::Remote { .. } => SSHCollector.collect(target).await?,
};
```

#### Context Collection Flow

```
User: "nginx 502 on production server"

Kaido:
  1. Parse target → SSH:user@production.server
  2. SSHCollector.collect() 
     ├─ SSH exec: uname -a
     ├─ SSH exec: df -h
     ├─ SSH exec: systemctl status nginx
     ├─ SSH exec: tail -100 /var/log/nginx/error.log
     └─ SSH exec: netstat -tlnp | grep :80
  3. Inject context into ReAct prompt
  4. Run diagnosis with rich context
```

### 3. Domain Knowledge Integration

```rust
// src/skills/mod.rs

/// Loaded skill from YAML
pub struct Skill {
    pub name: String,
    pub domain: String,
    pub triggers: Vec<String>,
    pub patterns: Vec<ErrorPattern>,
    pub diagnosis_flow: Vec<DiagnosisStep>,
}

/// Pattern matcher
impl Skill {
    /// Find matching pattern for error/output
    pub fn match_pattern(&self, input: &str) -> Option<&ErrorPattern> {
        self.patterns.iter().find(|p| {
            p.triggers.iter().any(|t| input.contains(t))
        })
    }
    
    /// Generate diagnosis commands
    pub fn generate_diagnosis(&self, pattern: &ErrorPattern) -> Vec<DiagnosisStep> {
        // Use skill's diagnosis_flow as template
        // Fill in {variables} from context
        self.diagnosis_flow.clone()
    }
}
```

---

## CLI Usage

```bash
# Local diagnosis
kaido "nginx returns 502"

# Remote diagnosis via SSH
kaido "postgres connection refused" --target user@prod-db.example.com

# With JSON output for AI agents
kaido "pod crash" --target prod-k8s --json | jq '.solution'

# List available skills
kaido skills

# Show skill details
kaido skills show kubernetes
```

---

## Implementation Phases

### Phase 1: Core Infrastructure (Week 1-2)

- [ ] Define Skill YAML schema
- [ ] Create skill loader
- [ ] Build context collector trait + local impl
- [ ] Add `--target` flag for SSH

### Phase 2: Skills Foundation (Week 3-4)

- [ ] Convert existing tool knowledge to skills
- [ ] Load skills from `~/.kaido/skills/`
- [ ] Pattern matching engine
- [ ] Integration with ReAct loop

### Phase 3: Domain Expansion (Week 5+)

- [ ] Add security skill (OWASP patterns)
- [ ] Add network skill
- [ ] Add database skill
- [ ] Community contribution guide

---

## Benefits

| Aspect | Before | After |
|--------|--------|-------|
| **Thin** | Hardcoded logic | Core + data |
| **Smart** | Tool-specific | Domain-aware |
| **Extensible** | Code change | Add YAML |
| **Evolvable** | Release cycles | Skill updates |
| **Remote** | Local only | SSH support |

---

## Why This Approach?

1. **No code changes for new domains** - Add YAML, not Rust
2. **Community can contribute** - Skills are data, not code
3. **Testable** - YAML validation, not integration tests
4. **Versionable** - Skills in separate repo
5. **Flexible** - Override with custom skills

---

## Related

- Extends: #30 (JSON output for AI agents)
- Depends on: MCP context tools
- Future: Community skill marketplace
