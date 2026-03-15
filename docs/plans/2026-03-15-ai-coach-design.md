# Kaido AI Coach - Side Panel Design

## Vision

Real-time AI coaching sidebar that watches user actions (terminal/vim) and provides contextual guidance: Diagnosis → Explanation → Best Practice.

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                        User Interface Layer                        │
├─────────────────────┬─────────────────────┬───────────────────────┤
│  Terminal Split     │     Vim Plugin      │   LSP Protocol       │
│  (kaido side)     │  (vim-kaido)        │  (language server)   │
└────────┬───────────┴──────────┬──────────┴─────────┬───────────┘
         │                      │                     │
         └──────────────────────┼─────────────────────┘
                                ↓
┌──────────────────────────────────────────────────────────────────┐
│                      Event Router                                 │
│  - Terminal: CommandExecuted, ErrorOccurred                      │
│  - Vim: BufRead, TextChanged, CursorHold                         │
│  - LSP: textDocument/diagnostic, textDocument/codeAction         │
└──────────────────────────────────────────────────────────────────┘
                                ↓
┌──────────────────────────────────────────────────────────────────┐
│                   Context Collector                                │
│  - File content + cursor position                               │
│  - Recent command output                                         │
│  - Error messages                                                │
│  - Skill context (YAML knowledge base)                          │
│  - Plugin diagnostics                                           │
└──────────────────────────────────────────────────────────────────┘
                                ↓
┌──────────────────────────────────────────────────────────────────┐
│                   AI Processor (LLM)                             │
│  System: "You are DevOps mentor. Guide don't give answers."      │
│  Input:  context + skill knowledge + diagnostics                │
│  Output: Diagnosis → Explanation → Best Practice                │
└──────────────────────────────────────────────────────────────────┘
                                ↓
┌──────────────────────────────────────────────────────────────────┐
│                   Response Formatter                               │
│  - Terminal: ANSI colored text                                  │
│  - Vim: vim9script / regular vim script                         │
│  - LSP: Diagnostic[] + CodeAction[]                             │
└──────────────────────────────────────────────────────────────────┘
```

## Three Implementations

### A. Terminal Split Mode (Priority: HIGH)

Simple terminal side panel running alongside main shell.

```
┌─────────────────────────────────────┐
│ $ kaido coach --side               │
│                                      │
│ ┌──────────┬────────────────────┐  │
│ │ Main     │  Kaido Side Panel  │  │
│ │ Terminal │                    │  │
│ │          │  [Diagnosis]      │  │
│ │ $ nginx  │  Port 80 used     │  │
│ │ -t       │                    │  │
│ │          │  [Explanation]    │  │
│ │          │  Apache is...      │  │
│ │          │                    │  │
│ │          │  [Best Practice]  │  │
│ │          │  Use 8080 instead │  │
│ └──────────┴────────────────────┘  │
└─────────────────────────────────────┘
```

**Implementation:**
- PTY pair: main terminal + side panel
- TMUX-like layout using ratatui
- Event: capture from main PTY or listen to shell events

### B. Vim Plugin (Priority: MEDIUM)

Vim plugin that communicates with Kaido via stdio.

```
" In vimrc
Plug 'rikaidev/vim-kaido'

" Usage
:Kaido analyze          " Analyze current buffer
:Kaido diag             " Show diagnosis only
:Kaido explain          " Explain selection
:Kaido best             " Show best practices
:Kaido toggle           " Toggle side panel
```

**Protocol (JSON over stdio):**
```json
{"method": "analyze", "params": {"file": "/etc/nginx/nginx.conf", "content": "..."}}
{"method": "response", "result": {"diagnosis": "...", "explanation": "...", "best_practice": "..."}}
```

### C. LSP Server (Priority: MEDIUM)

Kaido as Language Server implementing ms-lsp:

```
# kaido-lsp --port 5169

Capabilities:
- textDocument/publishDiagnostics
- textDocument/codeAction
- textDocument/hover
```

## Trigger Events

### Terminal Triggers
| Event | Context | Output Type |
|-------|---------|-------------|
| Command failed | error msg | Diagnosis |
| Config file opened | file path | Explanation |
| Repeated pattern | history | Best Practice |
| User asks "why?" | question | Explanation |

### Vim Triggers
| Event | Context | Output Type |
|-------|---------|-------------|
| BufRead *.conf | filename | Explanation |
| TextChanged (debounced 2s) | buffer diff | Diagnosis |
| CursorHold on error | error highlight | Diagnosis |
| User command :Kaido | selection | All |

### LSP Triggers
| Method | Context | Output Type |
|--------|---------|-------------|
| textDocument/diagnostic | file parse errors | Diagnostic[] |
| textDocument/codeAction | error at cursor | CodeAction[] |
| textDocument/hover | hover request | Hover (explanation) |

## Data Flow

```
1. Event Listener
   - Terminal: pty hook / shell event
   - Vim: autocmd / stdio
   - LSP: json-rpc

2. Context Builder
   file_content: read from buffer/path
   cursor_pos: line + column
   error_context: parse stderr / vim error
   skill_context: match YAML patterns
   plugin_context: run plugin diagnostics

3. LLM Prompt Builder
   system: "You are DevOps mentor..."
   context: all above
   format: Diagnosis | Explanation | Best Practice

4. Response Parser
   - Extract 3 sections
   - Format for display (ANSI/vim/LSP)
   - Cache recent responses

5. Display
   - Terminal: ratatui panel
   - Vim: popup / preview window
   - LSP: diagnostics + code actions
```

## File Structure

```
src/
├── coach/
│   ├── mod.rs              # Main coordinator
│   ├── event_router.rs     # Route events to handlers
│   ├── context/
│   │   ├── collector.rs    # Gather context from sources
│   │   ├── skills.rs       # Skill knowledge lookup
│   │   └── plugins.rs     # Plugin diagnostics
│   ├── ai/
│   │   ├── processor.rs   # LLM calls
│   │   └── formatter.rs   # Format responses
│   ├── triggers/
│   │   ├── terminal.rs    # Shell event listeners
│   │   ├── vim.rs         # Vim protocol handler
│   │   └── lsp.rs         # LSP server
│   └── ui/
│       ├── panel.rs        # Terminal side panel
│       └── display.rs      # Output formatters
│
├── plugins/
│   ├── nginx.rs           # Nginx diagnostics
│   ├── docker.rs         # Docker diagnostics
│   └── k8s.rs           # Kubernetes diagnostics
│
└── skills/
    └── nginx/
        ├── errors/        # Error pattern YAML
        ├── configs/       # Config best practices
        └── explanation/  # Config meaning YAML
```

## Implementation Priority

1. **Phase 1**: Terminal side panel (simplest, validate AI flow)
2. **Phase 2**: Vim plugin (most useful for config editing)
3. **Phase 3**: LSP server (IDE integration)

## Configuration

```toml
[coach]
enabled = true
model = "qwen2.5:1.5b"  # Auto-detect based on RAM

[coach.triggers]
terminal = ["command_failed", "config_opened", "repeated_pattern"]
vim = ["bufread", "textchanged", "cursorhold"]

[coach.display]
position = "right"  # right, left
width = 40
debounce_ms = 2000
```

## Open Questions

1. **Privacy**: Send file content to LLM? Add `[coach.privacy.local_only = true]`
2. **Offline**: Fallback to pattern matching if LLM unavailable
3. **Performance**: Cache responses? How long?
4. **Vim integration**: Neovim (Lua) vs Vim (Vimscript) support?
