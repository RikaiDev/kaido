use anyhow::Result;
use std::io::{self, Write};

use crate::agent::{AgentLoop, AgentStep, StepType};
use crate::ai::AIManager;
use crate::audit::AgentAuditLogger;
use crate::config::Config;
use crate::tools::ToolContext;

/// Kaido AI Agent REPL
pub struct KaidoREPL {
    ai_manager: AIManager,
    tool_context: ToolContext,
    audit_logger: Option<AgentAuditLogger>,
    config: Config,
}

impl KaidoREPL {
    /// Create new agent REPL
    pub fn new() -> Result<Self> {
        let config = Config::load().unwrap_or_else(|_| {
            log::warn!("Failed to load config, using defaults");
            Config::default()
        });

        let ai_manager = AIManager::new(config.clone());
        let tool_context = ToolContext::default();

        // Initialize audit logger
        let audit_logger = match Self::init_audit_logger() {
            Ok(logger) => {
                log::info!("Agent audit logging enabled");
                Some(logger)
            }
            Err(e) => {
                log::warn!("Agent audit logging disabled: {}", e);
                None
            }
        };

        Ok(Self {
            ai_manager,
            tool_context,
            audit_logger,
            config,
        })
    }
    
    /// Initialize audit logger
    fn init_audit_logger() -> Result<AgentAuditLogger> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
        let kaido_dir = home.join(".kaido");
        std::fs::create_dir_all(&kaido_dir)?;
        
        let db_path = kaido_dir.join("agent_audit.db");
        let logger = AgentAuditLogger::new(db_path.to_str().unwrap())?;
        
        // Clean old sessions (90 days retention)
        logger.clean_old_sessions(90)?;
        
        Ok(logger)
    }
    
    /// Run the REPL loop
    pub async fn run(&mut self) -> Result<()> {
        self.print_welcome();
        
        loop {
            // Read user input
            print!("\n\x1b[38;5;147m→\x1b[0m ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();
            
            // Handle special commands
            match input {
                "exit" | "quit" | "q" => {
                    println!("\n\x1b[38;5;245mAgent terminated\x1b[0m\n");
                    break;
                }
                "clear" | "cls" => {
                    print!("\x1B[2J\x1B[1;1H");
                    continue;
                }
                "help" | "?" => {
                    self.print_help();
                    continue;
                }
                "explain on" => {
                    self.config.display.explain_mode = true;
                    println!("\x1b[38;5;150m◆\x1b[0m Explain mode: \x1b[38;5;150mON\x1b[0m");
                    println!("  Commands will now include educational breakdowns.");
                    continue;
                }
                "explain off" => {
                    self.config.display.explain_mode = false;
                    println!("\x1b[38;5;245m◆\x1b[0m Explain mode: \x1b[38;5;245mOFF\x1b[0m");
                    println!("  Commands will execute without explanations.");
                    continue;
                }
                "explain" => {
                    let status = if self.config.display.explain_mode {
                        "\x1b[38;5;150mON\x1b[0m"
                    } else {
                        "\x1b[38;5;245mOFF\x1b[0m"
                    };
                    println!("Explain mode: {}", status);
                    println!("  Use 'explain on' or 'explain off' to toggle.");
                    continue;
                }
                "" => continue,
                _ => {}
            }
            
            // Run agent loop
            println!("\n\x1b[38;5;245m╭─ agent session initiated\x1b[0m");
            
            if let Err(e) = self.run_agent(input).await {
                println!("\n\x1b[38;5;203m◆ error:\x1b[0m {}", e);
            }
            
            println!("\x1b[38;5;245m╰─ session complete\x1b[0m");
        }

        Ok(())
    }

    /// Run agent loop for a problem
    async fn run_agent(&mut self, problem: &str) -> Result<()> {
        // Generate session ID
        let session_id = uuid::Uuid::new_v4().to_string();
        
        // Log session start
        if let Some(logger) = &self.audit_logger {
            logger.log_session_start(&session_id, problem)?;
        }
        
        let mut agent = AgentLoop::new(
            problem.to_string(),
            self.tool_context.clone()
        )
        .with_explain_mode(self.config.display.explain_mode);

        // Set up progress callback with audit logging
        let session_id_clone = session_id.clone();
        let logger_clone = self.audit_logger.clone();
        let callback = move |step: &AgentStep| {
            Self::display_step_static(step);

            // Log step to audit
            if let Some(logger) = &logger_clone {
                let _ = logger.log_step(&session_id_clone, step);
            }
        };

        agent = agent.with_progress_callback(callback);
        
        // Run until complete
        let final_state = agent.run_until_complete(&self.ai_manager).await?;
        
        // Log session end
        if let Some(logger) = &self.audit_logger {
            logger.log_session_end(&session_id, &final_state)?;
        }
        
        // Display final summary
        println!("\n\x1b[38;5;250m╭─ summary\x1b[0m");
        println!("\x1b[38;5;245m│\x1b[0m status:   \x1b[38;5;147m{:?}\x1b[0m", final_state.status);
        println!("\x1b[38;5;245m│\x1b[0m steps:    {}", final_state.history.len());
        println!("\x1b[38;5;245m│\x1b[0m actions:  {}", 
            final_state.history.iter()
                .filter(|s| s.step_type == StepType::Action)
                .count()
        );
        println!("\x1b[38;5;245m│\x1b[0m duration: {:.2}s", final_state.start_time.elapsed().as_secs_f64());
        
        if let Some(root_cause) = &final_state.root_cause {
            println!("\x1b[38;5;245m│\x1b[0m");
            println!("\x1b[38;5;245m│\x1b[0m \x1b[38;5;203mroot cause:\x1b[0m");
            println!("\x1b[38;5;245m│\x1b[0m   {}", root_cause);
        }
        
        if let Some(solution_plan) = &final_state.solution_plan {
            println!("\x1b[38;5;245m│\x1b[0m");
            println!("\x1b[38;5;245m│\x1b[0m \x1b[38;5;147msolution:\x1b[0m");
            for (i, step) in solution_plan.iter().enumerate() {
                println!("\x1b[38;5;245m│\x1b[0m   \x1b[38;5;242m{}.\x1b[0m {}", i + 1, step);
            }
        }
        
        println!("\x1b[38;5;250m╰─\x1b[0m");

        Ok(())
    }

    /// Display a single agent step (static version for callback)
    fn display_step_static(step: &AgentStep) {
        match step.step_type {
            StepType::Thought => {
                println!("\n\x1b[38;5;111m╭─ THOUGHT #{}\x1b[0m", step.step_number);
                for line in step.content.lines() {
                    println!("\x1b[38;5;245m│\x1b[0m {}", line);
                }
                println!("\x1b[38;5;245m╰─\x1b[0m");
            }
            StepType::Action => {
                println!("\n\x1b[38;5;221m╭─ ACTION #{}\x1b[0m", step.step_number);
                if let Some(tool) = &step.tool_used {
                    println!("\x1b[38;5;245m│\x1b[0m [\x1b[38;5;147m{}\x1b[0m] {}", tool, step.content);
                } else {
                    println!("\x1b[38;5;245m│\x1b[0m {}", step.content);
                }
                println!("\x1b[38;5;245m╰─\x1b[0m");

                // Display educational explanation if present (explain mode)
                if let Some(explanation) = &step.explanation {
                    println!();
                    println!("\x1b[38;5;150m┌─ WHAT YOU'RE LEARNING ─────────────────────────────────┐\x1b[0m");
                    for line in explanation.lines() {
                        println!("\x1b[38;5;150m│\x1b[0m {}", line);
                    }
                    println!("\x1b[38;5;150m└─────────────────────────────────────────────────────────┘\x1b[0m");
                    println!();
                }

                print!("\x1b[38;5;242m⟳ executing...\x1b[0m ");
                io::stdout().flush().ok();
            }
            StepType::Observation => {
                let (status, color) = if step.success.unwrap_or(false) {
                    ("✓", "\x1b[38;5;150m")
                } else {
                    ("✗", "\x1b[38;5;203m")
                };
                println!("{}{}\x1b[0m", color, status);
                
                println!("\n\x1b[38;5;250m╭─ OBSERVATION #{}\x1b[0m", step.step_number);
                let content = if step.content.len() > 600 {
                    format!("{}... \x1b[38;5;237m(truncated)\x1b[0m", &step.content[..600])
                } else {
                    step.content.clone()
                };
                for line in content.lines().take(15) {
                    println!("\x1b[38;5;245m│\x1b[0m {}", line);
                }
                println!("\x1b[38;5;245m╰─\x1b[0m");
            }
            StepType::Reflection => {
                println!("\n\x1b[38;5;183m╭─ REFLECTION #{}\x1b[0m", step.step_number);
                for line in step.content.lines() {
                    println!("\x1b[38;5;245m│\x1b[0m {}", line);
                }
                println!("\x1b[38;5;245m╰─\x1b[0m");
            }
            StepType::Solution => {
                println!("\n\x1b[38;5;147m╭─ SOLUTION\x1b[0m");
                for line in step.content.lines() {
                    println!("\x1b[38;5;245m│\x1b[0m {}", line);
                }
                println!("\x1b[38;5;245m╰─\x1b[0m");
            }
        }
    }
    
    /// Print welcome message
    fn print_welcome(&self) {
        println!("\n\x1b[38;5;147m");
        println!("     ⬡");
        println!("    ⬡ ⬡        \x1b[1mKAIDO\x1b[0;38;5;147m");
        println!("   ⬡ ⬡ ⬡       autonomous ops agent");
        println!("  ⬡ ⬡ ⬡ ⬡");
        println!(" ⬡ ⬡ ⬡ ⬡ ⬡    \x1b[38;5;245mv0.1.0\x1b[0m");
        println!();
        
        println!("\x1b[38;5;250mCapabilities:\x1b[0m");
        println!("  \x1b[38;5;147m◆\x1b[0m Autonomous problem diagnosis using ReAct reasoning");
        println!("  \x1b[38;5;147m◆\x1b[0m Multi-step diagnostic command execution");  
        println!("  \x1b[38;5;147m◆\x1b[0m Root cause analysis and solution generation");
        println!("  \x1b[38;5;147m◆\x1b[0m Complete audit trail of reasoning process");
        
        println!("\n\x1b[38;5;250mSupported Tools:\x1b[0m");
        println!("  \x1b[38;5;242mkubectl · docker · docker-compose\x1b[0m");
        println!("  \x1b[38;5;242mnginx · apache2 · network diagnostics\x1b[0m");
        println!("  \x1b[38;5;242mMySQL · PostgreSQL\x1b[0m");
        
        println!("\n\x1b[38;5;250mExample:\x1b[0m");
        println!("  \x1b[38;5;242m→\x1b[0m nginx won't start, port 80 already in use");
        println!("  \x1b[38;5;242m→\x1b[0m docker-compose services cannot connect");
        println!("  \x1b[38;5;242m→\x1b[0m kubernetes pod keeps restarting");
        
        println!("\n\x1b[38;5;237mType 'help' for commands · 'exit' to quit\x1b[0m");
    }
    
    /// Print help message
    fn print_help(&self) {
        println!("\n\x1b[38;5;250m╭─ help\x1b[0m");
        
        println!("\x1b[38;5;245m│\x1b[0m \x1b[38;5;250mUsage:\x1b[0m");
        println!("\x1b[38;5;245m│\x1b[0m   Describe your problem in natural language");
        println!("\x1b[38;5;245m│\x1b[0m   The agent will autonomously diagnose and solve it");
        
        println!("\x1b[38;5;245m│\x1b[0m");
        println!("\x1b[38;5;245m│\x1b[0m \x1b[38;5;250mCommands:\x1b[0m");
        println!("\x1b[38;5;245m│\x1b[0m   \x1b[38;5;147mhelp\x1b[0m        Show this help");
        println!("\x1b[38;5;245m│\x1b[0m   \x1b[38;5;147mclear\x1b[0m       Clear screen");
        println!("\x1b[38;5;245m│\x1b[0m   \x1b[38;5;147mexplain\x1b[0m     Toggle explain mode (on/off)");
        println!("\x1b[38;5;245m│\x1b[0m   \x1b[38;5;147mexit\x1b[0m        Quit agent");
        
        println!("\x1b[38;5;245m│\x1b[0m");
        println!("\x1b[38;5;245m│\x1b[0m \x1b[38;5;250mReAct Process:\x1b[0m");
        println!("\x1b[38;5;245m│\x1b[0m   \x1b[38;5;242m1.\x1b[0m Thought     → Decide what to investigate");
        println!("\x1b[38;5;245m│\x1b[0m   \x1b[38;5;242m2.\x1b[0m Action      → Execute diagnostic command");
        println!("\x1b[38;5;245m│\x1b[0m   \x1b[38;5;242m3.\x1b[0m Observation → Analyze output");
        println!("\x1b[38;5;245m│\x1b[0m   \x1b[38;5;242m4.\x1b[0m Reflection  → Evaluate progress");
        println!("\x1b[38;5;245m│\x1b[0m   \x1b[38;5;242m5.\x1b[0m Repeat until solved");
        
        println!("\x1b[38;5;250m╰─\x1b[0m");
    }
}

impl Default for KaidoREPL {
    fn default() -> Self {
        Self::new().expect("Failed to create REPL")
    }
}

/// Run the Kaido AI agent REPL
pub async fn run_agent_repl() -> Result<()> {
    let mut repl = KaidoREPL::new()?;
    repl.run().await
}
