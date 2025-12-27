# Quick Start Guide: Kaido AI Shell Setup

**Feature**: User Setup Guide  
**Date**: 2025-10-22

## Prerequisites

- Rust 1.75+ installed
- Internet connection (for external AI services)
- Terminal/command line access

## Installation

### 1. Clone and Build
```bash
git clone https://github.com/your-org/kaido-ai.git
cd kaido-ai
cargo build --release
```

### 2. Install Binary
```bash
cargo install --path .
```

## Basic Configuration

### 1. Create Configuration File
```bash
kaido --init-config
```

### 2. Configure External AI Service (OpenAI)
Edit `~/.config/kaido/config.toml`:
```toml
[model]
type = "cloud"

[cloud_api]
api_url = "https://api.openai.com/v1/chat/completions"
api_key = "sk-your-openai-api-key"
model_name = "gpt-3.5-turbo"
timeout_seconds = 30
```

### 3. Validate Configuration
```bash
kaido --validate-config
```

## First Use

### 1. Start Kaido Shell
```bash
kaido
```

### 2. Test AI Integration
```
kaido> list files in current directory
 I understand you want to: list files in current directory
Generated command: ls -la
Executing: ls -la
[file listing output]
```

### 3. Configure Safety Settings
Edit configuration to customize safety behavior:
```toml
[safety]
require_confirmation_for = ["rm -rf", "sudo", "chmod 777"]
auto_confirm_dangerous = false
log_all_commands = true
```

## Troubleshooting

### Common Issues

**Configuration not found**:
- Run `kaido --init-config` to create default configuration

**API key invalid**:
- Verify API key in OpenAI dashboard
- Check for typos in configuration file
- Ensure API key has sufficient credits

**Compilation errors**:
- Update Rust: `rustup update`
- Check dependencies: `cargo check`

### Getting Help

- Run `kaido --help` for command options
- Check logs in `~/.local/share/kaido/logs/`
- Visit documentation: `docs/` directory

## Next Steps

- Customize AI explanation style in configuration
- Set up command aliases
- Configure logging preferences
- Explore advanced safety features
