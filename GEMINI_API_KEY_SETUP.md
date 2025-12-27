# Gemini API Key Setup

Kaido AI uses Google's Gemini 2.0 Flash Exp model for AI-powered command translation and error analysis.

## Quick Setup (Recommended)

Run the interactive configuration wizard:

```bash
kaido init
```

This will guide you through:

- Setting up your Gemini API key
- Configuring safety settings
- Setting audit log retention
- Validating your API key

## Manual Setup

### Get Your API Key

1. Visit: <https://aistudio.google.com/app/apikey>
2. Sign in with your Google account
3. Click "Create API Key"
4. Copy your API key

### Configure Kaido AI

You have three options to configure your Gemini API key:

### Option 1: Interactive Wizard (Easiest)

```bash
kaido init
```

Follow the prompts to enter your API key. The wizard will:

- Validate your API key
- Save it to `~/.config/kaido/config.toml`
- Set appropriate file permissions

### Option 2: Environment Variable

```bash
# Add to your shell profile (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish, etc.)
export GEMINI_API_KEY="your_api_key_here"
```

### Option 3: Config File (Manual)

Create or edit `~/.config/kaido/config.toml`:

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

## Verify Setup

Run Kaido AI and check logs to see which method loaded the API key:

```bash
kaido
```

The logs will show:

```text
[OK] Gemini API key loaded from environment variable
```

or

```text
[OK] Gemini API key loaded from config file
```

## Troubleshooting

If you see:

```text
[!] Gemini API key not found. Please set GEMINI_API_KEY environment variable or configure in ~/.config/kaido/config.toml
```

Then:

1. Check that your environment variable is set: `echo $GEMINI_API_KEY`
2. Check that your config file exists: `cat ~/.config/kaido/config.toml`
3. Make sure there are no extra spaces or quotes around your API key
4. Restart your terminal after setting the environment variable

## Security Notes

- **Never commit your API key to version control**
- Keep your API key secure and private
- Don't share your API key with others
- Each user should use their own API key
- You can regenerate your API key at any time from Google AI Studio

## API Key Priority

Kaido AI checks for API keys in this order:

1. `GEMINI_API_KEY` environment variable (highest priority)
2. `gemini_api_key` in `~/.config/kaido/config.toml`
3. If none found, returns helpful error message
