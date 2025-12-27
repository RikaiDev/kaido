use clap::{Parser, Subcommand};
use kaido::shell::repl::KaidoREPL;
use kaido::config::Config;
use kaido::ai::GeminiBackend;
use kaido::tools::LLMBackend;
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "kaido")]
#[command(about = "Kaido AI - Universal Ops Assistant with AI-powered command translation", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Kaido AI configuration (setup API keys)
    Init {
        /// Skip interactive prompts and use defaults
        #[arg(long)]
        non_interactive: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if present (for API keys)
    let _ = dotenvy::dotenv();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .format_target(false)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init { non_interactive }) => {
            run_init(non_interactive).await?;
        }
        None => {
            // Initialize and run REPL
            let mut repl = KaidoREPL::new()?;
            repl.run().await?;
        }
    }

    Ok(())
}

async fn run_init(non_interactive: bool) -> anyhow::Result<()> {
    println!(">> Kaido AI - Configuration Setup\n");
    
    // Load existing config or create default
    let mut config = Config::load().unwrap_or_default();
    
    if !non_interactive {
        // Interactive setup
        println!("This wizard will help you configure Kaido AI.\n");
        
        // Gemini API Key
        println!("[*] Gemini API Key");
        println!("    Get your key from: https://aistudio.google.com/app/apikey\n");
        
        if let Some(existing_key) = &config.gemini_api_key {
            if !existing_key.is_empty() {
                println!("    Current key: {}...{}", &existing_key[..10.min(existing_key.len())], 
                    if existing_key.len() > 10 { "****" } else { "" });
                print!("    Keep existing key? [Y/n]: ");
                io::stdout().flush()?;
                
                let mut response = String::new();
                io::stdin().read_line(&mut response)?;
                
                if response.trim().to_lowercase() == "n" {
                    config.gemini_api_key = None;
                } else {
                    println!("    [OK] Keeping existing key\n");
                }
            }
        }
        
        if config.gemini_api_key.is_none() || config.gemini_api_key.as_ref().map_or(true, |k| k.is_empty()) {
            print!("    Enter your Gemini API key: ");
            io::stdout().flush()?;
            
            let mut api_key = String::new();
            io::stdin().read_line(&mut api_key)?;
            let api_key = api_key.trim().to_string();
            
            if !api_key.is_empty() {
                // Validate API key
                println!("    [?] Validating API key...");
                
                let gemini = GeminiBackend::with_api_key(api_key.clone());
                match gemini.infer("test").await {
                    Ok(_) => {
                        println!("    [OK] API key is valid!\n");
                        config.gemini_api_key = Some(api_key);
                    }
                    Err(e) => {
                        println!("    [X] API key validation failed: {}", e);
                        println!("    Saving anyway (you can fix it later)\n");
                        config.gemini_api_key = Some(api_key);
                    }
                }
            } else {
                println!("    [!] Skipped (you can set it later via environment variable)\n");
            }
        }
        
        // Safety settings
        println!("[#] Safety Settings");
        print!("    Require confirmation for destructive commands? [Y/n]: ");
        io::stdout().flush()?;
        
        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        config.safety.confirm_destructive = response.trim().to_lowercase() != "n";
        println!();
        
        // Audit settings
        println!("[=] Audit Log");
        println!("    Location: {:?}", config.audit.database_path);
        print!("    Retention days [default: {}]: ", config.audit.retention_days);
        io::stdout().flush()?;
        
        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        if let Ok(days) = response.trim().parse::<u32>() {
            config.audit.retention_days = days;
        }
        println!();
    }
    
    // Save configuration
    println!("[>] Saving configuration...");
    config.save()?;
    
    println!("[OK] Configuration saved to: {:?}", Config::get_config_path()?);
    println!("\n[+] Kaido AI is ready to use!");
    println!("\nRun 'kaido' to start the interactive shell.");
    
    if config.gemini_api_key.is_none() || config.gemini_api_key.as_ref().map_or(true, |k| k.is_empty()) {
        println!("\n[!] Remember to set your Gemini API key:");
        println!("    export GEMINI_API_KEY=your_key_here");
        println!("    or run 'kaido init' again");
    }
    
    Ok(())
}

