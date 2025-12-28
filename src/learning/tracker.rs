// Learning Tracker
//
// Tracks error encounters and learning progress for users.

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::schema::{ensure_learning_dir, init_schema};
use crate::mentor::ErrorType;

/// A recorded error encounter
#[derive(Debug, Clone)]
pub struct ErrorEncounter {
    pub id: i64,
    pub timestamp: u64,
    pub error_type: String,
    pub key_message: String,
    pub command: String,
    pub exit_code: Option<i32>,
    pub resolved: bool,
    pub resolution_time_ms: Option<u64>,
    pub mentor_shown: bool,
}

/// Summary of errors by type
#[derive(Debug, Clone)]
pub struct ErrorSummary {
    pub error_type: String,
    pub count: u32,
    pub resolved_count: u32,
    pub last_seen: u64,
}

/// Learning progress statistics
#[derive(Debug, Clone)]
pub struct LearningProgress {
    /// Total errors encountered
    pub total_errors: u32,
    /// Number of resolved errors
    pub resolved_errors: u32,
    /// Resolution rate (0.0 - 1.0)
    pub resolution_rate: f32,
    /// Errors grouped by type
    pub errors_by_type: HashMap<String, u32>,
    /// Most common errors (type, count)
    pub common_errors: Vec<(String, u32)>,
    /// Concepts encountered
    pub concepts: Vec<String>,
}

impl LearningProgress {
    /// Create empty progress
    pub fn empty() -> Self {
        Self {
            total_errors: 0,
            resolved_errors: 0,
            resolution_rate: 0.0,
            errors_by_type: HashMap::new(),
            common_errors: Vec::new(),
            concepts: Vec::new(),
        }
    }
}

/// Learning tracker for recording error encounters and progress
pub struct LearningTracker {
    conn: Mutex<Connection>,
    session_id: Option<i64>,
}

impl LearningTracker {
    /// Create a new learning tracker with the given database path
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        init_schema(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
            session_id: None,
        })
    }

    /// Create a learning tracker with the default database path
    pub fn with_default_path() -> Result<Self> {
        ensure_learning_dir()?;
        let db_path = super::schema::default_learning_db_path();
        Self::new(db_path)
    }

    /// Create an in-memory tracker (for testing)
    pub fn in_memory() -> Result<Self> {
        Self::new(":memory:")
    }

    /// Start a new learning session
    pub fn start_session(&mut self) -> Result<i64> {
        let now = current_timestamp();
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("{e}"))?;

        conn.execute("INSERT INTO sessions (start_time) VALUES (?)", params![now])?;

        let session_id = conn.last_insert_rowid();
        self.session_id = Some(session_id);
        Ok(session_id)
    }

    /// End the current session
    pub fn end_session(&mut self) -> Result<()> {
        if let Some(session_id) = self.session_id.take() {
            let now = current_timestamp();
            let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("{e}"))?;

            conn.execute(
                "UPDATE sessions SET end_time = ? WHERE id = ?",
                params![now, session_id],
            )?;
        }
        Ok(())
    }

    /// Record a new error encounter
    pub fn record_error(
        &self,
        error_type: &ErrorType,
        key_message: &str,
        command: &str,
        exit_code: Option<i32>,
        full_output: Option<&str>,
    ) -> Result<i64> {
        let now = current_timestamp();
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("{e}"))?;

        conn.execute(
            "INSERT INTO error_encounters (timestamp, error_type, key_message, command, exit_code, full_output)
             VALUES (?, ?, ?, ?, ?, ?)",
            params![
                now,
                error_type.name(),
                key_message,
                command,
                exit_code,
                full_output
            ],
        )?;

        let error_id = conn.last_insert_rowid();

        // Update session stats
        if let Some(session_id) = self.session_id {
            conn.execute(
                "UPDATE sessions SET errors_encountered = errors_encountered + 1 WHERE id = ?",
                params![session_id],
            )?;
        }

        // Record concept if applicable
        let concept = error_type.name().to_string();
        self.record_concept_internal(&conn, &concept, now)?;

        Ok(error_id)
    }

    /// Mark an error as resolved
    pub fn mark_resolved(&self, error_id: i64, resolution_time: Duration) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("{e}"))?;

        conn.execute(
            "UPDATE error_encounters SET resolved = 1, resolution_time_ms = ? WHERE id = ?",
            params![resolution_time.as_millis() as i64, error_id],
        )?;

        // Update session stats
        if let Some(session_id) = self.session_id {
            conn.execute(
                "UPDATE sessions SET errors_resolved = errors_resolved + 1 WHERE id = ?",
                params![session_id],
            )?;
        }

        Ok(())
    }

    /// Record a concept learned
    fn record_concept_internal(&self, conn: &Connection, concept: &str, now: u64) -> Result<()> {
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM concepts_learned WHERE concept = ?",
                params![concept],
                |row| row.get(0),
            )
            .optional()?;

        if let Some(id) = existing {
            conn.execute(
                "UPDATE concepts_learned SET encounter_count = encounter_count + 1, last_encounter = ? WHERE id = ?",
                params![now, id],
            )?;
        } else {
            conn.execute(
                "INSERT INTO concepts_learned (concept, first_encounter, last_encounter) VALUES (?, ?, ?)",
                params![concept, now, now],
            )?;
        }

        Ok(())
    }

    /// Get the most recent error encounter
    pub fn get_last_error(&self) -> Result<Option<ErrorEncounter>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("{e}"))?;

        let result = conn
            .query_row(
                "SELECT id, timestamp, error_type, key_message, command, exit_code, resolved, resolution_time_ms, mentor_shown
                 FROM error_encounters ORDER BY id DESC LIMIT 1",
                [],
                |row| {
                    Ok(ErrorEncounter {
                        id: row.get(0)?,
                        timestamp: row.get(1)?,
                        error_type: row.get(2)?,
                        key_message: row.get(3)?,
                        command: row.get(4)?,
                        exit_code: row.get(5)?,
                        resolved: row.get::<_, i32>(6)? != 0,
                        resolution_time_ms: row.get(7)?,
                        mentor_shown: row.get::<_, i32>(8)? != 0,
                    })
                },
            )
            .optional()?;

        Ok(result)
    }

    /// Get learning progress summary
    pub fn get_progress(&self) -> Result<LearningProgress> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("{e}"))?;

        // Total errors
        let total_errors: i64 =
            conn.query_row("SELECT COUNT(*) FROM error_encounters", [], |row| {
                row.get(0)
            })?;

        // Resolved errors
        let resolved_errors: i64 = conn.query_row(
            "SELECT COUNT(*) FROM error_encounters WHERE resolved = 1",
            [],
            |row| row.get(0),
        )?;

        // Resolution rate
        let resolution_rate = if total_errors > 0 {
            resolved_errors as f32 / total_errors as f32
        } else {
            0.0
        };

        // Errors by type
        let mut errors_by_type = HashMap::new();
        let mut stmt = conn.prepare(
            "SELECT error_type, COUNT(*) as count FROM error_encounters GROUP BY error_type",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        for row in rows {
            let (error_type, count) = row?;
            errors_by_type.insert(error_type, count as u32);
        }

        // Most common errors (top 5)
        let mut common_errors = Vec::new();
        let mut stmt = conn.prepare(
            "SELECT error_type, COUNT(*) as count FROM error_encounters
             GROUP BY error_type ORDER BY count DESC LIMIT 5",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        for row in rows {
            let (error_type, count) = row?;
            common_errors.push((error_type, count as u32));
        }

        // Concepts
        let mut concepts = Vec::new();
        let mut stmt =
            conn.prepare("SELECT concept FROM concepts_learned ORDER BY encounter_count DESC")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        for row in rows {
            concepts.push(row?);
        }

        Ok(LearningProgress {
            total_errors: total_errors as u32,
            resolved_errors: resolved_errors as u32,
            resolution_rate,
            errors_by_type,
            common_errors,
            concepts,
        })
    }

    /// Get error summary by type
    pub fn get_error_summaries(&self, limit: usize) -> Result<Vec<ErrorSummary>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("{e}"))?;

        let mut stmt = conn.prepare(
            "SELECT error_type,
                    COUNT(*) as total_count,
                    SUM(CASE WHEN resolved = 1 THEN 1 ELSE 0 END) as resolved_count,
                    MAX(timestamp) as last_seen
             FROM error_encounters
             GROUP BY error_type
             ORDER BY total_count DESC
             LIMIT ?",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(ErrorSummary {
                error_type: row.get(0)?,
                count: row.get::<_, i64>(1)? as u32,
                resolved_count: row.get::<_, i64>(2)? as u32,
                last_seen: row.get(3)?,
            })
        })?;

        let mut summaries = Vec::new();
        for row in rows {
            summaries.push(row?);
        }

        Ok(summaries)
    }

    /// Check if commands are similar (for resolution detection)
    pub fn is_similar_command(cmd1: &str, cmd2: &str) -> bool {
        // Extract the base command (first word)
        let base1 = cmd1.split_whitespace().next().unwrap_or("");
        let base2 = cmd2.split_whitespace().next().unwrap_or("");

        // Same base command = similar
        base1 == base2
    }
}

/// Get current timestamp in milliseconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_creation() {
        let tracker = LearningTracker::in_memory();
        assert!(tracker.is_ok());
    }

    #[test]
    fn test_record_error() {
        let tracker = LearningTracker::in_memory().unwrap();

        let error_id = tracker
            .record_error(
                &ErrorType::CommandNotFound,
                "command not found: foo",
                "foo --bar",
                Some(127),
                None,
            )
            .unwrap();

        assert!(error_id > 0);

        // Verify it was recorded
        let last = tracker.get_last_error().unwrap();
        assert!(last.is_some());
        let last = last.unwrap();
        assert_eq!(last.id, error_id);
        assert_eq!(last.key_message, "command not found: foo");
        assert!(!last.resolved);
    }

    #[test]
    fn test_mark_resolved() {
        let tracker = LearningTracker::in_memory().unwrap();

        let error_id = tracker
            .record_error(
                &ErrorType::CommandNotFound,
                "command not found: foo",
                "foo --bar",
                Some(127),
                None,
            )
            .unwrap();

        tracker
            .mark_resolved(error_id, Duration::from_secs(30))
            .unwrap();

        let last = tracker.get_last_error().unwrap().unwrap();
        assert!(last.resolved);
        assert_eq!(last.resolution_time_ms, Some(30000));
    }

    #[test]
    fn test_get_progress() {
        let tracker = LearningTracker::in_memory().unwrap();

        // Record some errors
        tracker
            .record_error(
                &ErrorType::CommandNotFound,
                "not found 1",
                "cmd1",
                Some(127),
                None,
            )
            .unwrap();
        tracker
            .record_error(
                &ErrorType::CommandNotFound,
                "not found 2",
                "cmd2",
                Some(127),
                None,
            )
            .unwrap();
        let id = tracker
            .record_error(
                &ErrorType::PermissionDenied,
                "permission denied",
                "cmd3",
                Some(1),
                None,
            )
            .unwrap();

        // Resolve one
        tracker.mark_resolved(id, Duration::from_secs(10)).unwrap();

        let progress = tracker.get_progress().unwrap();
        assert_eq!(progress.total_errors, 3);
        assert_eq!(progress.resolved_errors, 1);
        assert!((progress.resolution_rate - 0.333).abs() < 0.01);
        assert_eq!(progress.errors_by_type.get("Command Not Found"), Some(&2));
        assert_eq!(progress.errors_by_type.get("Permission Denied"), Some(&1));
    }

    #[test]
    fn test_session_tracking() {
        let mut tracker = LearningTracker::in_memory().unwrap();

        let session_id = tracker.start_session().unwrap();
        assert!(session_id > 0);

        tracker
            .record_error(&ErrorType::CommandNotFound, "error", "cmd", Some(127), None)
            .unwrap();

        tracker.end_session().unwrap();
        assert!(tracker.session_id.is_none());
    }

    #[test]
    fn test_is_similar_command() {
        assert!(LearningTracker::is_similar_command("ls -la", "ls /tmp"));
        assert!(LearningTracker::is_similar_command(
            "kubectl get pods",
            "kubectl get services"
        ));
        assert!(!LearningTracker::is_similar_command("ls", "cat"));
    }

    #[test]
    fn test_error_summaries() {
        let tracker = LearningTracker::in_memory().unwrap();

        // Record multiple errors
        for _ in 0..3 {
            tracker
                .record_error(
                    &ErrorType::CommandNotFound,
                    "not found",
                    "cmd",
                    Some(127),
                    None,
                )
                .unwrap();
        }
        for _ in 0..2 {
            tracker
                .record_error(&ErrorType::PermissionDenied, "denied", "cmd", Some(1), None)
                .unwrap();
        }

        let summaries = tracker.get_error_summaries(5).unwrap();
        assert_eq!(summaries.len(), 2);
        assert_eq!(summaries[0].error_type, "Command Not Found");
        assert_eq!(summaries[0].count, 3);
        assert_eq!(summaries[1].error_type, "Permission Denied");
        assert_eq!(summaries[1].count, 2);
    }
}
