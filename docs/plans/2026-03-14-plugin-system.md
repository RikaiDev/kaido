# Plugin System Design

## Overview

Kaido's plugin system allows extensible AI assistance for DevOps tools.

## Architecture

```
~/.kaido/
├── plugins.toml          # Plugin configuration
├── plugins/              # Custom plugins directory
│   └── my-plugin/
│       └── plugin.toml
└── cache/
    └── plugin-registry.json  # Official plugin list
```

## Plugin Types

### 1. Official Plugins (Bundled)
- nginx, docker, k8s, ssh, systemd
- Enabled by default
- Maintained by Kaido team

### 2. Community Plugins
- Downloaded from plugin registry
- Version managed
- User-installed via command palette

### 3. Custom Plugins
- Local plugins in `~/.kaido/plugins/`
- For personal use or testing

## Plugin Definition

```toml
# plugins/nginx/plugin.toml
name = "nginx"
version = "1.0.0"
description = "Nginx configuration and error assistance"

[hooks]
on_error = ["502", "503", "bind failed"]
on_config = ["nginx.conf", "nginx"]

[config]
file_patterns = ["*.conf"]

[ai]
system_prompt = "You are a Nginx expert..."

[skills]
directory = "./skills"
```

## Hook System

Plugins can hook into:

| Hook | Description |
|------|-------------|
| `on_command` | Before/after command execution |
| `on_error` | When command fails |
| `on_config_edit` | When editing config files |
| `on_startup` | Shell starts |
| `on_natural_language` | User inputs natural language |

## Plugin Registry

```json
{
  "plugins": [
    {
      "name": "nginx",
      "version": "1.0.0",
      "description": "Nginx expert",
      "author": "KaidoTeam",
      "repo": "https://github.com/rikaidev/kaido-plugin-nginx"
    }
  ]
}
```

## Command Palette Integration

```
kaido> /plugins
→ Opens plugin browser

Ctrl+P → "plugin nginx install"
```

## Future: Plugin Directory

- Official: `kaido install nginx`
- Community: User submits PR to `kaido-plugins` repo
- Auto-update: `kaido plugin update`
