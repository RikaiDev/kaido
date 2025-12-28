# Kaido - Your AI Ops Coach

Learn infrastructure operations through guided AI diagnosis. Not just answers — understanding.

```
You:   "nginx won't start, says port 80 already in use"

Kaido: Let me help you understand what's happening...

       THOUGHT: Need to identify what process is using port 80
       ACTION:  lsof -i :80 -P -n

       Here's what I found and why it matters:
       ┌─────────────────────────────────────────────────────────┐
       │ apache2 is running on port 80                          │
       │                                                         │
       │ lsof  → "list open files" (in Unix, network = file)    │
       │ -i :80 → filter by port 80                             │
       │ -P -n  → show port numbers, skip DNS lookup            │
       └─────────────────────────────────────────────────────────┘

       SOLUTION: Stop apache2 first, then start nginx

       Now you know: Two web servers can't share the same port.
       Next time, you can diagnose this yourself!
```

## Why Kaido?

**The Problem**: Ops expertise takes years to build. AI tools that "just solve problems" don't help you grow.

**Our Approach**: Kaido shows its reasoning, explains every command, and teaches you *why* — so you become the expert.

| Traditional AI | Kaido |
|----------------|-------|
| Gives you the answer | Teaches you the concept |
| Black box | Transparent reasoning |
| You stay dependent | You become independent |

## Who Is This For?

- **CS Students**: First time using a terminal? Start here.
- **Career Changers**: Transitioning to DevOps? Learn the fundamentals.
- **Frontend Developers**: Want to understand the backend? We'll guide you.
- **Junior SREs**: Building real-world debugging skills.

## Quick Start

### Installation

**macOS (Apple Silicon)**
```bash
curl -L https://github.com/RikaiDev/kaido/releases/latest/download/kaido-macos-arm64.tar.gz | tar xz
sudo mv kaido kaido-mcp /usr/local/bin/
```

**macOS (Intel)**
```bash
curl -L https://github.com/RikaiDev/kaido/releases/latest/download/kaido-macos-x64.tar.gz | tar xz
sudo mv kaido kaido-mcp /usr/local/bin/
```

**Linux (x64)**
```bash
curl -L https://github.com/RikaiDev/kaido/releases/latest/download/kaido-linux-x64.tar.gz | tar xz
sudo mv kaido kaido-mcp /usr/local/bin/
```

**Linux (ARM64)**
```bash
curl -L https://github.com/RikaiDev/kaido/releases/latest/download/kaido-linux-arm64.tar.gz | tar xz
sudo mv kaido kaido-mcp /usr/local/bin/
```

**From Source**
```bash
cargo install --git https://github.com/RikaiDev/kaido
```

**Initialize**
```bash
kaido init
```

### Your First Session

```bash
kaido
```

Then just describe your problem in plain language:

```
> nginx returns 404 for /api endpoint
> docker containers can't talk to each other
> pod keeps crashing in kubernetes
```

Kaido will diagnose step-by-step, explaining each action along the way.

## Features

### Learn-As-You-Go

Every command comes with context:

```
ACTION: kubectl get pods -n production

┌─ What This Means ───────────────────────────────────────┐
│ kubectl    → Kubernetes command-line tool              │
│ get pods   → List running containers                   │
│ -n         → Namespace flag (like a folder)            │
│ production → The environment we're checking            │
└─────────────────────────────────────────────────────────┘
```

### Transparent Reasoning (ReAct Pattern)

See how an expert thinks:

```
THOUGHT   → "Need to check if the service is running"
ACTION    → systemctl status nginx
OBSERVE   → "Service is failed, exit code 1"
REFLECT   → "Config error likely, should validate"
THOUGHT   → "Let me check the configuration"
ACTION    → nginx -t
SOLUTION  → Found syntax error on line 42
```

### Safe Learning Environment

- **Risk Classification**: Commands labeled Low/Medium/High/Critical
- **Confirmation Prompts**: Dangerous commands require explicit approval
- **Audit Trail**: Every session logged for review

### Privacy-First

- **Ollama integration** for local LLM inference
- Supports llama3.2, mistral, qwen2.5, and more
- All data stays on your machine
- Cloud AI optional (Gemini API)

## Supported Tools

| Domain | Tools |
|--------|-------|
| **Containers** | docker, docker-compose, kubectl |
| **Web Servers** | nginx, apache2 |
| **Network** | lsof, netstat, iptables, ufw |
| **Databases** | MySQL, PostgreSQL |
| **CMS** | drush (Drupal) |

## Configuration

Run the interactive setup wizard:

```bash
kaido init
```

This teaches you about Cloud vs Local AI while configuring:

**Option 1: Gemini API (Cloud)**
```bash
# Fast, powerful, requires internet
export GEMINI_API_KEY="your_key_here"
```

**Option 2: Ollama (Local)**
```bash
# Private, offline-capable
brew install ollama        # or curl -fsSL https://ollama.ai/install.sh | sh
ollama serve               # start the server
ollama pull llama3.2       # download a model
```

**Option 3: Both (Recommended)**
- Gemini for speed, Ollama as private fallback
- `kaido init` auto-detects and configures both

Config file: `~/.kaido/config.toml`

## MCP Server

Kaido exposes 6 tools via [Model Context Protocol](https://modelcontextprotocol.io/) for Claude Code integration:

```json
{
  "mcpServers": {
    "kaido": {
      "command": "kaido-mcp",
      "args": []
    }
  }
}
```

**Available Tools:**
- `kaido_diagnose` - Diagnose ops issues with AI reasoning
- `kaido_execute` - Execute commands with risk assessment
- `kaido_explain` - Get educational explanations for commands
- `kaido_get_context` - Get system environment info
- `kaido_list_tools` - List supported ops tools
- `kaido_check_risk` - Check command risk level

## Part of RikaiDev

Kaido is one of five tools in the [RikaiDev](https://github.com/RikaiDev) ecosystem:

| Tool | Purpose |
|------|---------|
| **Kaido** | Learn Ops through guided diagnosis |
| **Cortex** | AI memory for coding assistants |
| **inboxd** | Unified inbox with AI processing |
| **Toki** | Automatic time tracking |
| **Mimamori** | Workplace communication guardian |

All tools share a philosophy: **AI as coach, not replacement.**

## Development

```bash
# Build
cargo build

# Test
cargo test

# Run
cargo run
```

## Roadmap

See our [GitHub Issues](https://github.com/RikaiDev/kaido/issues) for planned features:

- [x] ReAct reasoning loop
- [x] Multi-tool support
- [x] Risk classification
- [x] Explain mode
- [x] Ollama local LLM
- [x] MCP server integration (6 tools for Claude Code)
- [x] Learning tracker with session summaries
- [x] Adaptive mentoring based on skill level
- [x] AI-native shell with LLM-powered explanations

## License

MIT

---

**Kaido** (海道) — The sea route. Your path to mastering infrastructure.
