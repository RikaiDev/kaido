// Error types and structures for the mentor system
//
// These types represent detected errors and provide context
// for generating educational guidance.

use std::path::PathBuf;

/// Classification of error types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorType {
    /// Command not found (exit code 127)
    CommandNotFound,
    /// Permission denied (EACCES)
    PermissionDenied,
    /// File or directory not found (ENOENT)
    FileNotFound,
    /// Syntax error in command or config
    SyntaxError,
    /// Network connection refused
    ConnectionRefused,
    /// Network connection timeout
    ConnectionTimeout,
    /// Configuration file error
    ConfigurationError,
    /// Resource not found (k8s, docker, etc.)
    ResourceNotFound,
    /// Authentication or authorization failed
    AuthenticationFailed,
    /// Disk full (ENOSPC)
    DiskFull,
    /// Operation timed out
    Timeout,
    /// Out of memory
    OutOfMemory,
    /// Port already in use
    PortInUse,
    /// Invalid argument or option
    InvalidArgument,
    /// Dependency or module not found
    DependencyError,
    /// Git-related error
    GitError,
    /// Docker-related error
    DockerError,
    /// Kubernetes-related error
    KubernetesError,
    /// Database error
    DatabaseError,
    /// Unknown error type
    Unknown,
}

impl ErrorType {
    /// Get a human-readable name for the error type
    pub fn name(&self) -> &'static str {
        match self {
            Self::CommandNotFound => "Command Not Found",
            Self::PermissionDenied => "Permission Denied",
            Self::FileNotFound => "File Not Found",
            Self::SyntaxError => "Syntax Error",
            Self::ConnectionRefused => "Connection Refused",
            Self::ConnectionTimeout => "Connection Timeout",
            Self::ConfigurationError => "Configuration Error",
            Self::ResourceNotFound => "Resource Not Found",
            Self::AuthenticationFailed => "Authentication Failed",
            Self::DiskFull => "Disk Full",
            Self::Timeout => "Timeout",
            Self::OutOfMemory => "Out of Memory",
            Self::PortInUse => "Port Already in Use",
            Self::InvalidArgument => "Invalid Argument",
            Self::DependencyError => "Dependency Error",
            Self::GitError => "Git Error",
            Self::DockerError => "Docker Error",
            Self::KubernetesError => "Kubernetes Error",
            Self::DatabaseError => "Database Error",
            Self::Unknown => "Unknown Error",
        }
    }

    /// Determine error type from exit code
    pub fn from_exit_code(code: i32) -> Self {
        match code {
            1 => Self::Unknown,           // General error
            2 => Self::InvalidArgument,   // Misuse of command
            126 => Self::PermissionDenied, // Permission problem
            127 => Self::CommandNotFound,  // Command not found
            128 => Self::Unknown,          // Invalid exit argument
            130 => Self::Unknown,          // Ctrl+C (not really an error)
            137 => Self::OutOfMemory,      // Often OOM killer (SIGKILL)
            139 => Self::Unknown,          // Segfault
            _ if code > 128 => Self::Unknown, // Killed by signal
            _ => Self::Unknown,
        }
    }
}

/// Location in source code where error occurred
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    /// File path
    pub file: PathBuf,
    /// Line number (1-indexed)
    pub line: Option<u32>,
    /// Column number (1-indexed)
    pub column: Option<u32>,
}

impl SourceLocation {
    /// Create a new source location
    pub fn new(file: impl Into<PathBuf>) -> Self {
        Self {
            file: file.into(),
            line: None,
            column: None,
        }
    }

    /// Set line number
    pub fn with_line(mut self, line: u32) -> Self {
        self.line = Some(line);
        self
    }

    /// Set column number
    pub fn with_column(mut self, column: u32) -> Self {
        self.column = Some(column);
        self
    }

    /// Format as "file:line:column" or "file:line" or just "file"
    pub fn to_string(&self) -> String {
        let mut s = self.file.display().to_string();
        if let Some(line) = self.line {
            s.push(':');
            s.push_str(&line.to_string());
            if let Some(col) = self.column {
                s.push(':');
                s.push_str(&col.to_string());
            }
        }
        s
    }
}

/// Detailed information about a detected error
#[derive(Debug, Clone)]
pub struct ErrorInfo {
    /// Classification of the error
    pub error_type: ErrorType,
    /// Exit code of the command
    pub exit_code: i32,
    /// The most important part of the error message
    pub key_message: String,
    /// The full error output
    pub full_output: String,
    /// The command that was executed
    pub command: String,
    /// Relevant context lines from the output
    pub context_lines: Vec<String>,
    /// Source location if detected (file:line)
    pub source_location: Option<SourceLocation>,
}

impl ErrorInfo {
    /// Create a new ErrorInfo
    pub fn new(
        error_type: ErrorType,
        exit_code: i32,
        key_message: impl Into<String>,
        command: impl Into<String>,
    ) -> Self {
        Self {
            error_type,
            exit_code,
            key_message: key_message.into(),
            full_output: String::new(),
            command: command.into(),
            context_lines: Vec::new(),
            source_location: None,
        }
    }

    /// Set full output
    pub fn with_output(mut self, output: impl Into<String>) -> Self {
        self.full_output = output.into();
        self
    }

    /// Set context lines
    pub fn with_context(mut self, lines: Vec<String>) -> Self {
        self.context_lines = lines;
        self
    }

    /// Set source location
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.source_location = Some(location);
        self
    }

    /// Check if this is a user interruption (Ctrl+C)
    pub fn is_interrupt(&self) -> bool {
        self.exit_code == 130
    }

    /// Check if this is a real error (not interrupt, not success)
    pub fn is_real_error(&self) -> bool {
        self.exit_code != 0 && self.exit_code != 130
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_type_from_exit_code() {
        assert_eq!(ErrorType::from_exit_code(127), ErrorType::CommandNotFound);
        assert_eq!(ErrorType::from_exit_code(126), ErrorType::PermissionDenied);
        assert_eq!(ErrorType::from_exit_code(2), ErrorType::InvalidArgument);
    }

    #[test]
    fn test_error_type_name() {
        assert_eq!(ErrorType::CommandNotFound.name(), "Command Not Found");
        assert_eq!(ErrorType::PermissionDenied.name(), "Permission Denied");
    }

    #[test]
    fn test_source_location() {
        let loc = SourceLocation::new("/etc/nginx/nginx.conf")
            .with_line(42)
            .with_column(10);

        assert_eq!(loc.to_string(), "/etc/nginx/nginx.conf:42:10");
    }

    #[test]
    fn test_source_location_no_column() {
        let loc = SourceLocation::new("/etc/nginx/nginx.conf")
            .with_line(42);

        assert_eq!(loc.to_string(), "/etc/nginx/nginx.conf:42");
    }

    #[test]
    fn test_error_info_creation() {
        let info = ErrorInfo::new(
            ErrorType::CommandNotFound,
            127,
            "command not found: foo",
            "foo --bar",
        );

        assert_eq!(info.exit_code, 127);
        assert_eq!(info.error_type, ErrorType::CommandNotFound);
        assert!(info.is_real_error());
        assert!(!info.is_interrupt());
    }

    #[test]
    fn test_error_info_interrupt() {
        let info = ErrorInfo::new(
            ErrorType::Unknown,
            130,
            "",
            "sleep 100",
        );

        assert!(info.is_interrupt());
        assert!(!info.is_real_error());
    }
}
