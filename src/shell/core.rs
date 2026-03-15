use crate::shell::ai::AIProcessor;
use crate::shell::executor::CommandExecutor;
use crate::shell::learning::LearningTracker;
use crate::shell::parser::CommandParser;
use crate::shell::skills::SkillsRegistry;
use crate::shell::plugin::{PluginManager, ShellEvent};
use crate::shell::theme::Theme;
use crate::shell::palette::CommandPalette;
use crate::coach::{CoachResponse, ui::SidePanel};
use anyhow::Result;
use std::io::{self, Read, Write};
use tokio::runtime::Handle;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

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
    pub skills: SkillsRegistry,
    plugins: PluginManager,
    theme: Theme,
    palette: CommandPalette,
    pub coach_response: Option<CoachResponse>,
    pub last_output: String,
    pub last_error: String,
}

impl Shell {
    pub fn new() -> Result<Self> {
        let plugins = PluginManager::load_from_config()?;
        
        Ok(Self {
            running: true,
            learning: LearningTracker::new(),
            parser: CommandParser::new(),
            executor: CommandExecutor::new(),
            ai: AIProcessor::new(),
            skills: SkillsRegistry::load(),
            plugins,
            theme: Theme::load(),
            palette: CommandPalette::default(),
            coach_response: None,
            last_output: String::new(),
            last_error: String::new(),
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
            
            let input = input.trim().to_string();
            if input.is_empty() {
                continue;
            }
            
            // Check for command palette trigger
            if input == "/palette" || input == "/cmd" {
                self.run_command_palette().await?;
                continue;
            }
            
            // Check plugin events - emit CommandExecuted event
            let diagnostics = self.plugins.collect_diagnostics(&ShellEvent::CommandExecuted {
                cmd: input.clone(),
                exit_code: 0,
                output: "".to_string(),
            });
            if !diagnostics.is_empty() {
                for ctx in &diagnostics {
                    println!("{YELLOW}🔧 Diagnostics:{RESET} {}", ctx.category);
                    for cmd in &ctx.commands {
                        println!("  → {}", cmd.purpose);
                        println!("    {}", cmd.cmd);
                    }
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
                                let error_msg = String::from_utf8_lossy(&output.stderr);
                                eprint!("{}", error_msg);
                                
                                // Get diagnostics from plugins
                                let diagnostics = self.plugins.collect_diagnostics(&ShellEvent::ErrorOccurred {
                                    cmd: input.clone(),
                                    error: error_msg.to_string(),
                                    exit_code: output.status.code(),
                                });
                                
                            // Get relevant skill knowledge (clone to avoid borrow issues)
                            let skill = self.skills.match_error_cloned(&error_msg);
                            
                            // Use AI with full context (async)
                            let cmd_input = input.clone();
                            let error_str = error_msg.to_string();
                            let error_str_for_fallback = error_str.clone();
                            let ai_explanation = Handle::current()
                                .spawn(async move {
                                    let ai = AIProcessor::with_model("qwen2.5:1.5b");
                                    ai.explain_error_with_context(&cmd_input, &error_str, &diagnostics, skill.as_ref()).await
                                })
                                .await
                                .unwrap_or_else(|_| self.ai.explain_error(&error_str_for_fallback));
                            
                            println!("\n{YELLOW}🔧 {ai_explanation}{RESET}");
                        }
                        
                        // Emit success event for learning
                            let _ = self.plugins.emit(&ShellEvent::CommandExecuted {
                                cmd: input.clone(),
                                exit_code: output.status.code().unwrap_or(0),
                                output: String::from_utf8_lossy(&output.stdout).to_string(),
                            });
                            
                            self.learning.record_command(&input);
                        }
                        Err(e) => {
                            let error_msg = e.to_string();
                            eprintln!("Error: {}", error_msg);
                            
                            // Get diagnostics
                            let diagnostics = self.plugins.collect_diagnostics(&ShellEvent::ErrorOccurred {
                                cmd: input.clone(),
                                error: error_msg.clone(),
                                exit_code: None,
                            });
                            
                            // Get relevant skill knowledge (clone to avoid borrow issues)
                            let skill = self.skills.match_error_cloned(&error_msg);
                            
                            // Use AI with full context (async)
                            let cmd_input = input.clone();
                            let error_str = error_msg.clone();
                            let ai_explanation = Handle::current()
                                .spawn(async move {
                                    let ai = AIProcessor::with_model("qwen2.5:1.5b");
                                    ai.explain_error_with_context(&cmd_input, &error_str, &diagnostics, skill.as_ref()).await
                                })
                                .await
                                .unwrap_or_else(|_| "Could not generate explanation".to_string());
                            
                            println!("\n{YELLOW}🔧 {ai_explanation}{RESET}");
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

    pub async fn run_tui(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let panel_width = 45u16;
        let side_panel = SidePanel::new(panel_width);
        let mut input = String::new();
        let mut cursor_position = 0;

        loop {
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Min(20),
                        Constraint::Length(panel_width),
                    ])
                    .split(f.size());

                let main_area = chunks[0];
                let prompt = self.build_prompt();

                // Show last command output above input
                let mut lines = Vec::new();
                if !self.last_output.is_empty() {
                    for line in self.last_output.lines().take(10) {
                        lines.push(Line::from(line));
                    }
                    lines.push(Line::from(""));
                }
                lines.push(Line::from(format!("{}{}", prompt, input)));

                let input_widget = Paragraph::new(lines)
                    .block(Block::default().borders(Borders::NONE))
                    .style(Style::default().fg(Color::White));
                f.render_widget(input_widget, main_area);

                side_panel.render(f, f.size(), self.coach_response.as_ref());
            })?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char(c) => {
                                input.insert(cursor_position, c);
                                cursor_position += 1;
                            }
                            KeyCode::Backspace => {
                                if cursor_position > 0 {
                                    cursor_position -= 1;
                                    input.remove(cursor_position);
                                }
                            }
                            KeyCode::Left => {
                                if cursor_position > 0 {
                                    cursor_position -= 1;
                                }
                            }
                            KeyCode::Right => {
                                if cursor_position < input.len() {
                                    cursor_position += 1;
                                }
                            }
                            KeyCode::Enter => {
                                let cmd = input.trim().to_string();
                                if !cmd.is_empty() {
                                    self.last_output = format!("$ {}\n", cmd);
                                    self.last_output = cmd.clone();
                                    self.execute_command().await?;
                                    // Redraw to show updated side panel
                                    terminal.draw(|f| {
                                        let chunks = Layout::default()
                                            .direction(Direction::Horizontal)
                                            .constraints([
                                                Constraint::Min(20),
                                                Constraint::Length(panel_width),
                                            ])
                                            .split(f.size());

                                        let main_area = chunks[0];
                                        let prompt = self.build_prompt();

                                        let mut lines = Vec::new();
                                        if !self.last_output.is_empty() {
                                            for line in self.last_output.lines().take(10) {
                                                lines.push(Line::from(line));
                                            }
                                            lines.push(Line::from(""));
                                        }
                                        lines.push(Line::from(format!("{}{}", prompt, input)));

                                        let input_widget = Paragraph::new(lines)
                                            .block(Block::default().borders(Borders::NONE))
                                            .style(Style::default().fg(Color::White));
                                        f.render_widget(input_widget, main_area);

                                        side_panel.render(f, f.size(), self.coach_response.as_ref());
                                    })?;
                                }
                                input.clear();
                                cursor_position = 0;
                            }
                            KeyCode::Esc => {
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    async fn execute_command(&mut self) -> Result<()> {
        // Get input from the input buffer set by caller
        let input = self.last_output.clone();
        
        if input == "/progress" || input == "progress" {
            self.last_output = self.learning.get_progress();
            return Ok(());
        }
        if input == "/quit" || input == "/exit" || input == "quit" || input == "exit" {
            self.running = false;
            return Ok(());
        }
        if input == "/palette" || input == "/cmd" {
            self.run_command_palette().await?;
            return Ok(());
        }

        self.plugins.collect_diagnostics(&ShellEvent::CommandExecuted {
            cmd: input.clone(),
            exit_code: 0,
            output: String::new(),
        });

        if self.ai.is_natural_language(&input) {
            self.last_output = "→ Detected natural language: use Ollama for AI translation".to_string();
            return Ok(());
        }

        match self.parser.parse(&input) {
            Ok(parsed) => {
                match self.executor.execute(&parsed.command, &parsed.args.iter().map(|s| s.as_str()).collect::<Vec<_>>()) {
                    Ok(output) => {
                        self.last_output = String::from_utf8_lossy(&output.stdout).to_string();
                        if !output.stderr.is_empty() {
                            self.last_error = String::from_utf8_lossy(&output.stderr).to_string();

                            let diagnostics = self.plugins.collect_diagnostics(&ShellEvent::ErrorOccurred {
                                cmd: input.clone(),
                                error: self.last_error.clone(),
                                exit_code: output.status.code(),
                            });

                            let skill = self.skills.match_error_cloned(&self.last_error);

                            let cmd_input = input.clone();
                            let error_str = self.last_error.clone();
                            let error_str_for_fallback = error_str.clone();
                            let explanation = Handle::current()
                                .spawn(async move {
                                    let ai = AIProcessor::with_model("qwen2.5:1.5b");
                                    ai.explain_error_with_context(&cmd_input, &error_str, &diagnostics, skill.as_ref()).await
                                })
                                .await
                                .unwrap_or_else(|_| self.ai.explain_error(&error_str_for_fallback));

                            let mut response = CoachResponse::new();
                            response.diagnosis = Some("Error detected".to_string());
                            response.explanation = Some(explanation.clone());
                            self.coach_response = Some(response);

                            self.last_output = format!("{}\n\n🔧 {}", self.last_output, explanation);
                        } else {
                            self.coach_response = None;
                            self.last_error.clear();
                        }

                        let _ = self.plugins.emit(&ShellEvent::CommandExecuted {
                            cmd: input.clone(),
                            exit_code: output.status.code().unwrap_or(0),
                            output: String::from_utf8_lossy(&output.stdout).to_string(),
                        });

                        self.learning.record_command(&input);
                    }
                    Err(e) => {
                        self.last_error = e.to_string();
                        self.last_output = format!("Error: {}", self.last_error);

                        let diagnostics = self.plugins.collect_diagnostics(&ShellEvent::ErrorOccurred {
                            cmd: input.clone(),
                            error: self.last_error.clone(),
                            exit_code: None,
                        });

                        let skill = self.skills.match_error_cloned(&self.last_error);

                        let cmd_input = input.clone();
                        let error_str = self.last_error.clone();
                        let explanation = Handle::current()
                            .spawn(async move {
                                let ai = AIProcessor::with_model("qwen2.5:1.5b");
                                ai.explain_error_with_context(&cmd_input, &error_str, &diagnostics, skill.as_ref()).await
                            })
                            .await
                            .unwrap_or_else(|_| "Could not generate explanation".to_string());

                        let mut response = CoachResponse::new();
                        response.diagnosis = Some("Error detected".to_string());
                        response.explanation = Some(explanation.clone());
                        self.coach_response = Some(response);

                        self.last_output = format!("{}\n\n🔧 {}", self.last_output, explanation);
                    }
                }
            }
            Err(e) => {
                self.last_output = format!("Parse error: {}", e);
                self.last_error.clear();
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
