// SQLite schema for audit logging
// Based on contracts/audit-log-schema.sql

/// SQL schema for audit_log table
pub const AUDIT_LOG_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS audit_log (
    -- Primary key
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    
    -- Timestamp (Unix timestamp for efficient range queries)
    timestamp INTEGER NOT NULL,
    
    -- User identification
    user_id TEXT NOT NULL,
    
    -- Command details
    natural_language_input TEXT NOT NULL,
    kubectl_command TEXT NOT NULL,
    original_command TEXT,  -- AI-generated command before user edit (NULL if not edited)
    confidence_score INTEGER,
    
    -- Risk and environment
    risk_level TEXT NOT NULL CHECK(risk_level IN ('LOW', 'MEDIUM', 'HIGH')),
    environment TEXT NOT NULL,
    cluster TEXT NOT NULL,
    namespace TEXT,
    
    -- Execution results
    exit_code INTEGER,
    stdout TEXT,
    stderr TEXT,
    execution_duration_ms INTEGER,
    
    -- User action tracking
    user_action TEXT NOT NULL CHECK(user_action IN ('EXECUTED', 'CANCELLED', 'EDITED')),
    
    -- Metadata
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))
);
"#;

/// SQL indexes for optimizing queries
pub const AUDIT_LOG_INDEXES: &str = r#"
-- Index for date range queries
CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp 
ON audit_log(timestamp DESC);

-- Index for environment filtering
CREATE INDEX IF NOT EXISTS idx_audit_log_environment 
ON audit_log(environment);

-- Index for user action analytics
CREATE INDEX IF NOT EXISTS idx_audit_log_user_action 
ON audit_log(user_action);

-- Composite index for common query: recent production commands
CREATE INDEX IF NOT EXISTS idx_audit_log_env_timestamp 
ON audit_log(environment, timestamp DESC);
"#;

/// SQL views for common queries
pub const AUDIT_LOG_VIEWS: &str = r#"
-- View for today's commands
CREATE VIEW IF NOT EXISTS v_today_commands AS
SELECT 
    id,
    datetime(timestamp, 'unixepoch') as executed_at,
    user_id,
    natural_language_input,
    kubectl_command,
    risk_level,
    environment,
    user_action,
    exit_code
FROM audit_log
WHERE timestamp >= strftime('%s', 'now', 'start of day')
ORDER BY timestamp DESC;

-- View for last week's commands
CREATE VIEW IF NOT EXISTS v_last_week_commands AS
SELECT 
    id,
    datetime(timestamp, 'unixepoch') as executed_at,
    user_id,
    natural_language_input,
    kubectl_command,
    risk_level,
    environment,
    user_action,
    exit_code
FROM audit_log
WHERE timestamp >= strftime('%s', 'now', '-7 days')
ORDER BY timestamp DESC;

-- View for production commands
CREATE VIEW IF NOT EXISTS v_production_commands AS
SELECT 
    id,
    datetime(timestamp, 'unixepoch') as executed_at,
    user_id,
    natural_language_input,
    kubectl_command,
    risk_level,
    environment,
    user_action,
    exit_code
FROM audit_log
WHERE environment LIKE '%prod%' OR environment LIKE '%production%'
ORDER BY timestamp DESC;
"#;

/// Initialize database schema
pub fn initialize_schema(conn: &rusqlite::Connection) -> anyhow::Result<()> {
    // Create audit_log table
    conn.execute(AUDIT_LOG_SCHEMA, [])?;

    // Create indexes
    conn.execute_batch(AUDIT_LOG_INDEXES)?;

    // Create views
    conn.execute_batch(AUDIT_LOG_VIEWS)?;

    // Set PRAGMA settings for better performance (use execute_batch for PRAGMA)
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;
         PRAGMA foreign_keys=ON;
         PRAGMA temp_store=MEMORY;",
    )?;

    log::info!("Audit log schema initialized");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_initialization() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();

        // Verify table exists
        let table_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='audit_log'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_count, 1);

        // Verify indexes exist
        let index_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_audit_log%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(index_count, 4);
    }
}
// Note: clean_old_entries test removed - function needs to be implemented
