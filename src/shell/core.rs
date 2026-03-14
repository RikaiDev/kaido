use crate::shell::ai::AIProcessor;
use crate::shell::executor::CommandExecutor;
use crate::shell::learning::LearningTracker;
use crate::shell::parser::CommandParser;
use crate::shell::skills::SkillsRegistry;
use crate::shell::plugin::PluginManager;
use crate::shell::theme::Theme;
use crate::shell::palette::CommandPalette;
use anyhow::Result;
use std::io::{self, Read, Write};

const CYAN: &str = "\x1b[38;5;87m";
const GREEN: &str = "\x1b[38;5;154m";
const YELLOW: &str = "\x1b[38;5;227m";
const RESET: &str = "\x1b[0m";

pub struct Shell {
    pub running: bool,
    pub learning: LearningTracker,
    parser: CommandParser,
    executor: CommandExecutor,
    ai: AIProcessor,
    skills: SkillsRegistry,
    plugins: PluginManager,
    theme: Theme,
    palette: CommandPalette,
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
            theme: Theme::load(),
            palette: CommandPalette::default(),
        })
    }
    
    fn get_git_branch(&self, cwd: &str) -> Option<String> {
        let git_dir = std::path::Path::new(cwd).join(".git");
        if !git_dir.exists() {
            return None;
        }
        
        // Try to read HEAD ref
        let head = std::path::Path::new(cwd).join(".git").join("HEAD");
        if let Ok(content) = std::fs::read_to_string(&head) {
            if let Some(branch) = content.strip_prefix("ref: refs/heads/") {
                return Some(branch.trim().to_string());
            }
        }
        None
    }
    
    fn build_prompt(&self) -> String {
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "~".to_string());
        
        // Home directory shorthand
        let home = std::env::var("HOME").unwrap_or_default();
        let display_cwd = if home.is_empty() {
            cwd.clone()
        } else {
            cwd.replace(&home, self.theme.get_home_symbol())
        };
        
        // Check git branch
        let git_branch = self.get_git_branch(&cwd);
        
        // Build prompt with theme
        let mut prompt = format!("{CYAN}{display_cwd}{RESET}");
        
        if let Some(branch) = git_branch {
            let git_symbol = self.theme.get_git_symbol();
            prompt.push_str(&format!(" {YELLOW}on{RESET} {GREEN}{git_symbol} {branch}{RESET}"));
        }
        
        let symbol = self.theme.get_symbol();
        prompt.push_str(&format!(" {GREEN}{symbol}{RESET} "));
        
        prompt
    }
    
    pub async fn run_command_palette(&mut self) -> Result<()> {
        println!();
        
        let mut query = String::new();
        let mut selected = 0;
        
        loop {
            // Clear and show palette
            print!("\x1b[2J\x1b[H");
            let items = self.palette.filter(&query);
            
            if items.is_empty() {
                println!("{YELLOW}No matches found{RESET}");
                break;
            }
            
            self.palette.display(&items, selected);
            
            // Read key
            let mut key = [0u8; 1];
            if std::io::stdin().read(&mut key).ok() != Some(1) {
                break;
            }
            
            match key[0] {
                27 => break, // Escape
                10 | 13 => { // Enter
                    if let Some(item) = items.get(selected) {
                        println!("\n{GREEN}→ {}{RESET}", item.command);
                        return Ok(());
                    }
                }
                65 => { // Up
                    if selected > 0 {
                        selected -= 1;
                    }
                }
                66 => { // Down
                    if selected < items.len() - 1 {
                        selected += 1;
                    }
                }
                127 => { // Backspace
                    query.pop();
                }
                _ => {
                    query.push(key[0] as char);
                }
            }
        }
        
        Ok(())
    }
    
    pub async fn run(&mut self) -> Result<()> {
        // ASCII banner on start
        println!("{CYAN}");
        println!("  _  __     _     _       ");
        println!(" | |/ /__ _(_) __| | ___  ");
        println!(" | ' // _` | |/ _` |/ _ \\ ");
        println!(" | . \\ (_| | | (_| | (_) |");
        println!(" |_|\\_\\__,_|_|\\__,_|\\___/ ");
        println!("{RESET}");
        println!("{YELLOW}AI Shell{RESET} - Your intelligent ops companion");
        println!();
        
        while self.running {
            let prompt = self.build_prompt();
            print!("{}", prompt);
            io::stdout().flush()?;

            let mut input = String::new();
            if io::stdin().read_line(&mut input).unwrap() == 0 {
                break;
            }
            
            let mut input = input.trim().to_string();
            if input.is_empty() {
                continue;
            }
            
            // Check for command palette trigger
            if input == "/palette" || input == "/cmd" {
                self.run_command_palette().await?;
                continue;
            }
            
            // Check plugin hooks
            if let Some(hook_result) = self.plugins.on_command(&input) {
                match hook_result {
                    crate::shell::plugin::HookResult::Suggestion(s) => {
                        println!("{YELLOW}💡 Suggestion: {}{RESET}", s);
                    }
                    crate::shell::plugin::HookResult::Modified(cmd) => {
                        println!("{YELLOW}→ Modified: {}{RESET}", cmd);
                        input = cmd;
                    }
                    crate::shell::plugin::HookResult::None => {}
                }
            }
            
            if self.handle_builtin(&input) {
                continue;
            }
            
            if self.ai.is_natural_language(&input) {
                println!("→ Detected natural language: {}", input);
                println!("→ (AI translation requires Ollama)");
                continue;
            }
            
            match self.parser.parse(&input) {
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
                            
                            self.learning.record_command(&input);
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
