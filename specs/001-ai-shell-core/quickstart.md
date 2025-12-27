# Quickstart Guide: Kaido AI Shell Core

**Date**: 2025-10-22  
**Feature**: Kaido AI Shell Core  
**Purpose**: Get users up and running quickly with Kaido AI Shell

## Installation

### Prerequisites

- Rust 1.75+ installed
- 4GB+ RAM available
- 5GB+ disk space for model files

### Quick Install

```bash
# Clone the repository
git clone https://github.com/your-org/kaido-ai.git
cd kaido-ai

# Build the project
cargo build --release

# Download the default model (optional, can be done later)
./target/release/kaido download-model phi3-mini
```

### Binary Distribution (Future)

```bash
# Download and install binary
curl -sSL https://install.kaido.ai | bash

# Or download directly
wget https://releases.kaido.ai/latest/kaido-linux-x64.tar.gz
tar -xzf kaido-linux-x64.tar.gz
sudo mv kaido /usr/local/bin/
```

## First Run

### Start Kaido Shell

```bash
# Start the shell
kaido

# You should see:
# kaido> 
```

### Basic Usage

#### Traditional Commands (P1)

```bash
# Kaido works exactly like bash/zsh
kaido> ls -la
kaido> cd ~/Documents
kaido> pwd
kaido> echo "Hello World"
```

#### Natural Language Commands (P1)

```bash
# Describe what you want to do
kaido> I want to create a new React project with TypeScript

# Kaido will:
# 1. Plan the steps
# 2. Show you the plan
# 3. Ask for confirmation
# 4. Execute the commands
```

#### Error Help (P2)

```bash
# Try a command that might fail
kaido> cd /nonexistent/directory

# Kaido will:
# 1. Detect the error
# 2. Explain what went wrong
# 3. Suggest solutions
# 4. Offer to help fix it
```

## Configuration

### Basic Configuration

Create `~/.config/kaido/config.toml`:

```toml
[model]
name = "phi3-mini"
path = "models/phi3-mini.gguf"
max_tokens = 2048
temperature = 0.7

[safety]
require_confirmation_for = [
    "rm -rf",
    "sudo",
    "chmod 777",
]
auto_confirm_safe_commands = true
log_all_commands = true

[shell]
default_prompt = "kaido> "
history_size = 1000
auto_complete = true
show_execution_time = true
```

### Model Management

```bash
# List available models
kaido list-models

# Download a model
kaido download-model phi3-mini

# Switch models
kaido config model.name = "phi3-mini"

# Check model status
kaido status
```

## Common Use Cases

### Project Setup

```bash
# Create a new project
kaido> I want to create a Python web app with FastAPI

# Kaido will plan and execute:
# 1. Create project directory
# 2. Initialize virtual environment
# 3. Install FastAPI and dependencies
# 4. Create basic project structure
# 5. Set up development environment
```

### File Operations

```bash
# Organize files
kaido> Move all my PDF files to a Documents folder

# Kaido will:
# 1. Find all PDF files
# 2. Create Documents folder if needed
# 3. Move files safely
# 4. Confirm completion
```

### Git Workflows

```bash
# Git operations
kaido> Commit my changes with a descriptive message

# Kaido will:
# 1. Check git status
# 2. Stage appropriate files
# 3. Generate commit message
# 4. Execute commit
```

### Package Management

```bash
# Install packages
kaido> Install the latest version of Node.js and npm

# Kaido will:
# 1. Check current versions
# 2. Download and install Node.js
# 3. Update npm
# 4. Verify installation
```

## Safety Features

### Dangerous Command Protection

```bash
# Try a dangerous command
kaido> rm -rf /tmp/test

# Kaido will:
# 1. Detect the dangerous pattern
# 2. Show warning message
# 3. Ask for confirmation
# 4. Suggest safer alternatives
```

### Command History

```bash
# View command history
kaido> history

# Search history
kaido> history | grep git

# Clear history
kaido> clear-history
```

### Logging

All commands are logged to `~/.local/share/kaido/logs/`:

```bash
# View recent logs
kaido logs

# View specific log file
kaido logs --file execution.log

# Clear logs
kaido logs --clear
```

## Troubleshooting

### Common Issues

#### Model Not Loading

```bash
# Check model status
kaido status

# Re-download model
kaido download-model phi3-mini --force

# Check available disk space
df -h
```

#### Permission Errors

```bash
# Check file permissions
ls -la ~/.config/kaido/

# Fix permissions
chmod 755 ~/.config/kaido/
chmod 644 ~/.config/kaido/config.toml
```

#### Performance Issues

```bash
# Check system resources
kaido status --verbose

# Reduce model size
kaido config model.max_tokens = 1024

# Use smaller model
kaido download-model phi3-mini-4k
```

### Getting Help

```bash
# Show help
kaido --help

# Show specific command help
kaido help <command>

# Show configuration help
kaido config --help

# Show model help
kaido model --help
```

### Debug Mode

```bash
# Run with debug logging
kaido --debug

# Run with verbose output
kaido --verbose

# Check configuration
kaido config --validate
```

## Advanced Usage

### Custom Safety Rules

Add custom safety rules to `config.toml`:

```toml
[safety.rules.custom]
pattern = "docker.*--privileged"
description = "Docker with privileged mode"
severity = "high"
requires_confirmation = true
```

### AI Context Management

```bash
# Clear AI conversation context
kaido clear-context

# Set AI preferences
kaido config ai.explanation_style = "detailed"
kaido config ai.safety_level = "high"
```

### Integration with Existing Shells

```bash
# Use Kaido as default shell
chsh -s $(which kaido)

# Or create alias
echo 'alias k="kaido"' >> ~/.bashrc
```

## Examples

### Complete Workflow Example

```bash
# Start Kaido
kaido

# Traditional command
kaido> ls
total 8
drwxr-xr-x  2 user user 4096 Oct 22 10:00 Documents
drwxr-xr-x  2 user user 4096 Oct 22 10:00 Downloads

# Natural language command
kaido> I want to organize my Downloads folder

# Kaido responds:
# I'll help you organize your Downloads folder. Let me first see what's in there.
# 
# Plan:
# 1. List files in Downloads
# 2. Create organized folders (Images, Documents, Archives, etc.)
# 3. Move files to appropriate folders
# 4. Clean up empty folders
# 
# Execute this plan? (y/n): y
# 
# Executing step 1: ls ~/Downloads
# Found 15 files
# 
# Executing step 2: mkdir -p ~/Downloads/{Images,Documents,Archives,Other}
# Created organization folders
# 
# Executing step 3: Moving files...
# Moved 5 images to Images/
# Moved 3 documents to Documents/
# Moved 2 archives to Archives/
# Moved 5 other files to Other/
# 
# Executing step 4: Cleaning up...
# Organization complete!

# Error handling
kaido> cd /nonexistent
# Error: No such file or directory: /nonexistent
# 
#  Kaido AI: The directory '/nonexistent' doesn't exist. Here are some suggestions:
# 1. Check if you meant a different path
# 2. Create the directory if needed: mkdir -p /nonexistent
# 3. List current directory: ls
# 
# Would you like me to help you with any of these options?

# Safety confirmation
kaido> rm -rf /tmp/test
# ️  Safety Warning: This command will permanently delete files
# Command: rm -rf /tmp/test
# Severity: High
# 
# This is a destructive operation. Are you sure? (y/n): n
# Command cancelled for safety.
```

## Next Steps

1. **Explore Commands**: Try different natural language inputs
2. **Customize Configuration**: Adjust settings in `config.toml`
3. **Add Safety Rules**: Define custom safety patterns
4. **Integrate Workflows**: Use Kaido in your daily development
5. **Provide Feedback**: Report issues and suggest improvements

## Support

- **Documentation**: https://docs.kaido.ai
- **Issues**: https://github.com/your-org/kaido-ai/issues
- **Discussions**: https://github.com/your-org/kaido-ai/discussions
- **Discord**: https://discord.gg/kaido-ai
