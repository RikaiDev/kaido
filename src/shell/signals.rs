// Signal handling for Kaido shell
//
// Provides proper signal handling for shell behavior:
// - SIGWINCH: Terminal resize
// - SIGINT: Interrupt (Ctrl+C) - handled by rustyline
// - SIGTSTP: Suspend (Ctrl+Z) - not yet implemented
//
// Note: Most signal handling is done through rustyline's ReadlineError
// for Ctrl+C and Ctrl+D. This module focuses on SIGWINCH for terminal resize.

use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::Arc;

/// Terminal size tracking with atomic updates
#[derive(Debug, Clone)]
pub struct TerminalSize {
    rows: Arc<AtomicU16>,
    cols: Arc<AtomicU16>,
    changed: Arc<AtomicBool>,
}

impl TerminalSize {
    /// Create a new terminal size tracker with current dimensions
    pub fn new() -> Self {
        let (cols, rows) = Self::get_current_size();
        Self {
            rows: Arc::new(AtomicU16::new(rows)),
            cols: Arc::new(AtomicU16::new(cols)),
            changed: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get the current terminal size from the system
    pub fn get_current_size() -> (u16, u16) {
        terminal_size::terminal_size()
            .map(|(w, h)| (w.0, h.0))
            .unwrap_or((80, 24))
    }

    /// Get the tracked terminal size
    pub fn get(&self) -> (u16, u16) {
        (
            self.cols.load(Ordering::Relaxed),
            self.rows.load(Ordering::Relaxed),
        )
    }

    /// Get rows
    pub fn rows(&self) -> u16 {
        self.rows.load(Ordering::Relaxed)
    }

    /// Get columns
    pub fn cols(&self) -> u16 {
        self.cols.load(Ordering::Relaxed)
    }

    /// Update the tracked size from current terminal dimensions
    /// Returns true if the size changed
    pub fn update(&self) -> bool {
        let (new_cols, new_rows) = Self::get_current_size();
        let old_cols = self.cols.swap(new_cols, Ordering::Relaxed);
        let old_rows = self.rows.swap(new_rows, Ordering::Relaxed);

        let changed = old_cols != new_cols || old_rows != new_rows;
        if changed {
            self.changed.store(true, Ordering::Relaxed);
        }
        changed
    }

    /// Check if size has changed since last check (and reset the flag)
    pub fn take_changed(&self) -> bool {
        self.changed.swap(false, Ordering::Relaxed)
    }

    /// Check if size has changed without resetting the flag
    pub fn has_changed(&self) -> bool {
        self.changed.load(Ordering::Relaxed)
    }
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self::new()
    }
}

/// Signal handler for the shell
///
/// Currently handles:
/// - Terminal resize (SIGWINCH)
///
/// Note: SIGINT and EOF are handled by rustyline directly.
pub struct SignalHandler {
    terminal_size: TerminalSize,
    #[cfg(unix)]
    resize_notify: Option<tokio::sync::watch::Sender<()>>,
}

impl SignalHandler {
    /// Create a new signal handler
    pub fn new() -> Self {
        Self {
            terminal_size: TerminalSize::new(),
            #[cfg(unix)]
            resize_notify: None,
        }
    }

    /// Get the terminal size tracker
    pub fn terminal_size(&self) -> &TerminalSize {
        &self.terminal_size
    }

    /// Setup signal handlers (call once at startup)
    #[cfg(unix)]
    pub fn setup(&mut self) -> anyhow::Result<()> {
        use tokio::sync::watch;

        let (tx, _rx) = watch::channel(());
        self.resize_notify = Some(tx);

        // Spawn a task to handle SIGWINCH
        let terminal_size = self.terminal_size.clone();
        let tx_clone = self.resize_notify.as_ref().unwrap().clone();

        tokio::spawn(async move {
            let mut sigwinch = match tokio::signal::unix::signal(
                tokio::signal::unix::SignalKind::window_change(),
            ) {
                Ok(s) => s,
                Err(e) => {
                    log::warn!("Failed to setup SIGWINCH handler: {}", e);
                    return;
                }
            };

            loop {
                sigwinch.recv().await;
                if terminal_size.update() {
                    let _ = tx_clone.send(());
                    log::debug!(
                        "Terminal resized to {}x{}",
                        terminal_size.cols(),
                        terminal_size.rows()
                    );
                }
            }
        });

        Ok(())
    }

    /// Setup signal handlers (no-op on non-Unix)
    #[cfg(not(unix))]
    pub fn setup(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Subscribe to resize notifications
    #[cfg(unix)]
    pub fn subscribe_resize(&self) -> Option<tokio::sync::watch::Receiver<()>> {
        self.resize_notify.as_ref().map(|tx| tx.subscribe())
    }

    /// Subscribe to resize notifications (no-op on non-Unix)
    #[cfg(not(unix))]
    pub fn subscribe_resize(&self) -> Option<tokio::sync::watch::Receiver<()>> {
        None
    }

    /// Check for terminal resize and return new size if changed
    pub fn check_resize(&self) -> Option<(u16, u16)> {
        if self.terminal_size.update() {
            Some(self.terminal_size.get())
        } else {
            None
        }
    }
}

impl Default for SignalHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_size_creation() {
        let size = TerminalSize::new();
        let (cols, rows) = size.get();
        // Should have reasonable dimensions
        assert!(cols > 0);
        assert!(rows > 0);
    }

    #[test]
    fn test_terminal_size_get_current() {
        let (cols, rows) = TerminalSize::get_current_size();
        // Should return either actual size or default 80x24
        assert!(cols >= 80 || cols > 0);
        assert!(rows >= 24 || rows > 0);
    }

    #[test]
    fn test_terminal_size_changed_flag() {
        let size = TerminalSize::new();

        // Initially not changed
        assert!(!size.has_changed());

        // take_changed should return false and keep it false
        assert!(!size.take_changed());
        assert!(!size.has_changed());
    }

    #[test]
    fn test_signal_handler_creation() {
        let handler = SignalHandler::new();
        let size = handler.terminal_size().get();
        assert!(size.0 > 0);
        assert!(size.1 > 0);
    }

    #[test]
    fn test_signal_handler_check_resize() {
        let handler = SignalHandler::new();
        // First check might return None or Some depending on timing
        let _ = handler.check_resize();
        // Second immediate check should return None (no change)
        assert!(handler.check_resize().is_none());
    }
}
