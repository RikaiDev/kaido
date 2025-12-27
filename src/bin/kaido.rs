use clap::{Parser, Subcommand};
use kaido::shell::repl::KaidoREPL;
use kaido::shell::KaidoShell;
use kaido::config::{Config, AIProvider};
use kaido::ai::{GeminiBackend, OllamaBackend};
use kaido::tools::LLMBackend;
use std::io::{self, Write};

// ANSI color codes
const CYAN: &str = "\x1b[38;5;147m";
const GREEN: &str = "\x1b[38;5;150m";
const YELLOW: &str = "\x1b[38;5;221m";
const DIM: &str = "\x1b[38;5;245m";
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";

#[derive(Parser)]
#[command(name = "kaido")]
#[command(about = "Kaido AI - Your AI Ops Coach", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start your learning journey - set up AI backends
    Init {
        /// Skip interactive prompts and use defaults
        #[arg(long)]
        non_interactive: bool,
    },
    /// Start the mentor shell (new shell wrapper mode)
    Shell,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if present (for API keys)
    let _ = dotenvy::dotenv();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .format_timestamp(None)
        .format_target(false)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init { non_interactive }) => {
            run_init_learning(non_interactive).await?;
        }
        Some(Commands::Shell) => {
            let mut shell = KaidoShell::new()?;
            shell.run().await?;
        }
        None => {
            let mut repl = KaidoREPL::new()?;
            repl.run().await?;
        }
    }

    Ok(())
}

/// Learning-focused init experience
async fn run_init_learning(non_interactive: bool) -> anyhow::Result<()> {
    // Load existing config or create default
    let mut config = Config::load().unwrap_or_default();

    if non_interactive {
        return run_init_non_interactive(&mut config).await;
    }

    // ══════════════════════════════════════════════════════════════
    // WELCOME SCREEN (width: 59 chars inside box)
    // ══════════════════════════════════════════════════════════════
    println!("\n{CYAN}╭───────────────────────────────────────────────────────────╮{RESET}");
    println!("{CYAN}│{RESET}                                                           {CYAN}│{RESET}");
    println!("{CYAN}│{RESET}   {BOLD}KAIDO SETUP{RESET} - Your First Ops Lesson                     {CYAN}│{RESET}");
    println!("{CYAN}│{RESET}                                                           {CYAN}│{RESET}");
    println!("{CYAN}│{RESET}   Kaido needs an AI \"brain\" to understand your requests   {CYAN}│{RESET}");
    println!("{CYAN}│{RESET}   and translate them into shell commands.                 {CYAN}│{RESET}");
    println!("{CYAN}│{RESET}                                                           {CYAN}│{RESET}");
    println!("{CYAN}│{RESET}   You have two options, each teaches different concepts:  {CYAN}│{RESET}");
    println!("{CYAN}│{RESET}                                                           {CYAN}│{RESET}");
    println!("{CYAN}╰───────────────────────────────────────────────────────────╯{RESET}\n");

    // ══════════════════════════════════════════════════════════════
    // OPTION COMPARISON (width: 59 chars inside box)
    // ══════════════════════════════════════════════════════════════
    println!("{GREEN}┌─ OPTION 1: Gemini API (Cloud) ───────────────────────────┐{RESET}");
    println!("{GREEN}│{RESET}                                                           {GREEN}│{RESET}");
    println!("{GREEN}│{RESET}  {BOLD}What it is:{RESET}  Google's AI, runs on their servers        {GREEN}│{RESET}");
    println!("{GREEN}│{RESET}  {BOLD}Speed:{RESET}       Fast (1-2 seconds)                        {GREEN}│{RESET}");
    println!("{GREEN}│{RESET}  {BOLD}Cost:{RESET}        Free tier: 60 requests/minute             {GREEN}│{RESET}");
    println!("{GREEN}│{RESET}  {BOLD}Setup:{RESET}       Get API key from Google AI Studio         {GREEN}│{RESET}");
    println!("{GREEN}│{RESET}                                                           {GREEN}│{RESET}");
    println!("{GREEN}│{RESET}  {DIM}WHAT YOU'RE LEARNING:{RESET}                                   {GREEN}│{RESET}");
    println!("{GREEN}│{RESET}  {DIM}Cloud APIs let you use powerful AI without running{RESET}     {GREEN}│{RESET}");
    println!("{GREEN}│{RESET}  {DIM}models locally. Trade-off: prompts sent to servers.{RESET}    {GREEN}│{RESET}");
    println!("{GREEN}│{RESET}                                                           {GREEN}│{RESET}");
    println!("{GREEN}└───────────────────────────────────────────────────────────┘{RESET}\n");

    println!("{YELLOW}┌─ OPTION 2: Ollama (Local) ───────────────────────────────┐{RESET}");
    println!("{YELLOW}│{RESET}                                                           {YELLOW}│{RESET}");
    println!("{YELLOW}│{RESET}  {BOLD}What it is:{RESET}  LLMs running on YOUR machine             {YELLOW}│{RESET}");
    println!("{YELLOW}│{RESET}  {BOLD}Speed:{RESET}       Depends on hardware (5-30 seconds)       {YELLOW}│{RESET}");
    println!("{YELLOW}│{RESET}  {BOLD}Cost:{RESET}        Free forever, uses your GPU/CPU          {YELLOW}│{RESET}");
    println!("{YELLOW}│{RESET}  {BOLD}Setup:{RESET}       Install Ollama + download a model        {YELLOW}│{RESET}");
    println!("{YELLOW}│{RESET}                                                           {YELLOW}│{RESET}");
    println!("{YELLOW}│{RESET}  {DIM}WHAT YOU'RE LEARNING:{RESET}                                   {YELLOW}│{RESET}");
    println!("{YELLOW}│{RESET}  {DIM}Local LLMs keep data 100% private. Great for{RESET}           {YELLOW}│{RESET}");
    println!("{YELLOW}│{RESET}  {DIM}enterprise or security-sensitive work.{RESET}                 {YELLOW}│{RESET}");
    println!("{YELLOW}│{RESET}                                                           {YELLOW}│{RESET}");
    println!("{YELLOW}└───────────────────────────────────────────────────────────┘{RESET}\n");

    // ══════════════════════════════════════════════════════════════
    // DETECT EXISTING SETUP
    // ══════════════════════════════════════════════════════════════
    let gemini_configured = std::env::var("GEMINI_API_KEY").is_ok()
        || config.gemini_api_key.as_ref().map_or(false, |k| !k.is_empty());
    let ollama_available = check_ollama_available().await;

    println!("{DIM}Checking your current setup...{RESET}\n");

    if gemini_configured {
        println!("  {GREEN}✓{RESET} Gemini API key detected");
    } else {
        println!("  {DIM}○{RESET} Gemini API key not configured");
    }

    if ollama_available {
        println!("  {GREEN}✓{RESET} Ollama is running");
        if let Ok(models) = OllamaBackend::new().list_models().await {
            if !models.is_empty() {
                println!("    {DIM}Available models: {}{RESET}", models.join(", "));
            }
        }
    } else {
        println!("  {DIM}○{RESET} Ollama not detected");
    }
    println!();

    // ══════════════════════════════════════════════════════════════
    // CHOOSE SETUP PATH
    // ══════════════════════════════════════════════════════════════
    println!("Which would you like to set up?\n");
    println!("  {GREEN}1{RESET}) Gemini API (cloud, fast, easy)");
    println!("  {YELLOW}2{RESET}) Ollama (local, private, learn more)");
    println!("  {CYAN}3{RESET}) Both (recommended for flexibility)");
    println!("  {DIM}4{RESET}) Skip for now\n");

    print!("Your choice [{GREEN}1{RESET}/{YELLOW}2{RESET}/{CYAN}3{RESET}/{DIM}4{RESET}]: ");
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice = choice.trim();

    match choice {
        "1" => setup_gemini(&mut config).await?,
        "2" => setup_ollama(&mut config).await?,
        "3" | "" => {
            setup_gemini(&mut config).await?;
            println!();
            setup_ollama(&mut config).await?;
        }
        _ => {
            println!("\n{DIM}Skipping setup. You can run 'kaido init' anytime.{RESET}");
        }
    }

    // ══════════════════════════════════════════════════════════════
    // SAVE & COMPLETE
    // ══════════════════════════════════════════════════════════════
    println!("\n{DIM}Saving configuration...{RESET}");
    config.save()?;

    println!("\n{GREEN}╭───────────────────────────────────────────────────────────╮{RESET}");
    println!("{GREEN}│{RESET}                                                           {GREEN}│{RESET}");
    println!("{GREEN}│{RESET}   {BOLD}Setup Complete!{RESET}                                       {GREEN}│{RESET}");
    println!("{GREEN}│{RESET}                                                           {GREEN}│{RESET}");
    println!("{GREEN}│{RESET}   Config saved to: ~/.kaido/config.toml                   {GREEN}│{RESET}");
    println!("{GREEN}│{RESET}                                                           {GREEN}│{RESET}");
    println!("{GREEN}│{RESET}   Run {CYAN}kaido{RESET} to start your AI Ops Coach!                 {GREEN}│{RESET}");
    println!("{GREEN}│{RESET}                                                           {GREEN}│{RESET}");
    println!("{GREEN}│{RESET}   Try: \"check what's using port 80\"                       {GREEN}│{RESET}");
    println!("{GREEN}│{RESET}        \"show disk usage\"                                  {GREEN}│{RESET}");
    println!("{GREEN}│{RESET}        \"find large files in current directory\"            {GREEN}│{RESET}");
    println!("{GREEN}│{RESET}                                                           {GREEN}│{RESET}");
    println!("{GREEN}╰───────────────────────────────────────────────────────────╯{RESET}\n");

    Ok(())
}

/// Setup Gemini API with learning content
async fn setup_gemini(config: &mut Config) -> anyhow::Result<()> {
    println!("\n{GREEN}━━━ Setting up Gemini API ━━━{RESET}\n");

    // Teaching moment: What is an API key?
    println!("{DIM}┌─ WHAT YOU'RE LEARNING ─────────────────────────────────────┐{RESET}");
    println!("{DIM}│{RESET}                                                             {DIM}│{RESET}");
    println!("{DIM}│{RESET}  An {BOLD}API key{RESET} is like a password that identifies you to a   {DIM}│{RESET}");
    println!("{DIM}│{RESET}  service. It lets Google track your usage and apply rate   {DIM}│{RESET}");
    println!("{DIM}│{RESET}  limits. Keep it secret!                                   {DIM}│{RESET}");
    println!("{DIM}│{RESET}                                                             {DIM}│{RESET}");
    println!("{DIM}│{RESET}  {BOLD}Security tip:{RESET} Store keys in .env files, never in git!   {DIM}│{RESET}");
    println!("{DIM}│{RESET}                                                             {DIM}│{RESET}");
    println!("{DIM}└─────────────────────────────────────────────────────────────┘{RESET}\n");

    println!("Get your free API key from:");
    println!("{CYAN}  https://aistudio.google.com/app/apikey{RESET}\n");

    // Check for existing key
    if let Some(existing) = &config.gemini_api_key {
        if !existing.is_empty() {
            let masked = format!("{}...{}", &existing[..8.min(existing.len())],
                &existing[existing.len().saturating_sub(4)..]);
            println!("Current key: {DIM}{masked}{RESET}");
            print!("Keep this key? [Y/n]: ");
            io::stdout().flush()?;

            let mut response = String::new();
            io::stdin().read_line(&mut response)?;
            if response.trim().to_lowercase() != "n" {
                println!("{GREEN}✓{RESET} Keeping existing key");
                return Ok(());
            }
        }
    }

    // Also check environment variable
    if let Ok(env_key) = std::env::var("GEMINI_API_KEY") {
        if !env_key.is_empty() {
            println!("{GREEN}✓{RESET} Found GEMINI_API_KEY in environment");
            println!("{DIM}  Using environment variable (recommended for security){RESET}");
            return Ok(());
        }
    }

    print!("Enter your Gemini API key: ");
    io::stdout().flush()?;

    let mut api_key = String::new();
    io::stdin().read_line(&mut api_key)?;
    let api_key = api_key.trim().to_string();

    if api_key.is_empty() {
        println!("{DIM}Skipped. Set GEMINI_API_KEY env var or run 'kaido init' later.{RESET}");
        return Ok(());
    }

    // Validate
    print!("{DIM}Validating API key...{RESET} ");
    io::stdout().flush()?;

    let gemini = GeminiBackend::with_api_key(api_key.clone());
    match gemini.infer("Say 'OK' if you can hear me.").await {
        Ok(_) => {
            println!("{GREEN}✓ Valid!{RESET}");
            config.gemini_api_key = Some(api_key);

            // Teaching moment: .env recommendation
            println!("\n{DIM}┌─ PRO TIP ─────────────────────────────────────────────────┐{RESET}");
            println!("{DIM}│{RESET}                                                           {DIM}│{RESET}");
            println!("{DIM}│{RESET}  Instead of config, use a .env file in your project:     {DIM}│{RESET}");
            println!("{DIM}│{RESET}                                                           {DIM}│{RESET}");
            println!("{DIM}│{RESET}    echo \"GEMINI_API_KEY=your_key\" > .env                 {DIM}│{RESET}");
            println!("{DIM}│{RESET}    echo \".env\" >> .gitignore                             {DIM}│{RESET}");
            println!("{DIM}│{RESET}                                                           {DIM}│{RESET}");
            println!("{DIM}│{RESET}  Kaido automatically loads .env files!                   {DIM}│{RESET}");
            println!("{DIM}│{RESET}                                                           {DIM}│{RESET}");
            println!("{DIM}└───────────────────────────────────────────────────────────┘{RESET}");
        }
        Err(e) => {
            println!("{YELLOW}⚠ Warning: {e}{RESET}");
            println!("{DIM}Saving anyway - you can fix it later.{RESET}");
            config.gemini_api_key = Some(api_key);
        }
    }

    Ok(())
}

/// Setup Ollama with learning content
async fn setup_ollama(config: &mut Config) -> anyhow::Result<()> {
    println!("\n{YELLOW}━━━ Setting up Ollama ━━━{RESET}\n");

    // Teaching moment: What is Ollama?
    println!("{DIM}┌─ WHAT YOU'RE LEARNING ─────────────────────────────────────┐{RESET}");
    println!("{DIM}│{RESET}                                                             {DIM}│{RESET}");
    println!("{DIM}│{RESET}  {BOLD}Ollama{RESET} runs LLMs locally on your machine. It handles     {DIM}│{RESET}");
    println!("{DIM}│{RESET}  model downloading, GPU acceleration, and provides an API. {DIM}│{RESET}");
    println!("{DIM}│{RESET}                                                             {DIM}│{RESET}");
    println!("{DIM}│{RESET}  Popular models:                                            {DIM}│{RESET}");
    println!("{DIM}│{RESET}    • llama3.2 (3B) - Fast, good for simple tasks           {DIM}│{RESET}");
    println!("{DIM}│{RESET}    • mistral (7B)  - Balanced speed and quality            {DIM}│{RESET}");
    println!("{DIM}│{RESET}    • qwen2.5 (7B)  - Good multilingual support             {DIM}│{RESET}");
    println!("{DIM}│{RESET}                                                             {DIM}│{RESET}");
    println!("{DIM}└─────────────────────────────────────────────────────────────┘{RESET}\n");

    // Check if Ollama is running
    let ollama = OllamaBackend::new();

    if !ollama.is_available().await {
        println!("{YELLOW}Ollama is not running.{RESET}\n");

        // Check if ollama binary exists
        let ollama_installed = which::which("ollama").is_ok();

        if ollama_installed {
            println!("Ollama is installed but not running.\n");
            println!("Start it with:");
            println!("{CYAN}  ollama serve{RESET}\n");
            println!("{DIM}(Run this in another terminal, then re-run 'kaido init'){RESET}");
        } else {
            println!("Ollama is not installed.\n");
            println!("Install from: {CYAN}https://ollama.ai{RESET}\n");

            #[cfg(target_os = "macos")]
            println!("Or with Homebrew: {CYAN}brew install ollama{RESET}\n");

            #[cfg(target_os = "linux")]
            println!("Or: {CYAN}curl -fsSL https://ollama.ai/install.sh | sh{RESET}\n");

            println!("After installing:");
            println!("  1. Start Ollama: {CYAN}ollama serve{RESET}");
            println!("  2. Pull a model: {CYAN}ollama pull llama3.2{RESET}");
            println!("  3. Re-run: {CYAN}kaido init{RESET}");
        }

        config.provider = AIProvider::Gemini;
        return Ok(());
    }

    println!("{GREEN}✓{RESET} Ollama is running at {}", config.ollama.base_url);

    // List available models
    match ollama.list_models().await {
        Ok(models) if !models.is_empty() => {
            println!("\nAvailable models:");
            for (i, model) in models.iter().enumerate() {
                let marker = if model == &config.ollama.model {
                    format!("{GREEN}→{RESET}")
                } else {
                    format!("{DIM} {RESET}")
                };
                println!("  {marker} {i}. {model}");
            }
            println!();

            print!("Choose a model (number or name) [{GREEN}{}{RESET}]: ", config.ollama.model);
            io::stdout().flush()?;

            let mut choice = String::new();
            io::stdin().read_line(&mut choice)?;
            let choice = choice.trim();

            if !choice.is_empty() {
                // Try as number first
                if let Ok(idx) = choice.parse::<usize>() {
                    if idx < models.len() {
                        config.ollama.model = models[idx].clone();
                    }
                } else {
                    // Use as model name directly
                    config.ollama.model = choice.to_string();
                }
            }

            println!("{GREEN}✓{RESET} Selected model: {}", config.ollama.model);
        }
        Ok(_) => {
            println!("\n{YELLOW}No models found.{RESET}\n");
            println!("Download a model first:");
            println!("{CYAN}  ollama pull llama3.2{RESET}     # Fast, 2GB");
            println!("{CYAN}  ollama pull mistral{RESET}      # Balanced, 4GB");
            println!("{CYAN}  ollama pull qwen2.5{RESET}      # Multilingual, 4GB\n");

            config.provider = AIProvider::Gemini;
            return Ok(());
        }
        Err(e) => {
            println!("{YELLOW}Could not list models: {e}{RESET}");
        }
    }

    // Test the model
    print!("\n{DIM}Testing model...{RESET} ");
    io::stdout().flush()?;

    let test_ollama = OllamaBackend::with_config(config.ollama.clone());
    match test_ollama.infer("Say 'OK' if you can hear me.").await {
        Ok(_) => {
            println!("{GREEN}✓ Working!{RESET}");

            // If Gemini is also configured, use Auto mode
            if config.gemini_api_key.as_ref().map_or(false, |k| !k.is_empty())
                || std::env::var("GEMINI_API_KEY").is_ok() {
                config.provider = AIProvider::Auto;
                println!("\n{DIM}Using Auto mode: Gemini (fast) → Ollama (private fallback){RESET}");
            } else {
                config.provider = AIProvider::Ollama;
            }
        }
        Err(e) => {
            println!("{YELLOW}⚠ {e}{RESET}");

            if e.to_string().contains("not found") {
                println!("\nThe model might not be downloaded. Try:");
                println!("{CYAN}  ollama pull {}{RESET}", config.ollama.model);
            }
        }
    }

    Ok(())
}

/// Check if Ollama is available
async fn check_ollama_available() -> bool {
    OllamaBackend::new().is_available().await
}

/// Non-interactive init
async fn run_init_non_interactive(config: &mut Config) -> anyhow::Result<()> {
    println!("Running non-interactive setup...\n");

    // Check for Gemini API key in environment
    if std::env::var("GEMINI_API_KEY").is_ok() {
        println!("{GREEN}✓{RESET} GEMINI_API_KEY found in environment");
    }

    // Check for Ollama
    if check_ollama_available().await {
        println!("{GREEN}✓{RESET} Ollama is available");
        config.provider = AIProvider::Auto;
    } else if std::env::var("GEMINI_API_KEY").is_ok() {
        config.provider = AIProvider::Gemini;
    }

    config.save()?;
    println!("\nConfiguration saved.");

    Ok(())
}
