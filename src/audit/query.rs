// Audit query implementation for command history retrieval
use anyhow::Result;
use rusqlite::{params, Connection};

/// Query result entry for TUI display
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub id: i64,
    pub executed_at: String,
    pub user_id: String,
    pub natural_language_input: String,
    pub kubectl_command: String,
    pub risk_level: String,
    pub environment: String,
    pub user_action: String,
    pub exit_code: Option<i32>,
}

impl QueryResult {
    /// Check if execution failed
    pub fn is_failed(&self) -> bool {
        self.exit_code.is_some_and(|code| code != 0)
    }

    /// Get risk level as enum
    pub fn risk_level_enum(&self) -> crate::kubectl::RiskLevel {
        match self.risk_level.as_str() {
            "LOW" => crate::kubectl::RiskLevel::Low,
            "MEDIUM" => crate::kubectl::RiskLevel::Medium,
            _ => crate::kubectl::RiskLevel::High,
        }
    }

    /// Get display description for TUI
    pub fn display_summary(&self) -> String {
        format!(
            "{} | {} | {} | {}",
            self.executed_at, self.user_id, self.kubectl_command, self.risk_level
        )
    }
}

/// Audit query interface for history retrieval
///
/// TODO: Future Analytics for Model Fine-Tuning
///
/// The audit log captures all command edits (user_action='EDITED') with both the original
/// AI-generated command and the user's corrected version. This data can be used to:
///
/// 1. Identify common AI translation errors (e.g., missing required flags, incorrect syntax)
/// 2. Analyze correction patterns to improve prompt engineering
/// 3. Build a training dataset for fine-tuning the translation model
/// 4. Track translation accuracy improvement over time
///
/// Example analytics queries:
///
/// ```sql
/// -- Find most common corrections (original → edited patterns)
/// SELECT original_command, kubectl_command, COUNT(*) as frequency
/// FROM audit_log
/// WHERE user_action = 'EDITED' AND original_command IS NOT NULL
/// GROUP BY original_command, kubectl_command
/// ORDER BY frequency DESC
/// LIMIT 20;
///
/// -- Identify low-confidence commands that users still execute without editing
/// SELECT natural_language_input, kubectl_command, confidence_score, AVG(confidence_score) as avg_confidence
/// FROM audit_log
/// WHERE user_action = 'EXECUTED' AND confidence_score < 70
/// GROUP BY natural_language_input, kubectl_command
/// ORDER BY avg_confidence ASC;
///
/// -- Find natural language patterns that consistently require editing
/// SELECT natural_language_input, COUNT(*) as edit_count
/// FROM audit_log
/// WHERE user_action = 'EDITED'
/// GROUP BY natural_language_input
/// HAVING edit_count > 2
/// ORDER BY edit_count DESC;
/// ```
pub struct AuditQuery {
    conn: Connection,
}

impl AuditQuery {
    /// Create new audit query interface
    pub fn new(database_path: &str) -> Result<Self> {
        let conn = Connection::open(database_path)?;
        Ok(Self { conn })
    }

    /// Query today's commands
    ///
    /// Returns all commands executed today, sorted by timestamp (newest first)
    /// Uses view v_today_commands for optimized query
    pub fn query_today(&self, limit: Option<usize>) -> Result<Vec<QueryResult>> {
        let sql = if let Some(limit) = limit {
            format!("SELECT * FROM v_today_commands LIMIT {limit}")
        } else {
            "SELECT * FROM v_today_commands".to_string()
        };

        self.execute_query(&sql, params![])
    }

    /// Query last week's commands
    ///
    /// Returns all commands from the last 7 days, sorted by timestamp (newest first)
    /// Uses view v_last_week_commands for optimized query
    pub fn query_last_week(&self, limit: Option<usize>) -> Result<Vec<QueryResult>> {
        let sql = if let Some(limit) = limit {
            format!("SELECT * FROM v_last_week_commands LIMIT {limit}")
        } else {
            "SELECT * FROM v_last_week_commands".to_string()
        };

        self.execute_query(&sql, params![])
    }

    /// Query production environment commands
    ///
    /// Returns all commands executed in production context (environment contains "prod" or "production")
    /// Uses view v_production_commands for optimized query
    pub fn query_production(&self, limit: Option<usize>) -> Result<Vec<QueryResult>> {
        let sql = if let Some(limit) = limit {
            format!("SELECT * FROM v_production_commands LIMIT {limit}")
        } else {
            "SELECT * FROM v_production_commands".to_string()
        };

        self.execute_query(&sql, params![])
    }

    /// Execute query and return results
    fn execute_query(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::ToSql],
    ) -> Result<Vec<QueryResult>> {
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params, |row| {
            Ok(QueryResult {
                id: row.get(0)?,
                executed_at: row.get(1)?,
                user_id: row.get(2)?,
                natural_language_input: row.get(3)?,
                kubectl_command: row.get(4)?,
                risk_level: row.get(5)?,
                environment: row.get(6)?,
                user_action: row.get(7)?,
                exit_code: row.get(8)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    /// Format query results as table for TUI display
    ///
    /// Returns formatted string with aligned columns:
    /// ID | Time | Command | Environment | Action | Exit Code
    pub fn format_table(results: &[QueryResult], max_rows: usize) -> String {
        if results.is_empty() {
            return "No commands found.".to_string();
        }

        let results_to_show = if results.len() > max_rows {
            &results[..max_rows]
        } else {
            results
        };

        let mut output = String::new();

        // Header
        output.push_str(&format!(
            "{:<6} {:<20} {:<40} {:<15} {:<10} {:<8}\n",
            "ID", "Time", "Command", "Environment", "Action", "Exit"
        ));
        output.push_str(&format!("{}\n", "-".repeat(100)));

        // Rows
        for result in results_to_show {
            let time = if result.executed_at.len() > 19 {
                &result.executed_at[..19] // Truncate to "YYYY-MM-DD HH:MM:SS"
            } else {
                &result.executed_at
            };

            let command = if result.kubectl_command.len() > 40 {
                format!("{}...", &result.kubectl_command[..37])
            } else {
                result.kubectl_command.clone()
            };

            let environment = if result.environment.len() > 15 {
                format!("{}...", &result.environment[..12])
            } else {
                result.environment.clone()
            };

            let exit_code = result
                .exit_code
                .map(|c| c.to_string())
                .unwrap_or_else(|| "-".to_string());

            output.push_str(&format!(
                "{:<6} {:<20} {:<40} {:<15} {:<10} {:<8}\n",
                result.id, time, command, environment, result.user_action, exit_code
            ));
        }

        // Footer
        if results.len() > max_rows {
            output.push_str(&format!(
                "\nShowing {} of {} results. Use pagination for more.\n",
                max_rows,
                results.len()
            ));
        } else {
            output.push_str(&format!("\nTotal: {} results\n", results.len()));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::logger::AuditLogEntry;
    use crate::audit::{AuditLogger, UserAction};
    use crate::kubectl::RiskLevel;
    use tempfile::NamedTempFile;

    fn create_test_db() -> (NamedTempFile, AuditLogger) {
        let temp_db = NamedTempFile::new().unwrap();
        let logger = AuditLogger::new(temp_db.path().to_str().unwrap()).unwrap();
        (temp_db, logger)
    }

    fn create_test_entry(
        natural_language: &str,
        kubectl_command: &str,
        risk: RiskLevel,
        env: &str,
    ) -> AuditLogEntry {
        AuditLogEntry {
            timestamp: AuditLogger::current_timestamp(),
            user_id: "testuser".to_string(),
            natural_language_input: natural_language.to_string(),
            kubectl_command: kubectl_command.to_string(),
            original_command: None,
            confidence_score: Some(95),
            risk_level: risk,
            environment: env.to_string(),
            cluster: "test-cluster".to_string(),
            namespace: Some("default".to_string()),
            exit_code: Some(0),
            stdout: Some("output".to_string()),
            stderr: None,
            execution_duration_ms: Some(100),
            user_action: UserAction::Executed,
        }
    }

    #[test]
    fn test_query_today() {
        let (temp_db, logger) = create_test_db();

        // Insert test entry
        let entry = create_test_entry("show pods", "kubectl get pods", RiskLevel::Low, "dev");
        logger.log_execution(entry).unwrap();

        // Query
        let query = AuditQuery::new(temp_db.path().to_str().unwrap()).unwrap();
        let results = query.query_today(None).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].natural_language_input, "show pods");
        assert_eq!(results[0].kubectl_command, "kubectl get pods");
    }

    #[test]
    fn test_query_production() {
        let (temp_db, logger) = create_test_db();

        // Insert test entries
        logger
            .log_execution(create_test_entry(
                "show pods",
                "kubectl get pods",
                RiskLevel::Low,
                "prod-cluster",
            ))
            .unwrap();

        logger
            .log_execution(create_test_entry(
                "show services",
                "kubectl get services",
                RiskLevel::Low,
                "dev-cluster",
            ))
            .unwrap();

        // Query production only
        let query = AuditQuery::new(temp_db.path().to_str().unwrap()).unwrap();
        let results = query.query_production(None).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].environment, "prod-cluster");
    }

    #[test]
    fn test_query_with_limit() {
        let (temp_db, logger) = create_test_db();

        // Insert multiple entries
        for i in 0..5 {
            logger
                .log_execution(create_test_entry(
                    &format!("command {i}"),
                    &format!("kubectl cmd {i}"),
                    RiskLevel::Low,
                    "dev",
                ))
                .unwrap();
        }

        // Query with limit
        let query = AuditQuery::new(temp_db.path().to_str().unwrap()).unwrap();
        let results = query.query_today(Some(3)).unwrap();

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_format_table() {
        let results = vec![QueryResult {
            id: 1,
            executed_at: "2025-10-25 10:00:00".to_string(),
            user_id: "testuser".to_string(),
            natural_language_input: "show pods".to_string(),
            kubectl_command: "kubectl get pods".to_string(),
            risk_level: "LOW".to_string(),
            environment: "dev".to_string(),
            user_action: "EXECUTED".to_string(),
            exit_code: Some(0),
        }];

        let formatted = AuditQuery::format_table(&results, 20);
        assert!(formatted.contains("ID"));
        assert!(formatted.contains("Time"));
        assert!(formatted.contains("kubectl get pods"));
        assert!(formatted.contains("Total: 1 results"));
    }

    #[test]
    fn test_format_table_empty() {
        let results = vec![];
        let formatted = AuditQuery::format_table(&results, 20);
        assert_eq!(formatted, "No commands found.");
    }

    #[test]
    fn test_format_table_truncation() {
        let results = vec![QueryResult {
            id: 1,
            executed_at: "2025-10-25 10:00:00".to_string(),
            user_id: "testuser".to_string(),
            natural_language_input: "very long command".to_string(),
            kubectl_command:
                "kubectl get pods -n namespace-with-very-long-name --selector=app=myapp".to_string(),
            risk_level: "LOW".to_string(),
            environment: "development-cluster".to_string(),
            user_action: "EXECUTED".to_string(),
            exit_code: Some(0),
        }];

        let formatted = AuditQuery::format_table(&results, 20);
        // Should truncate long command and environment
        assert!(formatted.contains("..."));
    }
}
