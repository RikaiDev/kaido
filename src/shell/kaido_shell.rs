// Kaido Shell - Interactive mentor shell
//
// A shell wrapper that executes commands via PTY and provides
// mentorship when errors occur.

use anyhow::{Context, Result};
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::{Config, Editor};

use std::time::Instant;

use super::builtins::{execute_builtin, parse_builtin, Builtin, BuiltinResult, ShellEnvironment};
use super::history::{ensure_history_dir, HistoryConfig};
use super::prompt::PromptBuilder;
use super::pty::{PtyExecutionResult, PtyExecutor};
use crate::learning::{LearningTracker, SessionStats, SkillDetector, SummaryGenerator, VerbosityMode};
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
    /// Verbosity mode (auto or fixed)
    pub verbosity_mode: VerbosityMode,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            history: HistoryConfig::default(),
            use_colors: true,
            show_git_branch: true,
            shell: None,
            mentor_verbosity: Verbosity::Normal,
            verbosity_mode: VerbosityMode::Auto,
        }
    }
}

/// Tracked error for resolution detection
#[derive(Debug)]
struct TrackedError {
    /// Database ID of the error
    id: i64,
    /// The command that caused the error
    command: String,
    /// When the error occurred
    timestamp: Instant,
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
    /// Shell environment (variables, aliases, previous dir)
    shell_env: ShellEnvironment,
    /// Error detector for mentor system
    error_detector: ErrorDetector,
    /// Mentor display for formatting guidance
    mentor_display: MentorDisplay,
    /// Learning tracker for progress
    learning_tracker: Option<LearningTracker>,
    /// Skill detector for adaptive verbosity
    skill_detector: SkillDetector,
    /// Session statistics for summary
    session_stats: SessionStats,
    /// Whether the shell is running
    running: bool,
    /// Last execution result (for mentor system)
    last_result: Option<PtyExecutionResult>,
    /// Last detected error (for mentor system)
    last_error: Option<ErrorInfo>,
    /// Tracked error for resolution detection
    tracked_error: Option<TrackedError>,
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

        // Try to create learning tracker (non-fatal if it fails)
        let learning_tracker = match LearningTracker::with_default_path() {
            Ok(tracker) => Some(tracker),
            Err(e) => {
                log::warn!("Failed to create learning tracker: {}", e);
                None
            }
        };

        Ok(Self {
            config,
            pty,
            editor,
            prompt_builder,
            shell_env: ShellEnvironment::new(),
            error_detector: ErrorDetector::new(),
            mentor_display,
            learning_tracker,
            skill_detector: SkillDetector::new(),
            session_stats: SessionStats::new(),
            running: false,
            last_result: None,
            last_error: None,
            tracked_error: None,
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

        // Start a learning session
        if let Some(ref mut tracker) = self.learning_tracker {
            let _ = tracker.start_session();
        }

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

                    // Try to expand aliases
                    let expanded = self.shell_env.expand_aliases(line);
                    let command = expanded.as_deref().unwrap_or(line);

                    // Execute the command
                    self.execute_command(command).await?;
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

        // Display session summary if we did anything
        if self.session_stats.commands_executed > 0 {
            self.display_session_summary();
        }

        // End learning session
        if let Some(ref mut tracker) = self.learning_tracker {
            let _ = tracker.end_session();
        }

        // Save history
        self.save_history()?;

        Ok(())
    }

    /// Display session summary
    fn display_session_summary(&self) {
        let summary = SummaryGenerator::generate(&self.session_stats);
        let output = SummaryGenerator::render(&summary);
        print!("{}", output);
    }

    /// Handle built-in shell commands
    /// Returns true if the command was handled
    fn handle_builtin(&mut self, line: &str) -> bool {
        // First check mentor-specific commands (not in builtins module)
        match line {
            "verbose" | "mentor verbose" => {
                self.config.verbosity_mode = VerbosityMode::Fixed(Verbosity::Verbose);
                self.set_verbosity(Verbosity::Verbose);
                println!("\x1b[36m◆\x1b[0m Mentor verbosity: \x1b[1mVerbose\x1b[0m (full explanations)");
                return true;
            }
            "normal" | "mentor normal" => {
                self.config.verbosity_mode = VerbosityMode::Fixed(Verbosity::Normal);
                self.set_verbosity(Verbosity::Normal);
                println!("\x1b[36m◆\x1b[0m Mentor verbosity: \x1b[1mNormal\x1b[0m (key points)");
                return true;
            }
            "compact" | "mentor compact" => {
                self.config.verbosity_mode = VerbosityMode::Fixed(Verbosity::Compact);
                self.set_verbosity(Verbosity::Compact);
                println!("\x1b[36m◆\x1b[0m Mentor verbosity: \x1b[1mCompact\x1b[0m (one-liner)");
                return true;
            }
            "mentor" => {
                let level = match self.config.mentor_verbosity {
                    Verbosity::Verbose => "Verbose",
                    Verbosity::Normal => "Normal",
                    Verbosity::Compact => "Compact",
                };
                println!("\x1b[36m◆\x1b[0m Mentor verbosity: \x1b[1m{}\x1b[0m", level);
                println!("  Use 'verbose', 'normal', or 'compact' to change.");
                return true;
            }
            "progress" | "/progress" => {
                self.display_progress();
                return true;
            }
            "skill" | "/skill" => {
                self.display_skill_assessment();
                return true;
            }
            "mentor auto" => {
                self.config.verbosity_mode = VerbosityMode::Auto;
                println!("\x1b[36m◆\x1b[0m Mentor mode: \x1b[1mAuto\x1b[0m (adapts to your skill level)");
                self.update_auto_verbosity();
                return true;
            }
            _ => {}
        }

        // Try to parse as a builtin
        if let Some(builtin) = parse_builtin(line) {
            match &builtin {
                Builtin::Help => {
                    self.display_help();
                    return true;
                }
                Builtin::History => {
                    self.display_history();
                    return true;
                }
                Builtin::Clear => {
                    print!("\x1b[2J\x1b[1;1H");
                    return true;
                }
                _ => {}
            }

            // Execute the builtin
            match execute_builtin(&builtin, &mut self.shell_env) {
                BuiltinResult::Ok(None) => {}
                BuiltinResult::Ok(Some(msg)) => {
                    println!("{}", msg);
                }
                BuiltinResult::Error(msg) => {
                    println!("\x1b[31m{}\x1b[0m", msg);
                }
                BuiltinResult::Exit(code) => {
                    if code == 0 {
                        println!("Goodbye! Keep learning!");
                    }
                    self.running = false;
                }
                BuiltinResult::Source(commands) => {
                    // Execute each command from the sourced file
                    // Note: This is synchronous; for async we'd need different handling
                    println!("\x1b[2mSourcing {} commands...\x1b[0m", commands.len());
                    for cmd in commands {
                        if !self.handle_builtin(&cmd) {
                            // Non-builtin commands from source would need async execution
                            // For now, just handle builtins from sourced files
                            println!("\x1b[33mSkipping external command: {}\x1b[0m", cmd);
                        }
                    }
                }
            }
            return true;
        }

        false
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
        println!("  \x1b[1mhelp\x1b[0m              Show this help message");
        println!("  \x1b[1mhistory\x1b[0m           Show command history");
        println!("  \x1b[1mclear\x1b[0m             Clear the screen");
        println!("  \x1b[1mexit\x1b[0m              Exit the shell");
        println!();
        println!("\x1b[1;36mDirectory & Environment\x1b[0m");
        println!();
        println!("  \x1b[1mcd <dir>\x1b[0m          Change directory");
        println!("  \x1b[1mcd -\x1b[0m              Go to previous directory");
        println!("  \x1b[1mexport VAR=val\x1b[0m    Set environment variable");
        println!("  \x1b[1munset VAR\x1b[0m         Remove environment variable");
        println!();
        println!("\x1b[1;36mAliases\x1b[0m");
        println!();
        println!("  \x1b[1malias\x1b[0m             List all aliases");
        println!("  \x1b[1malias k=kubectl\x1b[0m   Create an alias");
        println!("  \x1b[1munalias k\x1b[0m         Remove an alias");
        println!();
        println!("\x1b[1;36mScripting\x1b[0m");
        println!();
        println!("  \x1b[1msource <file>\x1b[0m     Execute commands from file");
        println!();
        println!("\x1b[1;36mMentor Verbosity\x1b[0m");
        println!();
        println!("  \x1b[1mmentor\x1b[0m            Show current verbosity level");
        println!("  \x1b[1mmentor auto\x1b[0m       Adapt to your skill level");
        println!("  \x1b[1mverbose\x1b[0m           Full explanations with next steps");
        println!("  \x1b[1mnormal\x1b[0m            Key points only (default)");
        println!("  \x1b[1mcompact\x1b[0m           One-liner for experts");
        println!();
        println!("\x1b[1;36mLearning Progress\x1b[0m");
        println!();
        println!("  \x1b[1mprogress\x1b[0m          Show your learning progress");
        println!("  \x1b[1mskill\x1b[0m             Show your skill assessment");
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

    /// Display learning progress
    fn display_progress(&self) {
        println!();

        let progress = match &self.learning_tracker {
            Some(tracker) => match tracker.get_progress() {
                Ok(p) => p,
                Err(_) => {
                    println!("\x1b[33mUnable to load learning progress.\x1b[0m");
                    println!();
                    return;
                }
            },
            None => {
                println!("\x1b[33mLearning tracker not available.\x1b[0m");
                println!();
                return;
            }
        };

        let resolution_pct = (progress.resolution_rate * 100.0) as u32;

        println!("\x1b[1;36m┌─ Your Learning Progress ─────────────────────────────────────┐\x1b[0m");
        println!("\x1b[36m│\x1b[0m                                                               \x1b[36m│\x1b[0m");
        println!(
            "\x1b[36m│\x1b[0m  Total errors encountered: \x1b[1m{:<5}\x1b[0m                              \x1b[36m│\x1b[0m",
            progress.total_errors
        );
        println!(
            "\x1b[36m│\x1b[0m  Resolution rate: \x1b[1m{}%\x1b[0m                                         \x1b[36m│\x1b[0m",
            resolution_pct
        );
        println!("\x1b[36m│\x1b[0m                                                               \x1b[36m│\x1b[0m");

        if !progress.common_errors.is_empty() {
            println!("\x1b[36m│\x1b[0m  \x1b[1mMost common errors:\x1b[0m                                        \x1b[36m│\x1b[0m");
            for (i, (error_type, count)) in progress.common_errors.iter().take(3).enumerate() {
                println!(
                    "\x1b[36m│\x1b[0m    {}. {} ({} times)                             \x1b[36m│\x1b[0m",
                    i + 1,
                    error_type,
                    count
                );
            }
            println!("\x1b[36m│\x1b[0m                                                               \x1b[36m│\x1b[0m");
        }

        if !progress.concepts.is_empty() {
            println!("\x1b[36m│\x1b[0m  \x1b[1mConcepts encountered:\x1b[0m                                       \x1b[36m│\x1b[0m");
            for concept in progress.concepts.iter().take(5) {
                println!("\x1b[36m│\x1b[0m    \x1b[32m✓\x1b[0m {}                                              \x1b[36m│\x1b[0m", concept);
            }
            println!("\x1b[36m│\x1b[0m                                                               \x1b[36m│\x1b[0m");
        }

        println!("\x1b[1;36m└───────────────────────────────────────────────────────────────┘\x1b[0m");
        println!();
    }

    /// Display skill assessment
    fn display_skill_assessment(&self) {
        println!();

        let progress = match &self.learning_tracker {
            Some(tracker) => match tracker.get_progress() {
                Ok(p) => p,
                Err(_) => {
                    println!("\x1b[33mUnable to load learning progress.\x1b[0m");
                    println!();
                    return;
                }
            },
            None => {
                println!("\x1b[33mLearning tracker not available.\x1b[0m");
                println!();
                return;
            }
        };

        let assessment = self.skill_detector.assess(&progress);

        println!("\x1b[1;36m┌─ Skill Assessment ───────────────────────────────────────────┐\x1b[0m");
        println!("\x1b[36m│\x1b[0m                                                               \x1b[36m│\x1b[0m");
        println!(
            "\x1b[36m│\x1b[0m  Level: \x1b[1m{:<20}\x1b[0m                            \x1b[36m│\x1b[0m",
            assessment.level.description()
        );
        println!(
            "\x1b[36m│\x1b[0m  Confidence: \x1b[1m{}%\x1b[0m                                            \x1b[36m│\x1b[0m",
            (assessment.confidence * 100.0) as u32
        );
        println!(
            "\x1b[36m│\x1b[0m  Score: \x1b[1m{:.2}\x1b[0m                                               \x1b[36m│\x1b[0m",
            assessment.score
        );
        println!("\x1b[36m│\x1b[0m                                                               \x1b[36m│\x1b[0m");

        if !assessment.indicators.is_empty() {
            println!("\x1b[36m│\x1b[0m  \x1b[1mIndicators:\x1b[0m                                                 \x1b[36m│\x1b[0m");
            for indicator in &assessment.indicators {
                let bar_len = (indicator.value * 10.0) as usize;
                let bar = "█".repeat(bar_len) + &"░".repeat(10 - bar_len);
                println!(
                    "\x1b[36m│\x1b[0m    {:<20} {} ({:.0}%)               \x1b[36m│\x1b[0m",
                    indicator.name,
                    bar,
                    indicator.value * 100.0
                );
            }
            println!("\x1b[36m│\x1b[0m                                                               \x1b[36m│\x1b[0m");
        }

        let recommended = assessment.level.recommended_verbosity();
        let mode_str = match self.config.verbosity_mode {
            VerbosityMode::Auto => format!("Auto ({:?})", recommended),
            VerbosityMode::Fixed(v) => format!("Fixed ({:?})", v),
        };
        println!(
            "\x1b[36m│\x1b[0m  Verbosity mode: \x1b[1m{}\x1b[0m                             \x1b[36m│\x1b[0m",
            mode_str
        );
        println!("\x1b[36m│\x1b[0m                                                               \x1b[36m│\x1b[0m");
        println!("\x1b[1;36m└───────────────────────────────────────────────────────────────┘\x1b[0m");
        println!();
    }

    /// Update verbosity based on auto mode and skill level
    fn update_auto_verbosity(&mut self) {
        if let VerbosityMode::Auto = self.config.verbosity_mode {
            if let Some(ref tracker) = self.learning_tracker {
                if let Ok(progress) = tracker.get_progress() {
                    let assessment = self.skill_detector.assess(&progress);
                    let verbosity = assessment.level.recommended_verbosity();
                    self.set_verbosity(verbosity);
                }
            }
        }
    }

    /// Execute a command via PTY
    async fn execute_command(&mut self, command: &str) -> Result<()> {
        // Track command in session stats
        self.session_stats.record_command(command);

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

        // Check if previous error was resolved (successful similar command)
        if result.exit_code == Some(0) {
            if let Some(tracked) = self.tracked_error.take() {
                if LearningTracker::is_similar_command(command, &tracked.command) {
                    // Error was resolved!
                    let resolution_time = tracked.timestamp.elapsed();
                    if let Some(ref tracker) = self.learning_tracker {
                        let _ = tracker.mark_resolved(tracked.id, resolution_time);
                    }
                    // Track resolution in session stats
                    self.session_stats.record_resolution();
                }
            }
        }

        // Analyze for errors using the mentor system
        if let Some(error_info) = self.error_detector.analyze(&result) {
            // Record error in learning tracker
            if let Some(ref tracker) = self.learning_tracker {
                if let Ok(error_id) = tracker.record_error(
                    &error_info.error_type,
                    &error_info.key_message,
                    command,
                    result.exit_code,
                    Some(&result.output),
                ) {
                    // Track this error for resolution detection
                    self.tracked_error = Some(TrackedError {
                        id: error_id,
                        command: command.to_string(),
                        timestamp: Instant::now(),
                    });
                }
            }

            // Track error in session stats
            self.session_stats.record_error(error_info.error_type.name());

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
