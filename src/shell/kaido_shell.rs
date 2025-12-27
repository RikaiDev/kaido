// Kaido Shell - Interactive mentor shell
//
// A shell wrapper that executes commands via PTY and provides
// mentorship when errors occur.

use anyhow::{Context, Result};
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::{Config, Editor};

use super::history::{ensure_history_dir, HistoryConfig};
use super::prompt::PromptBuilder;
use super::pty::{PtyExecutionResult, PtyExecutor};
use crate::mentor::{ErrorDetector, ErrorInfo, MentorDisplay, Verbosity};

/// Kaido shell configuration
#[derive(Debug, Clone)]
pub struct ShellConfig {
    /// History configuration
    pub history: HistoryConfig,
    /// Whether to show colors
    pub use_colors: bool,
    /// Whether to show git branch in prompt
    pub show_git_branch: bool,
    /// Shell to use for command execution
    pub shell: Option<String>,
    /// Mentor display verbosity level
    pub mentor_verbosity: Verbosity,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            history: HistoryConfig::default(),
            use_colors: true,
            show_git_branch: true,
            shell: None,
            mentor_verbosity: Verbosity::Normal,
        }
    }
}

/// The main Kaido shell
pub struct KaidoShell {
    /// Configuration
    config: ShellConfig,
    /// PTY executor for running commands
    pty: PtyExecutor,
    /// Readline editor with history
    editor: Editor<(), FileHistory>,
    /// Prompt builder
    prompt_builder: PromptBuilder,
    /// Error detector for mentor system
    error_detector: ErrorDetector,
    /// Mentor display for formatting guidance
    mentor_display: MentorDisplay,
    /// Whether the shell is running
    running: bool,
    /// Last execution result (for mentor system)
    last_result: Option<PtyExecutionResult>,
    /// Last detected error (for mentor system)
    last_error: Option<ErrorInfo>,
}

impl KaidoShell {
    /// Create a new Kaido shell with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(ShellConfig::default())
    }

    /// Create a new Kaido shell with custom configuration
    pub fn with_config(config: ShellConfig) -> Result<Self> {
        // Ensure history directory exists
        ensure_history_dir()?;

        // Configure rustyline
        let rl_config = Config::builder()
            .history_ignore_dups(config.history.ignore_dups)?
            .history_ignore_space(config.history.ignore_space)
            .max_history_size(config.history.max_entries)?
            .auto_add_history(true)
            .build();

        // Create editor with file history
        let mut editor = Editor::<(), FileHistory>::with_history(
            rl_config,
            FileHistory::with_config(rl_config),
        )?;

        // Load history if file exists
        if config.history.file_path.exists() {
            let _ = editor.load_history(&config.history.file_path);
        }

        // Create PTY executor
        let pty = if let Some(ref shell) = config.shell {
            PtyExecutor::with_shell(shell)
        } else {
            PtyExecutor::new()
        };

        // Create prompt builder
        let mut prompt_builder = PromptBuilder::new();
        if !config.use_colors {
            prompt_builder = prompt_builder.no_colors();
        }
        if !config.show_git_branch {
            prompt_builder = prompt_builder.no_git_branch();
        }

        // Create mentor display with config
        let mentor_display_config = crate::mentor::DisplayConfig {
            verbosity: config.mentor_verbosity,
            terminal_width: 0, // Auto-detect
            colors_enabled: config.use_colors,
        };
        let mentor_display = MentorDisplay::with_config(mentor_display_config);

        Ok(Self {
            config,
            pty,
            editor,
            prompt_builder,
            error_detector: ErrorDetector::new(),
            mentor_display,
            running: false,
            last_result: None,
            last_error: None,
        })
    }

    /// Display welcome message
    fn display_welcome(&self) {
        println!();
        println!(
            "\x1b[1;36m  _  __     _     _       \x1b[0m"
        );
        println!(
            "\x1b[1;36m | |/ /__ _(_) __| | ___  \x1b[0m"
        );
        println!(
            "\x1b[1;36m | ' // _` | |/ _` |/ _ \\ \x1b[0m"
        );
        println!(
            "\x1b[1;36m | . \\ (_| | | (_| | (_) |\x1b[0m"
        );
        println!(
            "\x1b[1;36m |_|\\_\\__,_|_|\\__,_|\\___/ \x1b[0m"
        );
        println!();
        println!("\x1b[1mYour AI Ops Mentor\x1b[0m - Learn by doing, with guidance when you need it.");
        println!();
        println!("\x1b[2mType commands normally. When errors occur, I'll help you understand them.\x1b[0m");
        println!("\x1b[2mType 'exit' or press Ctrl+D to quit.\x1b[0m");
        println!();
    }

    /// Run the shell main loop
    pub async fn run(&mut self) -> Result<()> {
        self.running = true;
        self.display_welcome();

        while self.running {
            let prompt = self.prompt_builder.build();

            match self.editor.readline(&prompt) {
                Ok(line) => {
                    let line = line.trim();

                    // Skip empty lines
                    if line.is_empty() {
                        continue;
                    }

                    // Handle built-in commands
                    if self.handle_builtin(line) {
                        continue;
                    }

                    // Execute the command
                    self.execute_command(line).await?;
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl+C - just show a new prompt
                    println!("^C");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl+D - exit
                    println!("\nGoodbye! Keep learning!");
                    self.running = false;
                }
                Err(err) => {
                    log::error!("Readline error: {}", err);
                    return Err(err.into());
                }
            }
        }

        // Save history
        self.save_history()?;

        Ok(())
    }

    /// Handle built-in shell commands
    /// Returns true if the command was handled
    fn handle_builtin(&mut self, line: &str) -> bool {
        match line {
            "exit" | "quit" => {
                println!("Goodbye! Keep learning!");
                self.running = false;
                true
            }
            "clear" => {
                print!("\x1b[2J\x1b[1;1H");
                true
            }
            "help" => {
                self.display_help();
                true
            }
            "history" => {
                self.display_history();
                true
            }
            // Verbosity commands
            "verbose" | "mentor verbose" => {
                self.set_verbosity(Verbosity::Verbose);
                println!("\x1b[36m◆\x1b[0m Mentor verbosity: \x1b[1mVerbose\x1b[0m (full explanations)");
                true
            }
            "normal" | "mentor normal" => {
                self.set_verbosity(Verbosity::Normal);
                println!("\x1b[36m◆\x1b[0m Mentor verbosity: \x1b[1mNormal\x1b[0m (key points)");
                true
            }
            "compact" | "mentor compact" => {
                self.set_verbosity(Verbosity::Compact);
                println!("\x1b[36m◆\x1b[0m Mentor verbosity: \x1b[1mCompact\x1b[0m (one-liner)");
                true
            }
            "mentor" => {
                let level = match self.config.mentor_verbosity {
                    Verbosity::Verbose => "Verbose",
                    Verbosity::Normal => "Normal",
                    Verbosity::Compact => "Compact",
                };
                println!("\x1b[36m◆\x1b[0m Mentor verbosity: \x1b[1m{}\x1b[0m", level);
                println!("  Use 'verbose', 'normal', or 'compact' to change.");
                true
            }
            _ if line.starts_with("cd ") => {
                self.handle_cd(&line[3..]);
                true
            }
            "cd" => {
                self.handle_cd("~");
                true
            }
            _ => false,
        }
    }

    /// Set mentor verbosity level
    fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.config.mentor_verbosity = verbosity;
        let display_config = crate::mentor::DisplayConfig {
            verbosity,
            terminal_width: 0,
            colors_enabled: self.config.use_colors,
        };
        self.mentor_display = MentorDisplay::with_config(display_config);
    }

    /// Display help message
    fn display_help(&self) {
        println!();
        println!("\x1b[1;36mKaido Shell - Built-in Commands\x1b[0m");
        println!();
        println!("  \x1b[1mhelp\x1b[0m       Show this help message");
        println!("  \x1b[1mhistory\x1b[0m    Show command history");
        println!("  \x1b[1mcd <dir>\x1b[0m   Change directory");
        println!("  \x1b[1mclear\x1b[0m      Clear the screen");
        println!("  \x1b[1mexit\x1b[0m       Exit the shell");
        println!();
        println!("\x1b[1;36mMentor Verbosity\x1b[0m");
        println!();
        println!("  \x1b[1mmentor\x1b[0m     Show current verbosity level");
        println!("  \x1b[1mverbose\x1b[0m    Full explanations with next steps");
        println!("  \x1b[1mnormal\x1b[0m     Key points only (default)");
        println!("  \x1b[1mcompact\x1b[0m    One-liner for experts");
        println!();
        println!("\x1b[2mAll other commands are executed in the system shell.\x1b[0m");
        println!("\x1b[2mWhen errors occur, I'll help you understand them.\x1b[0m");
        println!();
    }

    /// Display command history
    fn display_history(&self) {
        println!();
        for (i, entry) in self.editor.history().iter().enumerate() {
            println!("  {:4}  {}", i + 1, entry);
        }
        println!();
    }

    /// Handle cd command
    fn handle_cd(&mut self, path: &str) {
        let path = path.trim();

        // Expand ~ to home directory
        let expanded = if path == "~" || path.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                if path == "~" {
                    home
                } else {
                    home.join(&path[2..])
                }
            } else {
                std::path::PathBuf::from(path)
            }
        } else if path == "-" {
            // cd - : go to previous directory (would need to track this)
            println!("\x1b[33mcd -: previous directory tracking not yet implemented\x1b[0m");
            return;
        } else {
            std::path::PathBuf::from(path)
        };

        match std::env::set_current_dir(&expanded) {
            Ok(()) => {
                // Success - prompt will update automatically
            }
            Err(e) => {
                println!("\x1b[31mcd: {}: {}\x1b[0m", path, e);
            }
        }
    }

    /// Execute a command via PTY
    async fn execute_command(&mut self, command: &str) -> Result<()> {
        let result = self.pty.execute(command).await
            .context("Failed to execute command")?;

        // Print the output
        if !result.output.is_empty() {
            print!("{}", result.output);
            // Ensure output ends with newline
            if !result.output.ends_with('\n') {
                println!();
            }
        }

        // Analyze for errors using the mentor system
        if let Some(error_info) = self.error_detector.analyze(&result) {
            // Display mentor guidance
            self.display_mentor_block(&error_info);
            self.last_error = Some(error_info);
            self.last_result = Some(result);
        } else {
            self.last_error = None;
            self.last_result = None;
        }

        Ok(())
    }

    /// Display mentor guidance for detected errors
    fn display_mentor_block(&self, error: &ErrorInfo) {
        let output = self.mentor_display.render(error);
        print!("{}", output);
    }

    /// Save history to file
    fn save_history(&mut self) -> Result<()> {
        self.editor
            .save_history(&self.config.history.file_path)
            .context("Failed to save history")?;
        Ok(())
    }

    /// Get the last execution result
    pub fn last_result(&self) -> Option<&PtyExecutionResult> {
        self.last_result.as_ref()
    }

    /// Get the last detected error
    pub fn last_error(&self) -> Option<&ErrorInfo> {
        self.last_error.as_ref()
    }

    /// Check if shell is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Stop the shell
    pub fn stop(&mut self) {
        self.running = false;
    }
}

impl Default for KaidoShell {
    fn default() -> Self {
        Self::new().expect("Failed to create default KaidoShell")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_config_default() {
        let config = ShellConfig::default();
        assert!(config.use_colors);
        assert!(config.show_git_branch);
        assert!(config.shell.is_none());
    }

    #[test]
    fn test_shell_creation() {
        let shell = KaidoShell::new();
        assert!(shell.is_ok());
    }

    #[test]
    fn test_handle_builtin_exit() {
        let mut shell = KaidoShell::new().unwrap();
        assert!(shell.is_running() == false); // Not running until run() is called

        // Simulate running state
        shell.running = true;
        assert!(shell.handle_builtin("exit"));
        assert!(!shell.is_running());
    }

    #[test]
    fn test_handle_builtin_help() {
        let mut shell = KaidoShell::new().unwrap();
        assert!(shell.handle_builtin("help"));
    }

    #[test]
    fn test_handle_builtin_not_builtin() {
        let mut shell = KaidoShell::new().unwrap();
        assert!(!shell.handle_builtin("ls -la"));
        assert!(!shell.handle_builtin("echo hello"));
    }
}
