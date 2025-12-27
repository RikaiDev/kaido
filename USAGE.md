# Kaido - Usage Guide

Your AI Ops Coach. Learn infrastructure through hands-on diagnosis.

## Getting Started

### 1. Build

```bash
cargo build --release
```

### 2. Configure

**Option A: Cloud AI (Recommended for beginners)**

```bash
export GEMINI_API_KEY="your_gemini_api_key_here"
```

Get your free API key: <https://aistudio.google.com/app/apikey>

**Option B: Local AI (Privacy-first)**

```bash
kaido init --local-only
```

### 3. Launch

```bash
./target/release/kaido
```

## Your First Session

Just describe what's wrong in plain language:

```
> nginx won't start, says port 80 already in use
```

Kaido will guide you through the diagnosis, explaining each step.

## How Kaido Teaches

### The ReAct Learning Loop

Every diagnosis follows this pattern — watch and learn:

```
THOUGHT     What should I investigate?
    ↓
ACTION      Execute a diagnostic command
    ↓
OBSERVATION What did we find?
    ↓
REFLECTION  What does this mean? What's next?
    ↓
(repeat until solved)
    ↓
SOLUTION    Root cause + how to fix it
```

### Example: Learning to Debug Port Conflicts

```
You: nginx won't start, says address already in use

╭─ THOUGHT #1
│ Need to identify what process is using port 80
╰─────────────────────────────────────────

╭─ ACTION #1
│ [network] lsof -i :80 -P -n
╰─────────────────────────────────────────

┌─ WHAT YOU'RE LEARNING ──────────────────────────────────┐
│                                                          │
│ lsof = "list open files"                                │
│                                                          │
│ In Unix, everything is a file — including network       │
│ connections. So lsof can find which process is using    │
│ a specific port.                                        │
│                                                          │
│ -i :80  → filter by port 80                             │
│ -P      → show port numbers (not service names)         │
│ -n      → skip DNS lookup (faster)                      │
│                                                          │
│ This is a fundamental debugging skill for any SRE.      │
└──────────────────────────────────────────────────────────┘

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

┌─ WHAT YOU'RE LEARNING ──────────────────────────────────┐
│                                                          │
│ nginx -t = "test configuration"                         │
│                                                          │
│ Always validate config before restarting a service.     │
│ A syntax error in nginx.conf will prevent startup.      │
│                                                          │
│ Pro tip: Run this BEFORE every nginx restart.           │
└──────────────────────────────────────────────────────────┘

╭─ OBSERVATION #2
│ nginx: configuration file syntax is ok
│ nginx: configuration file test is successful
╰─────────────────────────────────────────

╭─ SOLUTION
│
│ ROOT CAUSE: Port conflict between nginx and apache2
│
│ OPTIONS:
│   1. Stop apache2: systemctl stop apache2
│   2. Reconfigure nginx to use port 8080
│   3. Set up reverse proxy configuration
│
│ CONCEPT LEARNED: Two services cannot bind to the same port.
│ SKILL ACQUIRED: Using lsof to find port usage.
╰─────────────────────────────────────────

Session completed in 3.2s (6 steps, 2 actions)
```

## Learning Scenarios

Each scenario teaches specific Ops concepts:

### 1. Port Conflicts

```
> nginx won't start, says address already in use
```

**What You'll Learn:**
- How to find processes using specific ports (`lsof`, `netstat`)
- How services bind to network ports
- Configuration validation before restart

**Key Commands:**
```bash
lsof -i :80 -P -n      # Find what's using port 80
nginx -t               # Validate nginx config
systemctl status nginx # Check service status
```

### 2. Reverse Proxy Issues

```
> apache returns 404 for /webhook endpoint, backend on 8080
```

**What You'll Learn:**
- How reverse proxies work
- VirtualHost configuration
- ProxyPass directives

**Key Commands:**
```bash
apache2ctl -S          # List virtual hosts
curl localhost:8080    # Test backend directly
tail -f /var/log/apache2/error.log
```

### 3. Container Networking

```
> docker-compose services cannot connect to each other
```

**What You'll Learn:**
- Docker network isolation
- Service discovery via DNS
- Container-to-container communication

**Key Commands:**
```bash
docker network ls                    # List networks
docker network inspect <network>     # See connected containers
docker-compose exec app ping db      # Test connectivity
```

### 4. Kubernetes Debugging

```
> pod keeps restarting in production namespace
```

**What You'll Learn:**
- Pod lifecycle and restart policies
- Reading container logs
- Resource limits and OOMKilled

**Key Commands:**
```bash
kubectl get pods -n production           # List pods
kubectl describe pod <name>              # Detailed status
kubectl logs <pod> --previous            # Logs from crashed container
```

## Skill Progression

As you use Kaido, you'll naturally progress:

```
Level 1: Observer
├── Watch Kaido diagnose problems
├── Learn command patterns
└── Understand the reasoning process

Level 2: Assistant
├── Predict what command comes next
├── Understand why each step matters
└── Start recognizing common patterns

Level 3: Independent
├── Diagnose similar problems yourself
├── Know which tools to reach for
└── Build your own mental models

Level 4: Expert
├── Handle novel situations
├── Combine multiple diagnostic techniques
└── Teach others what you've learned
```

## Commands

| Command | Description |
|---------|-------------|
| `help` | Show available commands |
| `clear` | Clear screen |
| `history` | View recent sessions |
| `exit` | Exit Kaido |

## Understanding Risk Levels

Kaido classifies every command by risk:

| Level | Description | Example | Confirmation |
|-------|-------------|---------|--------------|
| **Low** | Read-only, safe | `kubectl get pods` | None |
| **Medium** | Modifies state | `systemctl restart nginx` | Simple Y/n |
| **High** | Deletes resources | `kubectl delete pod` | Explicit yes |
| **Critical** | Batch destructive | `kubectl delete pods --all` | Type full command |

This teaches you to think about command impact before execution.

## Session History & Review

Every session is logged for your review:

```bash
# View recent sessions
sqlite3 ~/.kaido/agent_audit.db \
  "SELECT datetime(start_time, 'unixepoch'), problem_description
   FROM agent_sessions ORDER BY start_time DESC LIMIT 10"
```

Use this to:
- Review what you learned
- See patterns in problems you encounter
- Track your progress over time

## Configuration

Config file: `~/.config/kaido/config.toml`

```toml
# AI Provider
gemini_api_key = "your_key"
gemini_model = "gemini-1.5-flash-latest"

# Learning Settings
explain_mode = true              # Show command explanations
show_concepts = true             # Highlight learning points

# Safety
confirm_destructive = true       # Require confirmation for risky commands
max_iterations = 20              # Maximum diagnostic steps

# Audit
audit_enabled = true
audit_retention_days = 90
```

## Supported Tools

| Domain | Tools | Concepts You'll Learn |
|--------|-------|----------------------|
| **Web Servers** | nginx, apache2 | Config syntax, virtual hosts, reverse proxy |
| **Containers** | docker, compose | Images, networking, volumes, orchestration |
| **Kubernetes** | kubectl | Pods, services, deployments, debugging |
| **Network** | lsof, netstat, iptables | Ports, connections, firewalls |
| **Databases** | mysql, psql | Queries, connections, permissions |
| **System** | systemctl, journalctl | Services, logs, boot process |

## Troubleshooting

### "API key not found"

```bash
export GEMINI_API_KEY="your_key_here"
```

### "Permission denied"

Some diagnostic commands need elevated privileges:

```bash
sudo kaido
```

### "Tool not found"

Kaido works best when the tools are installed:

```bash
# Debian/Ubuntu
sudo apt install nginx docker.io

# macOS
brew install nginx docker
```

## Privacy & Data

- All session data stored locally in `~/.kaido/`
- No telemetry or usage tracking
- API calls only send your problem description
- Local LLM option for fully offline use

## Next Steps

1. **Practice**: Try diagnosing real problems on your system
2. **Review**: Look at session history to reinforce learning
3. **Experiment**: Try the commands manually after Kaido shows you
4. **Teach**: Explain what you learned to solidify understanding

---

**Remember**: The goal isn't to depend on Kaido forever. It's to learn enough that you don't need it anymore.

That's when you know you've become an Ops expert.
