# Phase 1 Implementation: --json + --target

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add `--json` and `--target` flags for AI agent integration and remote host support.

**Architecture:** Add flags to CLI, pass to REPL, output JSON when flag set.

**Tech Stack:** Rust (clap, serde_json)

---

## Task 1: Add --json Flag

**Files:**
- Modify: `src/bin/kaido.rs:17-24`

**Step 1: Add json flag to Cli struct**

```rust
#[derive(Parser)]
#[command(name = "kaido")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Kaido AI - Your AI Ops Coach", long_about = None)]
struct Cli {
    /// Output as JSON (for AI agent integration)
    #[arg(long, short)]
    json: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}
```

**Step 2: Pass to REPL**

```rust
None => {
    let mut repl = KaidoREPL::new()?;
    repl.set_json_mode(cli.json);
    repl.run().await?;
}
```

**Step 3: Commit**

```bash
git add src/bin/kaido.rs
git commit -m "feat: add --json flag for AI agent integration"
```

---

## Task 2: Add json_mode to REPL

**Files:**
- Modify: `src/shell/repl.rs:14-19`

**Step 1: Add field to KaidoREPL**

```rust
pub struct KaidoREPL {
    ai_manager: AIManager,
    tool_context: ToolContext,
    audit_logger: Option<AgentAuditLogger>,
    config: Config,
    json_mode: bool,  // ADD THIS
}
```

**Step 2: Initialize in new()**

```rust
impl KaidoREPL {
    pub fn new() -> Result<Self> {
        // ... existing code ...
        Ok(Self {
            // ... existing fields ...
            json_mode: false,
        })
    }
}
```

**Step 3: Add setter**

```rust
pub fn set_json_mode(&mut self, enabled: bool) {
    self.json_mode = enabled;
}
```

**Step 4: Commit**

```bash
git add src/shell/repl.rs
git commit -m "feat: add json_mode field to REPL"
```

---

## Task 3: Output JSON in REPL

**Files:**
- Modify: `src/shell/repl.rs` (find where agent completes)

**Step 1: Find output point**

Look for where `agent.run()` returns in `run()` method around line 140-160.

**Step 2: Add JSON branch**

After agent.run() completes, add:

```rust
if self.json_mode {
    let result = serde_json::json!({
        \"task\": state.task,
        \"status\": format!(\"{:?}\", state.status),
        \"thinking\": state.history.iter().map(|s| serde_json::json!({
            \"step\": s.step_number,
            \"type\": format!(\"{:?}\", s.step_type),
            \"content\": s.content,
        })).collect::<Vec<_>>(),
        \"action\": state.history.last().map(|s| s.content.clone()),
        \"root_cause\": state.root_cause,
        \"solution\": state.solution_plan,
    });
    println!(\"{}\", serde_json::to_string(&result)?);
    return Ok(());
}
```

**Step 3: Commit**

```bash
git add src/shell/repl.rs
git commit -m "feat: output JSON when --json flag set"
```

---

## Task 4: Add --target Flag

**Files:**
- Modify: `src/bin/kaido.rs:17-24`

**Step 1: Add Target enum**

```rust
/// Target host for operation
#[derive(Debug, Clone, Default)]
pub enum Target {
    #[default]
    Local,
    Remote {
        host: String,
        user: Option<String>,
    },
}
```

**Step 2: Add target flag to Cli**

```rust
struct Cli {
    /// Output as JSON
    #[arg(long, short)]
    json: bool,
    
    /// Target host (user@host for remote, empty for local)
    #[arg(long, value_name = "user@host")]
    target: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}
```

**Step 3: Parse target**

```rust
fn parse_target(s: &str) -> Target {
    if s.contains('@') {
        let parts: Vec<&str> = s.split('@').collect();
        Target::Remote {
            user: Some(parts[0].to_string()),
            host: parts[1].to_string(),
        }
    } else {
        Target::Local
    }
}
```

**Step 4: Pass to REPL**

```rust
None => {
    let mut repl = KaidoREPL::new()?;
    repl.set_json_mode(cli.json);
    repl.set_target(parse_target(cli.target.as_deref().unwrap_or(\"\")));
    repl.run().await?;
}
```

**Step 5: Commit**

```bash
git add src/bin/kaido.rs
git commit -m "feat: add --target flag for remote hosts"
```

---

## Task 5: Add target to REPL

**Files:**
- Modify: `src/shell/repl.rs`

**Step 1: Add Target enum to repl.rs**

```rust
use crate::bin::kaido::Target;
```

**Step 2: Add field**

```rust
pub struct KaidoREPL {
    // ... existing fields ...
    target: Target,
}
```

**Step 3: Initialize**

```rust
target: Target::Local,
```

**Step 4: Add setter**

```rust
pub fn set_target(&mut self, target: Target) {
    self.target = target;
}
```

**Step 5: Commit**

```bash
git add src/shell/repl.rs
git commit -m "feat: add target field to REPL"
```

---

## Task 6: Test

```bash
# Build
cargo build --release

# Test JSON output
echo \"nginx port 80\" | cargo run -- --json 2>/dev/null

# Verify JSON is valid
echo \"nginx port 80\" | cargo run -- --json 2>/dev/null | jq '.'

# Test target flag (basic parse)
cargo run -- --target user@host 2>&1 | head -5
```

---

## Summary

| Task | Description |
|------|-------------|
| 1 | Add --json flag to CLI |
| 2 | Add json_mode to REPL |
| 3 | Output JSON when flag set |
| 4 | Add --target flag |
| 5 | Add target to REPL |
| 6 | Test |
