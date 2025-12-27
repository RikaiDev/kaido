// Audit logger implementation for kubectl command history
use anyhow::Result;
use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::kubectl::{ExecutionResult, RiskLevel};

/// Maximum length for stdout/stderr (10KB)
const MAX_OUTPUT_LENGTH: usize = 10 * 1024;

/// User action type for audit log
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserAction {
    /// Command was executed
    Executed,
    /// Command was cancelled by user
    Cancelled,
    /// Command was edited before execution
    Edited,
}

impl UserAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserAction::Executed => "EXECUTED",
            UserAction::Cancelled => "CANCELLED",
            UserAction::Edited => "EDITED",
        }
    }
}

/// Audit log entry
#[derive(Debug, Clone)]
pub struct AuditLogEntry {
    /// Unix timestamp
    pub timestamp: i64,
    /// System username
    pub user_id: String,
    /// Original natural language input
    pub natural_language_input: String,
    /// Translated kubectl command
    pub kubectl_command: String,
    /// Original AI-generated command (before user edit), None if not edited
    pub original_command: Option<String>,
    /// AI confidence score (0-100), None if direct kubectl input
    pub confidence_score: Option<u8>,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Environment (context name)
    pub environment: String,
    /// Cluster name
    pub cluster: String,
    /// Namespace
    pub namespace: Option<String>,
    /// Exit code (None if cancelled)
    pub exit_code: Option<i32>,
    /// stdout (truncated to 10KB)
    pub stdout: Option<String>,
    /// stderr (truncated to 10KB)
    pub stderr: Option<String>,
    /// Execution duration in milliseconds
    pub execution_duration_ms: Option<i64>,
    /// User action
    pub user_action: UserAction,
}

/// Audit logger for recording kubectl commands
#[derive(Clone)]
pub struct AuditLogger {
    conn: Arc<Mutex<Connection>>,
}

impl AuditLogger {
    /// Create new audit logger
    /// 
    /// Initializes database connection, applies schema, and runs retention policy
    pub fn new(database_path: &str) -> Result<Self> {
        // Open connection
        let conn = Connection::open(database_path)?;
        
        // Set PRAGMA for better performance
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA foreign_keys=ON;
             PRAGMA temp_store=MEMORY;"
        )?;
        
        // Initialize schema (from schema.rs)
        crate::audit::schema::initialize_schema(&conn)?;
        
        // Clean old entries (retention policy: 90 days)
        Self::clean_old_entries_internal(&conn, 90)?;
        
        Ok(Self { 
            conn: Arc::new(Mutex::new(conn))
        })
    }
    
    /// Log a command execution
    /// 
    /// This function is non-blocking - it will log errors but not fail the command execution
    pub fn log_execution(&self, entry: AuditLogEntry) -> Result<i64> {
        // Truncate stdout/stderr to 10KB
        let stdout = entry.stdout.as_ref().map(|s| truncate_output(s));
        let stderr = entry.stderr.as_ref().map(|s| truncate_output(s));
        
        // Insert into database        
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO audit_log (
                timestamp,
                user_id,
                natural_language_input,
                kubectl_command,
                original_command,
                confidence_score,
                risk_level,
                environment,
                cluster,
                namespace,
                exit_code,
                stdout,
                stderr,
                execution_duration_ms,
                user_action
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                entry.timestamp,
                entry.user_id,
                entry.natural_language_input,
                entry.kubectl_command,
                entry.original_command,
                entry.confidence_score,
                entry.risk_level.as_str(),
                entry.environment,
                entry.cluster,
                entry.namespace,
                entry.exit_code,
                stdout,
                stderr,
                entry.execution_duration_ms,
                entry.user_action.as_str(),
            ],
        )?;
        
        Ok(conn.last_insert_rowid())
    }
    
    /// Clean entries older than specified days
    /// 
    /// This is called on startup to enforce retention policy
    pub fn clean_old_entries(&self, days: u32) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        Self::clean_old_entries_internal(&conn, days)
    }
    
    fn clean_old_entries_internal(conn: &Connection, days: u32) -> Result<usize> {
        let cutoff_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() as i64
            - (days as i64 * 24 * 60 * 60);
        
        let deleted = conn.execute(
            "DELETE FROM audit_log WHERE timestamp < ?",
            params![cutoff_timestamp],
        )?;
        
        if deleted > 0 {
            log::info!("Cleaned {deleted} old audit log entries (older than {days} days)");
        }
        
        Ok(deleted)
    }
    
    /// Get current Unix timestamp
    pub fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time is before Unix epoch")
            .as_secs() as i64
    }
    
    /// Get current system username
    pub fn current_user() -> String {
        users::get_current_username()
            .and_then(|name| name.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string())
    }
}

/// Truncate output to maximum length (10KB)
fn truncate_output(output: &str) -> String {
    if output.len() <= MAX_OUTPUT_LENGTH {
        output.to_string()
    } else {
        let mut truncated = String::with_capacity(MAX_OUTPUT_LENGTH + 20);
        truncated.push_str(&output[..MAX_OUTPUT_LENGTH]);
        truncated.push_str("\n\n[OUTPUT TRUNCATED]");
        truncated
    }
}

/// Context for audit log entry creation
pub struct AuditContext<'a> {
    pub natural_language: &'a str,
    pub kubectl_command: &'a str,
    pub confidence_score: Option<u8>,
    pub risk_level: RiskLevel,
    pub environment: &'a str,
    pub cluster: &'a str,
    pub namespace: Option<&'a str>,
}

/// Helper to create audit log entry from execution
pub fn audit_entry_from_execution(
    ctx: AuditContext,
    result: &ExecutionResult,
    user_action: UserAction,
) -> AuditLogEntry {
    AuditLogEntry {
        timestamp: AuditLogger::current_timestamp(),
        user_id: AuditLogger::current_user(),
        natural_language_input: ctx.natural_language.to_string(),
        kubectl_command: ctx.kubectl_command.to_string(),
        original_command: None, // Will be set by caller if edited
        confidence_score: ctx.confidence_score,
        risk_level: ctx.risk_level,
        environment: ctx.environment.to_string(),
        cluster: ctx.cluster.to_string(),
        namespace: ctx.namespace.map(|s| s.to_string()),
        exit_code: result.exit_code,
        stdout: if result.stdout.is_empty() {
            None
        } else {
            Some(result.stdout.clone())
        },
        stderr: if result.stderr.is_empty() {
            None
        } else {
            Some(result.stderr.clone())
        },
        execution_duration_ms: Some(result.execution_duration_ms),
        user_action,
    }
}

/// Helper to create audit log entry for cancelled command
pub fn audit_entry_cancelled(
    natural_language: &str,
    kubectl_command: &str,
    confidence_score: Option<u8>,
    risk_level: RiskLevel,
    environment: &str,
    cluster: &str,
    namespace: Option<&str>,
) -> AuditLogEntry {
    AuditLogEntry {
        timestamp: AuditLogger::current_timestamp(),
        user_id: AuditLogger::current_user(),
        natural_language_input: natural_language.to_string(),
        kubectl_command: kubectl_command.to_string(),
        original_command: None, // Will be set by caller if edited
        confidence_score,
        risk_level,
        environment: environment.to_string(),
        cluster: cluster.to_string(),
        namespace: namespace.map(|s| s.to_string()),
        exit_code: None,
        stdout: None,
        stderr: None,
        execution_duration_ms: None,
        user_action: UserAction::Cancelled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_truncate_output_short() {
        let short_output = "Hello, world!";
        assert_eq!(truncate_output(short_output), short_output);
    }

    #[test]
    fn test_truncate_output_long() {
        let long_output = "x".repeat(20 * 1024); // 20KB
        let truncated = truncate_output(&long_output);
        assert!(truncated.len() <= MAX_OUTPUT_LENGTH + 50);
        assert!(truncated.ends_with("[OUTPUT TRUNCATED]"));
    }

    #[test]
    fn test_audit_logger_new() {
        let temp_db = NamedTempFile::new().unwrap();
        let logger = AuditLogger::new(temp_db.path().to_str().unwrap());
        assert!(logger.is_ok());
    }

    #[test]
    fn test_log_execution() {
        let temp_db = NamedTempFile::new().unwrap();
        let logger = AuditLogger::new(temp_db.path().to_str().unwrap()).unwrap();
        
        let entry = AuditLogEntry {
            timestamp: AuditLogger::current_timestamp(),
            user_id: "testuser".to_string(),
            natural_language_input: "show pods".to_string(),
            kubectl_command: "kubectl get pods".to_string(),
            original_command: None,
            confidence_score: Some(95),
            risk_level: RiskLevel::Low,
            environment: "dev-cluster".to_string(),
            cluster: "dev".to_string(),
            namespace: Some("default".to_string()),
            exit_code: Some(0),
            stdout: Some("pod1   Running\npod2   Running".to_string()),
            stderr: None,
            execution_duration_ms: Some(123),
            user_action: UserAction::Executed,
        };
        
        let result = logger.log_execution(entry);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }

    #[test]
    fn test_clean_old_entries() {
        let temp_db = NamedTempFile::new().unwrap();
        let logger = AuditLogger::new(temp_db.path().to_str().unwrap()).unwrap();
        
        // Insert old entry (timestamp from 100 days ago)
        let old_timestamp = AuditLogger::current_timestamp() - (100 * 24 * 60 * 60);
        let entry = AuditLogEntry {
            timestamp: old_timestamp,
            user_id: "testuser".to_string(),
            natural_language_input: "old command".to_string(),
            kubectl_command: "kubectl get pods".to_string(),
            original_command: None,
            confidence_score: Some(95),
            risk_level: RiskLevel::Low,
            environment: "dev".to_string(),
            cluster: "dev".to_string(),
            namespace: None,
            exit_code: Some(0),
            stdout: None,
            stderr: None,
            execution_duration_ms: Some(100),
            user_action: UserAction::Executed,
        };
        
        logger.log_execution(entry).unwrap();
        
        // Clean entries older than 90 days
        let deleted = logger.clean_old_entries(90).unwrap();
        assert_eq!(deleted, 1);
    }

    #[test]
    fn test_user_action_as_str() {
        assert_eq!(UserAction::Executed.as_str(), "EXECUTED");
        assert_eq!(UserAction::Cancelled.as_str(), "CANCELLED");
        assert_eq!(UserAction::Edited.as_str(), "EDITED");
    }
}
