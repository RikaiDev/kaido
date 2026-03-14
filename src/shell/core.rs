use crate::shell::ai::AIProcessor;
use crate::shell::executor::CommandExecutor;
use crate::shell::learning::LearningTracker;
use crate::shell::parser::CommandParser;
use crate::shell::skills::SkillsRegistry;
use crate::shell::plugin::PluginManager;
use anyhow::Result;
use std::io::{self, Write};

pub struct Shell {
    pub running: bool,
    pub learning: LearningTracker,
    parser: CommandParser,
    executor: CommandExecutor,
    ai: AIProcessor,
    skills: SkillsRegistry,
    plugins: PluginManager,
}

impl Shell {
    pub fn new() -> Result<Self> {
        Ok(Self {
            running: true,
            learning: LearningTracker::new(),
            parser: CommandParser::new(),
            executor: CommandExecutor::new(),
            ai: AIProcessor::new(),
            skills: SkillsRegistry::load(),
            plugins: PluginManager::new(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("Welcome to Kaido AI Shell!");
        println!("Type commands or natural language. /progress to see learning progress.\n");

        while self.running {
            print!("kaido> ");
            io::stdout().flush()?;

            let mut input = String::new();
            if io::stdin().read_line(&mut input).unwrap() == 0 {
                break;
            }
            
            let input = input.trim();
            if input.is_empty() {
                continue;
            }
            
            if self.handle_builtin(input) {
                continue;
            }
            
            if self.ai.is_natural_language(input) {
                println!("→ Detected natural language: {}", input);
                println!("→ (AI translation requires Ollama)");
                continue;
            }
            
            match self.parser.parse(input) {
                Ok(parsed) => {
                    match self.executor.execute(&parsed.command, &parsed.args.iter().map(|s| s.as_str()).collect::<Vec<_>>()) {
                        Ok(output) => {
                            print!("{}", String::from_utf8_lossy(&output.stdout));
                            if !output.stderr.is_empty() {
                                eprint!("{}", String::from_utf8_lossy(&output.stderr));
                                
                                let error_msg = String::from_utf8_lossy(&output.stderr);
                                if let Some(skill) = self.skills.match_error(&error_msg) {
                                    println!("\n💡 Learn: {}", skill.teaches.join(", "));
                                } else {
                                    let explanation = self.ai.explain_error(&error_msg);
                                    println!("\n💡 {}", explanation);
                                }
                            }
                            
                            self.learning.record_command(input);
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Parse error: {}", e);
                }
            }
        }
        
        Ok(())
    }

    fn handle_builtin(&mut self, cmd: &str) -> bool {
        match cmd {
            "/progress" | "progress" => {
                println!("{}", self.learning.get_progress());
                true
            }
            "/quit" | "/exit" | "quit" | "exit" => {
                self.running = false;
                true
            }
            _ => false,
        }
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new().expect("Shell::new() should not fail")
    }
}
