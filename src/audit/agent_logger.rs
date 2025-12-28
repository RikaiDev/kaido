use anyhow::Result;
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::agent::{AgentState, AgentStep, StepType};

/// Agent audit logger for recording complete diagnosis sessions
#[derive(Clone)]
pub struct AgentAuditLogger {
    conn: Arc<Mutex<Connection>>,
}

impl AgentAuditLogger {
    /// Create new agent audit logger
    pub fn new(database_path: &str) -> Result<Self> {
        let conn = Connection::open(database_path)?;

        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA foreign_keys=ON;
             PRAGMA temp_store=MEMORY;",
        )?;

        Self::initialize_schema(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Initialize database schema
    fn initialize_schema(conn: &Connection) -> Result<()> {
        // Agent sessions table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS agent_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL UNIQUE,
                task_description TEXT NOT NULL,
                start_time INTEGER NOT NULL,
                end_time INTEGER,
                status TEXT NOT NULL,
                total_steps INTEGER NOT NULL DEFAULT 0,
                total_actions INTEGER NOT NULL DEFAULT 0,
                duration_ms INTEGER,
                root_cause TEXT,
                solution_plan TEXT,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )?;

        // Agent steps table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS agent_steps (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                step_number INTEGER NOT NULL,
                step_type TEXT NOT NULL,
                content TEXT NOT NULL,
                tool_used TEXT,
                success INTEGER,
                timestamp INTEGER NOT NULL,
                FOREIGN KEY (session_id) REFERENCES agent_sessions(session_id)
            )",
            [],
        )?;

        // Create indices
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_agent_sessions_start_time 
             ON agent_sessions(start_time DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_agent_steps_session 
             ON agent_steps(session_id, step_number)",
            [],
        )?;

        Ok(())
    }

    /// Log agent session start
    pub fn log_session_start(&self, session_id: &str, task: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

        conn.execute(
            "INSERT INTO agent_sessions 
             (session_id, task_description, start_time, status) 
             VALUES (?1, ?2, ?3, ?4)",
            params![session_id, task, timestamp, "RUNNING"],
        )?;

        Ok(())
    }

    /// Log agent step
    pub fn log_step(&self, session_id: &str, step: &AgentStep) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let timestamp = step.timestamp.duration_since(UNIX_EPOCH)?.as_secs() as i64;

        let step_type = match step.step_type {
            StepType::Thought => "THOUGHT",
            StepType::Action => "ACTION",
            StepType::Observation => "OBSERVATION",
            StepType::Reflection => "REFLECTION",
            StepType::Solution => "SOLUTION",
        };

        let success_int = step.success.map(|b| if b { 1 } else { 0 });

        conn.execute(
            "INSERT INTO agent_steps 
             (session_id, step_number, step_type, content, tool_used, success, timestamp) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                session_id,
                step.step_number as i64,
                step_type,
                step.content,
                step.tool_used,
                success_int,
                timestamp
            ],
        )?;

        Ok(())
    }

    /// Log agent session completion
    pub fn log_session_end(&self, session_id: &str, final_state: &AgentState) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let end_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

        let duration_ms = final_state.start_time.elapsed().as_millis() as i64;

        let status = format!("{:?}", final_state.status);
        let total_steps = final_state.history.len() as i64;
        let total_actions = final_state
            .history
            .iter()
            .filter(|s| s.step_type == StepType::Action)
            .count() as i64;

        let solution_plan_json = if let Some(plan) = &final_state.solution_plan {
            Some(serde_json::to_string(plan)?)
        } else {
            None
        };

        conn.execute(
            "UPDATE agent_sessions 
             SET end_time = ?1, status = ?2, total_steps = ?3, total_actions = ?4, 
                 duration_ms = ?5, root_cause = ?6, solution_plan = ?7
             WHERE session_id = ?8",
            params![
                end_time,
                status,
                total_steps,
                total_actions,
                duration_ms,
                final_state.root_cause,
                solution_plan_json,
                session_id
            ],
        )?;

        Ok(())
    }

    /// Get recent agent sessions
    pub fn get_recent_sessions(&self, limit: usize) -> Result<Vec<AgentSessionSummary>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT session_id, task_description, start_time, end_time, status, 
                    total_steps, total_actions, duration_ms, root_cause
             FROM agent_sessions
             ORDER BY start_time DESC
             LIMIT ?1",
        )?;

        let sessions = stmt
            .query_map(params![limit as i64], |row| {
                Ok(AgentSessionSummary {
                    session_id: row.get(0)?,
                    task_description: row.get(1)?,
                    start_time: row.get(2)?,
                    end_time: row.get(3)?,
                    status: row.get(4)?,
                    total_steps: row.get(5)?,
                    total_actions: row.get(6)?,
                    duration_ms: row.get(7)?,
                    root_cause: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    /// Get agent session details with all steps
    pub fn get_session_details(&self, session_id: &str) -> Result<Option<AgentSessionDetail>> {
        let conn = self.conn.lock().unwrap();

        // Get session
        let mut session_stmt = conn.prepare(
            "SELECT task_description, start_time, end_time, status, 
                    total_steps, total_actions, duration_ms, root_cause, solution_plan
             FROM agent_sessions
             WHERE session_id = ?1",
        )?;

        let session = session_stmt.query_row(params![session_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Option<i64>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, Option<i64>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
            ))
        });

        let session = match session {
            Ok(s) => s,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        // Get steps
        let mut steps_stmt = conn.prepare(
            "SELECT step_number, step_type, content, tool_used, success, timestamp
             FROM agent_steps
             WHERE session_id = ?1
             ORDER BY step_number",
        )?;

        let steps = steps_stmt
            .query_map(params![session_id], |row| {
                Ok(AgentStepSummary {
                    step_number: row.get(0)?,
                    step_type: row.get(1)?,
                    content: row.get(2)?,
                    tool_used: row.get(3)?,
                    success: row.get(4)?,
                    timestamp: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Some(AgentSessionDetail {
            session_id: session_id.to_string(),
            task_description: session.0,
            start_time: session.1,
            end_time: session.2,
            status: session.3,
            total_steps: session.4,
            total_actions: session.5,
            duration_ms: session.6,
            root_cause: session.7,
            solution_plan: session.8,
            steps,
        }))
    }

    /// Clean old sessions (retention policy)
    pub fn clean_old_sessions(&self, retention_days: i64) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let cutoff_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64
            - (retention_days * 24 * 60 * 60);

        // Delete old steps first (foreign key constraint)
        conn.execute(
            "DELETE FROM agent_steps 
             WHERE session_id IN (
                 SELECT session_id FROM agent_sessions WHERE start_time < ?1
             )",
            params![cutoff_time],
        )?;

        // Delete old sessions
        let deleted = conn.execute(
            "DELETE FROM agent_sessions WHERE start_time < ?1",
            params![cutoff_time],
        )?;

        Ok(deleted)
    }
}

/// Agent session summary
#[derive(Debug, Clone)]
pub struct AgentSessionSummary {
    pub session_id: String,
    pub task_description: String,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub status: String,
    pub total_steps: i64,
    pub total_actions: i64,
    pub duration_ms: Option<i64>,
    pub root_cause: Option<String>,
}

/// Agent session detail with steps
#[derive(Debug, Clone)]
pub struct AgentSessionDetail {
    pub session_id: String,
    pub task_description: String,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub status: String,
    pub total_steps: i64,
    pub total_actions: i64,
    pub duration_ms: Option<i64>,
    pub root_cause: Option<String>,
    pub solution_plan: Option<String>,
    pub steps: Vec<AgentStepSummary>,
}

/// Agent step summary
#[derive(Debug, Clone)]
pub struct AgentStepSummary {
    pub step_number: i64,
    pub step_type: String,
    pub content: String,
    pub tool_used: Option<String>,
    pub success: Option<i32>,
    pub timestamp: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_logger_creation() {
        let logger = AgentAuditLogger::new(":memory:").unwrap();
        assert!(logger.get_recent_sessions(10).unwrap().is_empty());
    }
}
