# Kaido AI - Usage Guide

## Quick Start

### 1. Build

```bash
cd /Users/gloomcheng/Workspace/RikaiDev/kaido-ai
cargo build --release
```

### 2. Configure API Key (Required)

Kaido AI requires a Gemini API key for agent reasoning:

```bash
export GEMINI_API_KEY="your_gemini_api_key_here"
```

Or edit `~/.config/kaido/config.toml`:

```toml
gemini_api_key = "your_api_key_here"
```

Get your API key: <https://aistudio.google.com/app/apikey>

### 3. Run

```bash
./target/release/kaido
```

Or with cargo:

```bash
cargo run --release
```

## Usage

### Describe Your Problem

Simply describe your ops problem in natural language:

```text
→ nginx won't start, says port 80 already in use
```

```text
→ apache returns 404 for /webhook endpoint, backend running on 8080
```

```text
→ docker-compose services cannot connect to each other
```

### Agent Autonomous Diagnosis

The agent will autonomously execute diagnostic steps using the ReAct pattern:

**Thought** → Decides what to investigate  
**Action** → Executes diagnostic command  
**Observation** → Analyzes the output  
**Reflection** → Evaluates progress and determines next step  
**Solution** → Provides root cause and remediation plan

### Example Output

```text
╭─ THOUGHT #1
│ Need to identify what process is using port 80
╰─────────────────────────────────────────

╭─ ACTION #1
│ [network] lsof -i :80 -P -n
╰─────────────────────────────────────────
⟳ executing...

╭─ OBSERVATION #1
│ apache2  1234 root  4u  IPv6  TCP *:80 (LISTEN)
╰─────────────────────────────────────────

╭─ REFLECTION #1
│ Apache2 is occupying port 80, causing the conflict.
│ Should validate nginx configuration before proposing solution.
╰─────────────────────────────────────────

╭─ THOUGHT #2
│ Verify nginx configuration is valid
╰─────────────────────────────────────────

╭─ ACTION #2
│ [nginx] nginx -t
╰─────────────────────────────────────────
⟳ executing... ✓

╭─ OBSERVATION #2
│ nginx: configuration file syntax is ok
│ nginx: configuration file test is successful
╰─────────────────────────────────────────

╭─ SOLUTION
│ ▸ Root Cause: Port conflict between nginx and apache2
│ ▸ Resolution Options:
│   1. Stop apache2: systemctl stop apache2
│   2. Reconfigure nginx to use port 8080
│   3. Set up reverse proxy configuration
╰─────────────────────────────────────────

Session completed in 3.2s (6 steps, 2 actions)
```

## Example Scenarios

### 1. Nginx Port Conflict

```text
→ nginx won't start, says address already in use
```

Agent will:

- Check port 80/443 usage (`lsof`, `netstat`)
- Validate nginx configuration (`nginx -t`)
- Identify conflicting service
- Provide resolution steps

### 2. Apache 404 Issue

```text
→ apache returns 404 for /webhook endpoint
```

Agent will:

- Check VirtualHost configuration (`apache2ctl -S`)
- Verify backend service status
- Analyze ProxyPass setup
- Suggest configuration fixes

### 3. Docker Network Problem

```text
→ docker-compose services cannot connect to each other
```

Agent will:

- Check container status (`docker-compose ps`)
- Inspect network configuration (`docker network inspect`)
- Analyze docker-compose.yml
- Diagnose DNS or network issues

### 4. Kubernetes Pod Issues

```text
→ pod keeps restarting in production namespace
```

Agent will:

- Check pod status (`kubectl get pods`)
- View pod logs (`kubectl logs`)
- Examine resource limits
- Analyze error patterns

## Commands

- `help` - Show help information
- `clear` - Clear screen
- `exit` / `quit` / `q` - Exit agent

## Audit Trail

All agent executions are logged to:

```text
~/.kaido/agent_audit.db
```

Contains:

- Complete diagnostic session records
- Step-by-step execution details
- Root cause and solution data
- Execution duration statistics
- 90-day retention policy

## Testing

Run test suite:

```bash
# Unit and integration tests
cargo test

# Scenario tests (requires actual system tools)
cargo test --test agent_scenarios -- --ignored
```

## Requirements

1. **Gemini API Key** - For agent reasoning
2. **System Permissions** - Some diagnostic commands may require sudo
3. **Network Connection** - API calls require internet
4. **System Tools** - nginx, apache2, docker, etc. should be installed

## Troubleshooting

### API Key Error

```text
[WARN] Gemini API key not found
```

**Solution:** Set `GEMINI_API_KEY` environment variable

### Tool Not Found

Agent will automatically fallback to shell execution

### Audit Database Error

Check `~/.kaido/` directory permissions

## Advanced Usage

### Query Audit History

```bash
sqlite3 ~/.kaido/agent_audit.db \
  "SELECT * FROM agent_sessions ORDER BY start_time DESC LIMIT 10"
```

### Manual Cleanup

Automatic 90-day retention. Manual cleanup:

```bash
sqlite3 ~/.kaido/agent_audit.db \
  "DELETE FROM agent_sessions WHERE start_time < strftime('%s', 'now', '-30 days')"
```

## Performance

- First execution may be slower (model initialization)
- Subsequent executions are faster
- Complex problems may require 10-20 steps
- Average diagnosis time: 2-5 seconds

## Supported Tools

| Tool | Capabilities |
|------|-------------|
| **nginx** | Config validation, status check, port diagnosis, log analysis |
| **apache2** | Config test, module check, VirtualHost listing |
| **docker** | Container management, compose analysis, network diagnosis |
| **network** | Port scanning, firewall check, connection testing |
| **kubectl** | Pod management, log queries, resource inspection |
| **sql** | Database queries, connection diagnosis |

## Architecture

Kaido AI uses a **ReAct (Reasoning + Acting)** pattern:

```text
Observation → Thought → Action → Reflection → (repeat until solved)
```

The agent:

1. Maintains state across multiple reasoning steps
2. Executes tools through a unified registry
3. Self-reflects on progress after each action
4. Terminates when solution is found or max iterations reached

## Configuration

Default config location: `~/.config/kaido/config.toml`

```toml
# Gemini API configuration
gemini_api_key = "your_key"
gemini_model = "gemini-1.5-flash-latest"

# Agent behavior
max_iterations = 20
max_execution_time_seconds = 300

# Audit settings
audit_enabled = true
audit_retention_days = 90
```

## Contributing

See `CONTRIBUTING.md` for guidelines.

## License

MIT License - Copyright (c) 2025 RikaiDev

---

**Ready to diagnose? Start Kaido AI!** 🚀
