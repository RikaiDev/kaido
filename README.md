# Kaido AI - Autonomous Ops AI Agent

Self-driving AI agent that diagnoses, troubleshoots, and resolves infrastructure problems through multi-step reasoning and tool execution.

## Features

### Autonomous Agent Capabilities

- **ReAct Loop**: Observation → Thought → Action → Reflection cycle
- **Problem Diagnosis**: Systematic root cause analysis
- **Multi-step Planning**: Breaks down complex problems into executable steps
- **Self-healing**: Automatically identifies and resolves issues

### Universal Tool Support

- kubectl (Kubernetes)
- docker & docker-compose (Containers)
- nginx & apache2 (Web servers)
- netstat, iptables, ufw (Network diagnostics)
- SQL (MySQL/PostgreSQL)
- drush (Drupal)

### Dual AI Engine

- Local GGUF models (privacy-first)
- Gemini 2.0 Flash Exp (cloud fallback)

### Safety Features

- Risk classification (Low/Medium/High/Critical)
- Confirmation prompts for destructive commands
- Full audit logging to SQLite

### Intelligent Problem Solving

- **Pattern Matching**: Recognizes common error signatures
- **AI Diagnosis**: Multi-source evidence analysis
- **Root Cause Analysis**: Identifies true causes, not just symptoms
- **Solution Generation**: Proposes multiple remediation options with risk levels

### Real-world Scenarios

- Port conflicts (nginx vs apache2)
- Docker networking issues
- Service failures and configuration errors
- Resource exhaustion
- Permission problems

## Quick Start

### 1. Installation

```bash
cargo build --release
sudo cp target/release/kaido /usr/local/bin/
```

### 2. Initialize Configuration

```bash
kaido init
```

This will:

- Set up your Gemini API key
- Configure safety settings
- Create audit database
- Save config to `~/.config/kaido/config.toml`

### 3. Run Kaido AI

```bash
kaido
```

## Configuration

### Gemini API Key

Get your API key from: <https://aistudio.google.com/app/apikey>

### Option 1: Environment Variable (Recommended)

```bash
export GEMINI_API_KEY="your_api_key_here"
```

### Option 2: Config File

Run `kaido init` or manually edit `~/.config/kaido/config.toml`:

```toml
[ai]
api_key = ""
model = "gpt-4-turbo-preview"
base_url = "https://api.openai.com/v1"
timeout_seconds = 10

[audit]
database_path = "~/.kaido/audit.db"
retention_days = 90

[safety]
confirm_destructive = true
require_typed_confirmation_in_production = true
log_commands = true

[display]
show_confidence_threshold = 70
show_reasoning = false

# Gemini API Key
gemini_api_key = "your_api_key_here"
```

## Usage Examples

### kubectl Operations

```bash
kaido> list all pods in production
[AI] Analyzing command (trying local GGUF first)...
Tool: kubectl
Command: kubectl get pods -n production
Confidence: 95%
Risk Level: LOW
[EXECUTING]
[OK] Success
```

### Docker Management

```bash
kaido> show running containers
Tool: docker
Command: docker ps
```

### SQL Queries

```bash
kaido> show all databases
Tool: mysql
Command: SHOW DATABASES;
```

### Error Handling

```bash
kaido> kubectl get pods -n wrong-namespace
[X] Failed
[?] AI analyzing error...
Error Type: Namespace Not Found
Reason: The specified namespace does not exist

Suggested Solutions:
  1. List available namespaces
     $ kubectl get namespaces
  2. Check your current context
     $ kubectl config current-context
```

## Architecture

```text
User Input (Natural Language)
    ↓
AIManager (Local GGUF + Gemini fallback)
    ↓
CommandEngine (Universal tool processing)
    ↓
Tool Detection (kubectl/docker/sql/drush)
    ↓
Risk Classification (Low/Medium/High/Critical)
    ↓
Execution
    ↓
Error Handling (PatternMatcher + LLM)
    ↓
Audit Logging (SQLite)
```

## Commands

### `kaido`

Start the interactive shell

### `kaido init`

Initialize configuration with interactive wizard

### `kaido init --non-interactive`

Initialize with default values (no prompts)

## Safety System

### Risk Levels

- **Low**: Read-only operations (get, list, show, describe)
- **Medium**: State-modifying operations (create, update, restart)
- **High**: Deletion operations (delete, remove, drop)
- **Critical**: Batch destructive operations (with wildcards or command substitution)

### Confirmations

- Medium/High risk: Simple confirmation
- Critical risk: Typed confirmation required
- Production environments: Extra safeguards

### Audit Log

All commands are logged to `~/.kaido/audit.db`:

- User action (executed/cancelled/edited)
- Command and translation
- Risk level
- Execution result
- Timestamp
- Environment context

## Development

### Build

```bash
cargo build
```

### Test

```bash
cargo test
```

### Run Example

```bash
cargo run --example api_demo
```

## Security Notes

- **Never commit API keys to version control**
- Each user should use their own Gemini API key
- API keys are stored in `~/.config/kaido/config.toml` (600 permissions)
- Environment variables take precedence over config file
- Audit logs contain command history - secure accordingly

## License

MIT

## Credits

Built with:

- [Rust](https://www.rust-lang.org/)
- [Gemini AI](https://ai.google.dev/)
- [ratatui](https://github.com/ratatui-org/ratatui) for TUI
- [rustyline](https://github.com/kkawakam/rustyline) for REPL
