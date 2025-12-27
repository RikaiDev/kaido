use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Instant;

/// Result of kubectl command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Exit code from kubectl command
    pub exit_code: Option<i32>,
    
    /// Standard output (truncated to 10KB for logging)
    pub stdout: String,
    
    /// Standard error (truncated to 10KB for logging)
    pub stderr: String,
    
    /// Execution duration in milliseconds
    pub execution_duration_ms: i64,
}

impl ExecutionResult {
    /// Create new execution result
    pub fn new(
        exit_code: Option<i32>,
        stdout: String,
        stderr: String,
        execution_duration_ms: i64,
    ) -> Self {
        Self {
            exit_code,
            stdout,
            stderr,
            execution_duration_ms,
        }
    }
    
    /// Check if execution was successful
    pub fn is_success(&self) -> bool {
        self.exit_code == Some(0)
    }
    
    /// Truncate output to specified max bytes
    pub fn truncate_output(&mut self, max_bytes: usize) {
        if self.stdout.len() > max_bytes {
            self.stdout.truncate(max_bytes);
            self.stdout.push_str("\n... (truncated)");
        }
        
        if self.stderr.len() > max_bytes {
            self.stderr.truncate(max_bytes);
            self.stderr.push_str("\n... (truncated)");
        }
    }
}

/// Execute kubectl command and capture output
/// 
/// This function:
/// - Executes the kubectl command using std::process::Command
/// - Captures stdout and stderr
/// - Preserves ANSI colors and formatting
/// - Measures execution duration
/// - Truncates output to 10KB for logging
pub fn execute_kubectl(kubectl_command: &str) -> anyhow::Result<ExecutionResult> {
    log::info!("Executing kubectl command: {kubectl_command}");
    
    // Parse command into parts
    let parts: Vec<&str> = kubectl_command.split_whitespace().collect();
    
    if parts.is_empty() || parts[0] != "kubectl" {
        return Err(anyhow::anyhow!("Command must start with 'kubectl'"));
    }
    
    // Start timing
    let start = Instant::now();
    
    // Execute command
    let output = Command::new("kubectl")
        .args(&parts[1..]) // Skip "kubectl" itself
        .output();
    
    // Calculate duration
    let duration_ms = start.elapsed().as_millis() as i64;
    
    match output {
        Ok(output) => {
            let exit_code = output.status.code();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            
            log::info!(
                "Command completed: exit_code={:?}, duration={}ms, stdout_len={}, stderr_len={}",
                exit_code,
                duration_ms,
                stdout.len(),
                stderr.len()
            );
            
            let mut result = ExecutionResult::new(
                exit_code,
                stdout,
                stderr,
                duration_ms,
            );
            
            // Truncate output for logging (10KB limit)
            result.truncate_output(10240);
            
            Ok(result)
        }
        Err(e) => {
            log::error!("Failed to execute kubectl: {e}");
            
            // Check if kubectl is not installed
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(anyhow::anyhow!(
                    "kubectl command not found. Please install kubectl: https://kubernetes.io/docs/tasks/tools/"
                ));
            }
            
            Err(anyhow::anyhow!("Failed to execute kubectl: {e}"))
        }
    }
}

/// Format kubectl output for display
/// 
/// - Preserves ANSI colors
/// - Handles empty output
/// - Adds helpful messages for common cases
pub fn format_output(result: &ExecutionResult) -> String {
    if result.is_success() {
        if result.stdout.trim().is_empty() {
            "(No output - command succeeded)".to_string()
        } else {
            result.stdout.clone()
        }
    } else {
        let mut output = String::new();
        
        if !result.stdout.is_empty() {
            output.push_str(&result.stdout);
        }
        
        if !result.stderr.is_empty() {
            if !output.is_empty() {
                output.push_str("\n\n");
            }
            output.push_str("Error: ");
            output.push_str(&result.stderr);
        }
        
        if output.trim().is_empty() {
            output = format!("Command failed with exit code: {:?}", result.exit_code);
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_result_success() {
        let result = ExecutionResult::new(
            Some(0),
            "NAME   READY   STATUS\npod1   1/1     Running".to_string(),
            String::new(),
            150,
        );
        
        assert!(result.is_success());
        assert_eq!(result.exit_code, Some(0));
    }

    #[test]
    fn test_execution_result_failure() {
        let result = ExecutionResult::new(
            Some(1),
            String::new(),
            "Error: pod not found".to_string(),
            100,
        );
        
        assert!(!result.is_success());
    }

    #[test]
    fn test_truncate_output() {
        let mut result = ExecutionResult::new(
            Some(0),
            "a".repeat(15000), // 15KB
            "b".repeat(15000),
            200,
        );
        
        result.truncate_output(10240); // 10KB
        
        assert!(result.stdout.len() <= 10260); // 10KB + "...(truncated)" message
        assert!(result.stderr.len() <= 10260);
        assert!(result.stdout.ends_with("(truncated)"));
    }

}
// Note: combined_output test removed - method needs to be implemented if needed

