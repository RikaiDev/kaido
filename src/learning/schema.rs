// Learning database schema
//
// SQLite schema for tracking error encounters and learning progress.

use anyhow::Result;
use rusqlite::Connection;

/// Initialize the learning database schema
pub fn init_schema(conn: &Connection) -> Result<()> {
    // Error encounters table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS error_encounters (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL,
            error_type TEXT NOT NULL,
            key_message TEXT NOT NULL,
            command TEXT NOT NULL,
            exit_code INTEGER,
            full_output TEXT,
            resolved INTEGER DEFAULT 0,
            resolution_time_ms INTEGER,
            mentor_shown INTEGER DEFAULT 1
        )",
        [],
    )?;

    // Concepts learned table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS concepts_learned (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            concept TEXT NOT NULL UNIQUE,
            first_encounter INTEGER NOT NULL,
            encounter_count INTEGER DEFAULT 1,
            last_encounter INTEGER
        )",
        [],
    )?;

    // Session statistics table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            start_time INTEGER NOT NULL,
            end_time INTEGER,
            commands_executed INTEGER DEFAULT 0,
            errors_encountered INTEGER DEFAULT 0,
            errors_resolved INTEGER DEFAULT 0
        )",
        [],
    )?;

    // Create indexes for efficient queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_error_type ON error_encounters(error_type)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_timestamp ON error_encounters(timestamp)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_resolved ON error_encounters(resolved)",
        [],
    )?;

    Ok(())
}

/// Get the default learning database path
pub fn default_learning_db_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".kaido")
        .join("learning.db")
}

/// Ensure the learning database directory exists
pub fn ensure_learning_dir() -> Result<()> {
    let db_path = default_learning_db_path();
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_schema() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_schema(&conn).is_ok());

        // Verify tables exist
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='error_encounters'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_default_learning_db_path() {
        let path = default_learning_db_path();
        assert!(path.ends_with("learning.db"));
        assert!(path.to_string_lossy().contains(".kaido"));
    }
}
