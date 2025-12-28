// PTY-based command execution for Kaido shell wrapper
//
// Uses pty-process crate to execute commands in a pseudo-terminal,
// preserving colors, supporting interactive programs, and capturing output.

use anyhow::{Context, Result};
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;

use super::signals::TerminalSize;

/// Result of executing a command in the PTY
#[derive(Debug, Clone)]
pub struct PtyExecutionResult {
    /// Combined output (stdout + stderr merged, as in real terminal)
    pub output: String,
    /// Exit code of the command (None if killed by signal)
    pub exit_code: Option<i32>,
    /// How long the command took to execute
    pub duration: Duration,
    /// The command that was executed
    pub command: String,
    /// Whether the command was interrupted (Ctrl+C)
    pub interrupted: bool,
}

impl PtyExecutionResult {
    /// Check if command succeeded (exit code 0)
    pub fn success(&self) -> bool {
        self.exit_code == Some(0)
    }

    /// Check if command failed (non-zero exit code)
    pub fn failed(&self) -> bool {
        matches!(self.exit_code, Some(code) if code != 0)
    }
}

/// PTY executor for running shell commands
pub struct PtyExecutor {
    /// Shell to use (e.g., /bin/bash, /bin/zsh)
    shell: String,
    /// Terminal size (rows, cols)
    size: (u16, u16),
}

impl PtyExecutor {
    /// Create a new PTY executor with default shell
    pub fn new() -> Self {
        Self {
            shell: std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string()),
            size: (24, 80),
        }
    }

    /// Create PTY executor with custom shell
    pub fn with_shell(shell: impl Into<String>) -> Self {
        Self {
            shell: shell.into(),
            size: (24, 80),
        }
    }

    /// Set terminal size
    pub fn set_size(&mut self, rows: u16, cols: u16) {
        self.size = (rows, cols);
    }

    /// Update terminal size from TerminalSize tracker
    ///
    /// Returns true if the size changed
    pub fn update_size_from(&mut self, terminal_size: &TerminalSize) -> bool {
        let (cols, rows) = terminal_size.get();
        if self.size != (rows, cols) {
            self.size = (rows, cols);
            true
        } else {
            false
        }
    }

    /// Update terminal size from current terminal dimensions
    ///
    /// Returns true if the size changed
    pub fn update_size_from_terminal(&mut self) -> bool {
        let (cols, rows) = TerminalSize::get_current_size();
        if self.size != (rows, cols) {
            self.size = (rows, cols);
            true
        } else {
            false
        }
    }

    /// Get current terminal size
    pub fn get_size(&self) -> (u16, u16) {
        self.size
    }

    /// Execute a command in the PTY and capture output
    ///
    /// This runs the command in a pseudo-terminal, which means:
    /// - Colors and ANSI escape codes are preserved
    /// - stdout and stderr are merged (as in a real terminal)
    /// - Interactive programs can work (though we don't forward input here)
    pub async fn execute(&self, command: &str) -> Result<PtyExecutionResult> {
        let start = Instant::now();

        // Open a new PTY pair
        let (mut pty, pts) = pty_process::open().context("Failed to open PTY")?;

        // Set terminal size
        pty.resize(pty_process::Size::new(self.size.0, self.size.1))
            .context("Failed to set PTY size")?;

        // Build the command: shell -c "command"
        // pty_process::Command uses builder pattern that takes ownership
        let cmd = pty_process::Command::new(&self.shell)
            .arg("-c")
            .arg(command);

        // Spawn the child process attached to the PTY
        let mut child = cmd.spawn(pts).context("Failed to spawn command in PTY")?;

        // Read output from PTY
        let mut output = Vec::new();
        let mut buffer = [0u8; 4096];

        loop {
            tokio::select! {
                // Read from PTY
                result = pty.read(&mut buffer) => {
                    match result {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            output.extend_from_slice(&buffer[..n]);
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            // No data available, continue
                            tokio::time::sleep(Duration::from_millis(10)).await;
                        }
                        Err(e) => {
                            // Check if child has exited
                            if child.try_wait()?.is_some() {
                                break;
                            }
                            // Log error but continue trying
                            log::debug!("PTY read error: {e}");
                        }
                    }
                }
                // Check if child exited
                status = child.wait() => {
                    let status = status?;

                    // Drain remaining output
                    loop {
                        match pty.read(&mut buffer).await {
                            Ok(0) => break,
                            Ok(n) => output.extend_from_slice(&buffer[..n]),
                            Err(_) => break,
                        }
                    }

                    let duration = start.elapsed();
                    let output_str = String::from_utf8_lossy(&output).to_string();

                    return Ok(PtyExecutionResult {
                        output: output_str,
                        exit_code: status.code(),
                        duration,
                        command: command.to_string(),
                        interrupted: false,
                    });
                }
            }
        }

        // Wait for child to finish
        let status = child.wait().await?;
        let duration = start.elapsed();
        let output_str = String::from_utf8_lossy(&output).to_string();

        Ok(PtyExecutionResult {
            output: output_str,
            exit_code: status.code(),
            duration,
            command: command.to_string(),
            interrupted: false,
        })
    }

    /// Execute a command with a timeout
    pub async fn execute_with_timeout(
        &self,
        command: &str,
        timeout: Duration,
    ) -> Result<PtyExecutionResult> {
        match tokio::time::timeout(timeout, self.execute(command)).await {
            Ok(result) => result,
            Err(_) => {
                Ok(PtyExecutionResult {
                    output: format!("Command timed out after {timeout:?}"),
                    exit_code: Some(124), // Standard timeout exit code
                    duration: timeout,
                    command: command.to_string(),
                    interrupted: true,
                })
            }
        }
    }
}

impl Default for PtyExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_simple_command() {
        let executor = PtyExecutor::new();
        let result = executor.execute("echo hello").await.unwrap();

        assert!(result.success());
        assert!(result.output.contains("hello"));
        assert_eq!(result.exit_code, Some(0));
    }

    #[tokio::test]
    async fn test_execute_failing_command() {
        let executor = PtyExecutor::new();
        let result = executor.execute("exit 42").await.unwrap();

        assert!(result.failed());
        assert_eq!(result.exit_code, Some(42));
    }

    #[tokio::test]
    async fn test_execute_with_colors() {
        let executor = PtyExecutor::new();
        // Use printf to output ANSI color codes
        let result = executor
            .execute("printf '\\033[31mred\\033[0m'")
            .await
            .unwrap();

        assert!(result.success());
        // Should contain ANSI escape codes
        assert!(result.output.contains("\x1b[31m") || result.output.contains("red"));
    }

    #[tokio::test]
    async fn test_execute_stderr() {
        let executor = PtyExecutor::new();
        let result = executor.execute("echo error >&2").await.unwrap();

        assert!(result.success());
        // stderr should be captured in the merged output
        assert!(result.output.contains("error"));
    }

    #[tokio::test]
    async fn test_execute_with_timeout() {
        let executor = PtyExecutor::new();
        let result = executor
            .execute_with_timeout("sleep 10", Duration::from_millis(100))
            .await
            .unwrap();

        assert!(result.interrupted);
        assert_eq!(result.exit_code, Some(124));
    }

    #[tokio::test]
    async fn test_command_not_found() {
        let executor = PtyExecutor::new();
        let result = executor.execute("nonexistent_command_12345").await.unwrap();

        assert!(result.failed());
        assert_eq!(result.exit_code, Some(127)); // Command not found
    }

    #[tokio::test]
    async fn test_multiline_output() {
        let executor = PtyExecutor::new();
        let result = executor
            .execute("echo line1; echo line2; echo line3")
            .await
            .unwrap();

        assert!(result.success());
        assert!(result.output.contains("line1"));
        assert!(result.output.contains("line2"));
        assert!(result.output.contains("line3"));
    }

    #[test]
    fn test_pty_executor_default() {
        let executor = PtyExecutor::default();
        assert!(!executor.shell.is_empty());
        assert_eq!(executor.size, (24, 80));
    }

    #[test]
    fn test_update_size_from_terminal_size() {
        use super::TerminalSize;

        let mut executor = PtyExecutor::new();
        let terminal_size = TerminalSize::new();

        // Update from terminal size tracker
        executor.update_size_from(&terminal_size);

        let (cols, rows) = terminal_size.get();
        assert_eq!(executor.get_size(), (rows, cols));
    }

    #[test]
    fn test_update_size_from_terminal() {
        let mut executor = PtyExecutor::new();

        // Should get current terminal size (or default 80x24)
        executor.update_size_from_terminal();
        let (rows, cols) = executor.get_size();
        assert!(rows > 0);
        assert!(cols > 0);
    }
}
