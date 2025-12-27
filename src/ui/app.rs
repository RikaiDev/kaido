use crate::ui::modal::ModalDialog;
use crate::ui::spinner::SPINNER_FRAMES;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Stdout};

/// Application state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Normal,      // Regular input mode
    ModalActive, // Modal is blocking, waiting for button selection
    Executing,   // Command running, UI shows progress
}

/// Thinking stage for AI processing visualization
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThinkingStage {
    Idle,
    AnalyzingInput,
    GeneratingCommands,
    ValidatingOutput,
    Complete,
}

/// Main application state container for the TUI
#[derive(Clone)]
pub struct KaidoApp {
    pub input: String,
    pub history: Vec<String>,
    pub output: String,
    pub ai_panel_toggle: bool,
    pub ai_thinking: bool,
    pub ai_output: String,
    pub spinner_index: usize,
    pub modal: Option<ModalDialog>,
    pub state: AppState,
    pub thinking_stage: ThinkingStage,
    pub thinking_start_time: Option<std::time::Instant>,
}

impl KaidoApp {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            history: Vec::new(),
            output: String::new(),
            ai_panel_toggle: false,
            ai_thinking: false,
            ai_output: String::new(),
            spinner_index: 0,
            modal: None,
            state: AppState::Normal,
            thinking_stage: ThinkingStage::Idle,
            thinking_start_time: None,
        }
    }

    pub fn toggle_ai_panel(&mut self) {
        self.ai_panel_toggle = !self.ai_panel_toggle;
    }

    pub fn next_spinner_frame(&mut self) {
        self.spinner_index = (self.spinner_index + 1) % SPINNER_FRAMES.len();
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
    }

    pub fn add_to_history(&mut self, cmd: String) {
        self.history.push(cmd);
        // Enforce max 1000 entries
        if self.history.len() > 1000 {
            self.history.remove(0);
        }
    }
}

/// RAII guard for terminal raw mode
/// Automatically restores terminal on drop
pub struct TerminalGuard {
    stdout_handle: Stdout,  // RAII: Terminal restored via Drop trait
}

impl TerminalGuard {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        Ok(Self { stdout_handle: stdout })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Always restore terminal, even on panic
        let _ = execute!(self.stdout_handle, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_new() {
        let app = KaidoApp::new();
        assert_eq!(app.input, "");
        assert_eq!(app.history.len(), 0);
        assert_eq!(app.ai_panel_toggle, false);
        assert_eq!(app.ai_thinking, false);
        assert_eq!(app.spinner_index, 0);
        assert!(app.modal.is_none());
        assert_eq!(app.state, AppState::Normal);
        assert_eq!(app.thinking_stage, ThinkingStage::Idle);
        assert!(app.thinking_start_time.is_none());
    }

    #[test]
    fn test_toggle_ai_panel() {
        let mut app = KaidoApp::new();
        assert_eq!(app.ai_panel_toggle, false);
        app.toggle_ai_panel();
        assert_eq!(app.ai_panel_toggle, true);
        app.toggle_ai_panel();
        assert_eq!(app.ai_panel_toggle, false);
    }

    #[test]
    fn test_next_spinner_frame() {
        let mut app = KaidoApp::new();
        assert_eq!(app.spinner_index, 0);
        app.next_spinner_frame();
        assert_eq!(app.spinner_index, 1);
        // Test cycling
        for _ in 0..20 {
            app.next_spinner_frame();
        }
        assert!(app.spinner_index < SPINNER_FRAMES.len());
    }

    #[test]
    fn test_history_max_size() {
        let mut app = KaidoApp::new();
        // Add 1100 items
        for i in 0..1100 {
            app.add_to_history(format!("command {}", i));
        }
        // Should cap at 1000
        assert_eq!(app.history.len(), 1000);
        // Should have removed oldest (command 0-99)
        assert!(app.history[0].contains("100"));
    }
}

