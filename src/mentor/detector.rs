// Error detection engine for the mentor system
//
// Analyzes command execution results to detect errors and
// extract useful information for educational guidance.

use regex::Regex;

use super::types::{ErrorInfo, ErrorType, SourceLocation};
use crate::shell::PtyExecutionResult;

/// Pattern for detecting specific error types
#[derive(Debug)]
struct ErrorPattern {
    /// Regex pattern to match
    regex: Regex,
    /// Error type this pattern indicates
    error_type: ErrorType,
    /// Group index for extracting key message (0 = whole match)
    key_group: usize,
}

/// Error detection engine
pub struct ErrorDetector {
    /// Patterns for detecting error types
    patterns: Vec<ErrorPattern>,
    /// Regex for extracting file:line:column references
    location_regex: Regex,
}

impl ErrorDetector {
    /// Create a new error detector with built-in patterns
    pub fn new() -> Self {
        Self {
            patterns: Self::build_patterns(),
            location_regex: Regex::new(
                r"(?:^|[:\s])(/[^\s:]+):(\d+)(?::(\d+))?"
            ).unwrap(),
        }
    }

    /// Build the default error patterns
    fn build_patterns() -> Vec<ErrorPattern> {
        vec![
            // Command not found
            ErrorPattern {
                regex: Regex::new(r"(?i)(?:command not found|not found):\s*(\S+)").unwrap(),
                error_type: ErrorType::CommandNotFound,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r"(?i)(\S+):\s*command not found").unwrap(),
                error_type: ErrorType::CommandNotFound,
                key_group: 0,
            },
            // Permission denied
            ErrorPattern {
                regex: Regex::new(r"(?i)permission denied").unwrap(),
                error_type: ErrorType::PermissionDenied,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r"(?i)EACCES").unwrap(),
                error_type: ErrorType::PermissionDenied,
                key_group: 0,
            },
            // File not found
            ErrorPattern {
                regex: Regex::new(r"(?i)no such file or directory").unwrap(),
                error_type: ErrorType::FileNotFound,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r"(?i)ENOENT").unwrap(),
                error_type: ErrorType::FileNotFound,
                key_group: 0,
            },
            // Dependency errors (npm, pip, cargo, etc.) - must be before generic "cannot find"
            ErrorPattern {
                regex: Regex::new(r"(?i)cannot find module").unwrap(),
                error_type: ErrorType::DependencyError,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r"(?i)(?:module|package|dependency) .+ not found").unwrap(),
                error_type: ErrorType::DependencyError,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r"(?i)no matching version").unwrap(),
                error_type: ErrorType::DependencyError,
                key_group: 0,
            },
            // Generic "cannot find" for files (after dependency patterns)
            ErrorPattern {
                regex: Regex::new(r#"(?i)cannot (?:open|access|stat)\s+['"]?([^'"]+)['"]?"#).unwrap(),
                error_type: ErrorType::FileNotFound,
                key_group: 0,
            },
            // Connection refused
            ErrorPattern {
                regex: Regex::new(r"(?i)connection refused").unwrap(),
                error_type: ErrorType::ConnectionRefused,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r"(?i)ECONNREFUSED").unwrap(),
                error_type: ErrorType::ConnectionRefused,
                key_group: 0,
            },
            // Connection timeout
            ErrorPattern {
                regex: Regex::new(r"(?i)(?:connection|operation) timed? ?out").unwrap(),
                error_type: ErrorType::ConnectionTimeout,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r"(?i)ETIMEDOUT").unwrap(),
                error_type: ErrorType::ConnectionTimeout,
                key_group: 0,
            },
            // Syntax errors
            ErrorPattern {
                regex: Regex::new(r"(?i)syntax error").unwrap(),
                error_type: ErrorType::SyntaxError,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r"(?i)unexpected token").unwrap(),
                error_type: ErrorType::SyntaxError,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r"(?i)parse error").unwrap(),
                error_type: ErrorType::SyntaxError,
                key_group: 0,
            },
            // Nginx specific
            ErrorPattern {
                regex: Regex::new(r"nginx:\s*\[emerg\]\s*(.+)").unwrap(),
                error_type: ErrorType::ConfigurationError,
                key_group: 1,
            },
            ErrorPattern {
                regex: Regex::new(r#"(?i)unknown directive\s+['"]?(\w+)['"]?"#).unwrap(),
                error_type: ErrorType::ConfigurationError,
                key_group: 0,
            },
            // Docker specific
            ErrorPattern {
                regex: Regex::new(r"(?i)(?:unable to find|cannot find) image").unwrap(),
                error_type: ErrorType::DockerError,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r"(?i)error response from daemon:\s*(.+)").unwrap(),
                error_type: ErrorType::DockerError,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r"(?i)container .+ is not running").unwrap(),
                error_type: ErrorType::DockerError,
                key_group: 0,
            },
            // Kubernetes specific
            ErrorPattern {
                regex: Regex::new(r"(?i)error from server \((\w+)\):\s*(.+)").unwrap(),
                error_type: ErrorType::KubernetesError,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r"(?i)the server doesn't have a resource type").unwrap(),
                error_type: ErrorType::KubernetesError,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r#"(?i)(?:pods?|deployments?|services?|configmaps?)\s+['\"]?(\S+)['\"]?\s+not found"#).unwrap(),
                error_type: ErrorType::ResourceNotFound,
                key_group: 0,
            },
            // Git specific
            ErrorPattern {
                regex: Regex::new(r"(?i)fatal:\s*(.+)").unwrap(),
                error_type: ErrorType::GitError,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r"(?i)not a git repository").unwrap(),
                error_type: ErrorType::GitError,
                key_group: 0,
            },
            // Authentication
            ErrorPattern {
                regex: Regex::new(r"(?i)(?:authentication|auth) (?:failed|error|denied)").unwrap(),
                error_type: ErrorType::AuthenticationFailed,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r"(?i)unauthorized").unwrap(),
                error_type: ErrorType::AuthenticationFailed,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r"(?i)access denied").unwrap(),
                error_type: ErrorType::AuthenticationFailed,
                key_group: 0,
            },
            // Disk full
            ErrorPattern {
                regex: Regex::new(r"(?i)no space left on device").unwrap(),
                error_type: ErrorType::DiskFull,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r"(?i)ENOSPC").unwrap(),
                error_type: ErrorType::DiskFull,
                key_group: 0,
            },
            // Out of memory
            ErrorPattern {
                regex: Regex::new(r"(?i)out of memory").unwrap(),
                error_type: ErrorType::OutOfMemory,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r"(?i)cannot allocate memory").unwrap(),
                error_type: ErrorType::OutOfMemory,
                key_group: 0,
            },
            // Port in use
            ErrorPattern {
                regex: Regex::new(r"(?i)address already in use").unwrap(),
                error_type: ErrorType::PortInUse,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r"(?i)EADDRINUSE").unwrap(),
                error_type: ErrorType::PortInUse,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r"(?i)port \d+ (?:is )?(?:already )?in use").unwrap(),
                error_type: ErrorType::PortInUse,
                key_group: 0,
            },
            // Invalid arguments
            ErrorPattern {
                regex: Regex::new(r"(?i)invalid (?:option|argument|flag)").unwrap(),
                error_type: ErrorType::InvalidArgument,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r"(?i)unrecognized (?:option|argument|flag)").unwrap(),
                error_type: ErrorType::InvalidArgument,
                key_group: 0,
            },
            // Database errors
            ErrorPattern {
                regex: Regex::new(r"(?i)(?:mysql|postgres|sqlite).*error").unwrap(),
                error_type: ErrorType::DatabaseError,
                key_group: 0,
            },
            ErrorPattern {
                regex: Regex::new(r"(?i)database .+ does not exist").unwrap(),
                error_type: ErrorType::DatabaseError,
                key_group: 0,
            },
        ]
    }

    /// Analyze a command execution result for errors
    pub fn analyze(&self, result: &PtyExecutionResult) -> Option<ErrorInfo> {
        // Don't analyze successful commands
        if result.success() {
            return None;
        }

        // Don't analyze interrupts (Ctrl+C)
        if result.exit_code == Some(130) {
            return None;
        }

        let exit_code = result.exit_code.unwrap_or(1);
        let output = &result.output;

        // Detect error type from patterns
        let (error_type, key_message) = self.detect_error_type(output, exit_code);

        // Extract source location if present
        let source_location = self.extract_source_location(output);

        // Extract context lines
        let context_lines = self.extract_context_lines(output);

        Some(ErrorInfo {
            error_type,
            exit_code,
            key_message,
            full_output: output.clone(),
            command: result.command.clone(),
            context_lines,
            source_location,
        })
    }

    /// Detect error type and extract key message from output
    fn detect_error_type(&self, output: &str, exit_code: i32) -> (ErrorType, String) {
        // Try pattern matching first
        for pattern in &self.patterns {
            if let Some(captures) = pattern.regex.captures(output) {
                let key_message = if pattern.key_group > 0 {
                    captures.get(pattern.key_group)
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_else(|| captures.get(0).unwrap().as_str().to_string())
                } else {
                    captures.get(0).unwrap().as_str().to_string()
                };
                return (pattern.error_type.clone(), key_message);
            }
        }

        // Fall back to exit code
        let error_type = ErrorType::from_exit_code(exit_code);
        let key_message = self.extract_first_error_line(output);

        (error_type, key_message)
    }

    /// Extract the first meaningful error line from output
    fn extract_first_error_line(&self, output: &str) -> String {
        // Look for lines containing common error indicators
        let error_indicators = [
            "error", "Error", "ERROR",
            "failed", "Failed", "FAILED",
            "fatal", "Fatal", "FATAL",
            "cannot", "Cannot", "CANNOT",
            "unable", "Unable", "UNABLE",
            "denied", "Denied", "DENIED",
        ];

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Check for error indicators
            for indicator in &error_indicators {
                if line.contains(indicator) {
                    return line.to_string();
                }
            }
        }

        // Just return the first non-empty line
        output.lines()
            .map(|l| l.trim())
            .find(|l| !l.is_empty())
            .unwrap_or("Unknown error")
            .to_string()
    }

    /// Extract file:line:column references from output
    fn extract_source_location(&self, output: &str) -> Option<SourceLocation> {
        // Try the general pattern first
        if let Some(captures) = self.location_regex.captures(output) {
            let file = captures.get(1)?.as_str();
            let line = captures.get(2)?.as_str().parse().ok();
            let column = captures.get(3).and_then(|m| m.as_str().parse().ok());

            let mut loc = SourceLocation::new(file);
            if let Some(l) = line {
                loc = loc.with_line(l);
            }
            if let Some(c) = column {
                loc = loc.with_column(c);
            }
            return Some(loc);
        }

        None
    }

    /// Extract context lines around the error
    fn extract_context_lines(&self, output: &str) -> Vec<String> {
        let lines: Vec<&str> = output.lines().collect();
        let mut context = Vec::new();

        // Find the most relevant lines (containing error keywords)
        for (i, line) in lines.iter().enumerate() {
            let lower = line.to_lowercase();
            if lower.contains("error") || lower.contains("failed") || lower.contains("fatal") {
                // Add this line and some context
                let start = i.saturating_sub(1);
                let end = (i + 2).min(lines.len());

                for j in start..end {
                    let trimmed = lines[j].trim();
                    if !trimmed.is_empty() && !context.contains(&trimmed.to_string()) {
                        context.push(trimmed.to_string());
                    }
                }
            }
        }

        // Limit to 5 lines
        context.truncate(5);
        context
    }
}

impl Default for ErrorDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_result(output: &str, exit_code: i32) -> PtyExecutionResult {
        PtyExecutionResult {
            output: output.to_string(),
            exit_code: Some(exit_code),
            duration: std::time::Duration::from_secs(0),
            command: "test command".to_string(),
            interrupted: false,
        }
    }

    #[test]
    fn test_detect_command_not_found() {
        let detector = ErrorDetector::new();
        let result = make_result("bash: foo: command not found", 127);

        let error = detector.analyze(&result).unwrap();
        assert_eq!(error.error_type, ErrorType::CommandNotFound);
        assert!(error.key_message.contains("command not found"));
    }

    #[test]
    fn test_detect_permission_denied() {
        let detector = ErrorDetector::new();
        let result = make_result("cat: /etc/shadow: Permission denied", 1);

        let error = detector.analyze(&result).unwrap();
        assert_eq!(error.error_type, ErrorType::PermissionDenied);
    }

    #[test]
    fn test_detect_file_not_found() {
        let detector = ErrorDetector::new();
        let result = make_result("cat: /nonexistent: No such file or directory", 1);

        let error = detector.analyze(&result).unwrap();
        assert_eq!(error.error_type, ErrorType::FileNotFound);
    }

    #[test]
    fn test_detect_connection_refused() {
        let detector = ErrorDetector::new();
        let result = make_result("curl: (7) Failed to connect: Connection refused", 7);

        let error = detector.analyze(&result).unwrap();
        assert_eq!(error.error_type, ErrorType::ConnectionRefused);
    }

    #[test]
    fn test_detect_nginx_config_error() {
        let detector = ErrorDetector::new();
        let result = make_result(
            "nginx: [emerg] unknown directive \"proxy_passs\" in /etc/nginx/nginx.conf:42",
            1,
        );

        let error = detector.analyze(&result).unwrap();
        assert_eq!(error.error_type, ErrorType::ConfigurationError);
    }

    #[test]
    fn test_detect_docker_error() {
        let detector = ErrorDetector::new();
        let result = make_result(
            "Unable to find image 'nonexistent:latest' locally",
            1,
        );

        let error = detector.analyze(&result).unwrap();
        assert_eq!(error.error_type, ErrorType::DockerError);
    }

    #[test]
    fn test_detect_kubernetes_error() {
        let detector = ErrorDetector::new();
        let result = make_result(
            "Error from server (NotFound): pods \"my-pod\" not found",
            1,
        );

        let error = detector.analyze(&result).unwrap();
        assert_eq!(error.error_type, ErrorType::KubernetesError);
    }

    #[test]
    fn test_detect_port_in_use() {
        let detector = ErrorDetector::new();
        let result = make_result("Error: listen EADDRINUSE: address already in use :::3000", 1);

        let error = detector.analyze(&result).unwrap();
        assert_eq!(error.error_type, ErrorType::PortInUse);
    }

    #[test]
    fn test_extract_source_location() {
        let detector = ErrorDetector::new();
        let location = detector.extract_source_location(
            "Error in /etc/nginx/nginx.conf:42:10 - unknown directive"
        );

        assert!(location.is_some());
        let loc = location.unwrap();
        assert_eq!(loc.file, PathBuf::from("/etc/nginx/nginx.conf"));
        assert_eq!(loc.line, Some(42));
        assert_eq!(loc.column, Some(10));
    }

    #[test]
    fn test_no_error_on_success() {
        let detector = ErrorDetector::new();
        let result = PtyExecutionResult {
            output: "success".to_string(),
            exit_code: Some(0),
            duration: std::time::Duration::from_secs(0),
            command: "echo success".to_string(),
            interrupted: false,
        };

        assert!(detector.analyze(&result).is_none());
    }

    #[test]
    fn test_no_error_on_interrupt() {
        let detector = ErrorDetector::new();
        let result = PtyExecutionResult {
            output: "^C".to_string(),
            exit_code: Some(130),
            duration: std::time::Duration::from_secs(0),
            command: "sleep 100".to_string(),
            interrupted: true,
        };

        assert!(detector.analyze(&result).is_none());
    }

    #[test]
    fn test_git_error() {
        let detector = ErrorDetector::new();
        let result = make_result("fatal: not a git repository", 128);

        let error = detector.analyze(&result).unwrap();
        assert_eq!(error.error_type, ErrorType::GitError);
    }

    #[test]
    fn test_dependency_error() {
        let detector = ErrorDetector::new();
        let result = make_result("Error: Cannot find module 'express'", 1);

        let error = detector.analyze(&result).unwrap();
        assert_eq!(error.error_type, ErrorType::DependencyError);
    }
}
