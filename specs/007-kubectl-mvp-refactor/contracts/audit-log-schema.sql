-- Audit Log Schema for Kaido AI Shell
-- Feature: Kubectl-Only MVP (60-Day Reality Check)
-- Database: SQLite 3
-- Location: ~/.kaido/audit.db

-- =============================================================================
-- TABLE: audit_log
-- Purpose: Permanent record of all kubectl commands (executed, cancelled, edited)
-- =============================================================================

CREATE TABLE IF NOT EXISTS audit_log (
    -- Primary key
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    
    -- Timestamp (Unix timestamp for efficient range queries)
    timestamp INTEGER NOT NULL,  -- Unix timestamp (seconds since epoch)
    
    -- User identification
    user_id TEXT NOT NULL,  -- System username from users::get_current_username()
    
    -- Command details
    natural_language_input TEXT NOT NULL,  -- Original user input (e.g., "show pods")
    kubectl_command TEXT NOT NULL,  -- Translated command (e.g., "kubectl get pods -n default")
    confidence_score INTEGER,  -- AI confidence 0-100, NULL if direct kubectl input
    
    -- Risk and environment
    risk_level TEXT NOT NULL CHECK(risk_level IN ('LOW', 'MEDIUM', 'HIGH')),
    environment TEXT NOT NULL,  -- Context name (e.g., "prod-cluster", "dev-cluster")
    cluster TEXT NOT NULL,  -- Cluster name from kubeconfig
    namespace TEXT,  -- Target namespace, NULL if not specified
    
    -- Execution results
    exit_code INTEGER,  -- Command exit code, NULL if cancelled before execution
    stdout TEXT,  -- Command stdout, truncated to 10KB
    stderr TEXT,  -- Command stderr, truncated to 10KB
    execution_duration_ms INTEGER,  -- Execution time in milliseconds
    
    -- User action tracking
    user_action TEXT NOT NULL CHECK(user_action IN ('EXECUTED', 'CANCELLED', 'EDITED')),
    
    -- Metadata
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))  -- ISO 8601 timestamp for human readability
);

-- =============================================================================
-- INDEXES
-- Purpose: Optimize common query patterns for TUI history commands
-- =============================================================================

-- Index for date range queries ("show history today", "show history last week")
CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp 
ON audit_log(timestamp DESC);

-- Index for environment filtering ("show history production")
CREATE INDEX IF NOT EXISTS idx_audit_log_environment 
ON audit_log(environment);

-- Index for user action analytics
CREATE INDEX IF NOT EXISTS idx_audit_log_user_action 
ON audit_log(user_action);

-- Composite index for common query: recent production commands
CREATE INDEX IF NOT EXISTS idx_audit_log_env_timestamp 
ON audit_log(environment, timestamp DESC);

-- =============================================================================
-- VIEWS
-- Purpose: Simplified queries for common TUI display patterns
-- =============================================================================

-- View: Commands executed today
CREATE VIEW IF NOT EXISTS v_today_commands AS
SELECT 
    id,
    datetime(timestamp, 'unixepoch') AS executed_at,
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

-- View: Commands from last 7 days
CREATE VIEW IF NOT EXISTS v_last_week_commands AS
SELECT 
    id,
    datetime(timestamp, 'unixepoch') AS executed_at,
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

-- View: Production environment commands
CREATE VIEW IF NOT EXISTS v_production_commands AS
SELECT 
    id,
    datetime(timestamp, 'unixepoch') AS executed_at,
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

-- View: High risk commands (for security audit)
CREATE VIEW IF NOT EXISTS v_high_risk_commands AS
SELECT 
    id,
    datetime(timestamp, 'unixepoch') AS executed_at,
    user_id,
    natural_language_input,
    kubectl_command,
    environment,
    cluster,
    namespace,
    user_action,
    exit_code
FROM audit_log
WHERE risk_level = 'HIGH'
ORDER BY timestamp DESC;

-- View: Cancelled commands (user safety behavior analysis)
CREATE VIEW IF NOT EXISTS v_cancelled_commands AS
SELECT 
    id,
    datetime(timestamp, 'unixepoch') AS cancelled_at,
    user_id,
    natural_language_input,
    kubectl_command,
    risk_level,
    environment,
    confidence_score
FROM audit_log
WHERE user_action = 'CANCELLED'
ORDER BY timestamp DESC;

-- =============================================================================
-- TRIGGERS
-- Purpose: Automatic data management (retention, validation)
-- =============================================================================

-- Trigger: Truncate stdout/stderr to 10KB on insert
CREATE TRIGGER IF NOT EXISTS trg_truncate_output
BEFORE INSERT ON audit_log
FOR EACH ROW
BEGIN
    SELECT CASE
        WHEN length(NEW.stdout) > 10240 THEN
            RAISE(IGNORE)  -- This will be handled in application code
    END;
END;

-- =============================================================================
-- SAMPLE QUERIES
-- Purpose: Common query patterns for application code
-- =============================================================================

-- Query: Get today's commands
-- SELECT * FROM v_today_commands;

-- Query: Get last week's commands with pagination
-- SELECT * FROM v_last_week_commands LIMIT 20 OFFSET 0;

-- Query: Get production commands
-- SELECT * FROM v_production_commands;

-- Query: Search commands by natural language input
-- SELECT * FROM audit_log 
-- WHERE natural_language_input LIKE '%delete%' 
-- ORDER BY timestamp DESC;

-- Query: Get command details by ID
-- SELECT 
--     datetime(timestamp, 'unixepoch') AS executed_at,
--     natural_language_input,
--     kubectl_command,
--     confidence_score,
--     risk_level,
--     environment,
--     cluster,
--     namespace,
--     user_action,
--     exit_code,
--     CASE 
--         WHEN length(stdout) > 200 THEN substr(stdout, 1, 200) || '...' 
--         ELSE stdout 
--     END AS stdout_preview,
--     stderr,
--     execution_duration_ms
-- FROM audit_log
-- WHERE id = ?;

-- Query: Delete old records (retention policy)
-- DELETE FROM audit_log 
-- WHERE timestamp < strftime('%s', 'now', '-90 days');

-- Query: Get statistics
-- SELECT 
--     COUNT(*) AS total_commands,
--     SUM(CASE WHEN user_action = 'EXECUTED' THEN 1 ELSE 0 END) AS executed,
--     SUM(CASE WHEN user_action = 'CANCELLED' THEN 1 ELSE 0 END) AS cancelled,
--     SUM(CASE WHEN user_action = 'EDITED' THEN 1 ELSE 0 END) AS edited,
--     AVG(execution_duration_ms) AS avg_duration_ms,
--     AVG(CASE WHEN confidence_score IS NOT NULL THEN confidence_score END) AS avg_confidence
-- FROM audit_log
-- WHERE timestamp >= strftime('%s', 'now', '-30 days');

-- =============================================================================
-- INITIALIZATION SCRIPT
-- Purpose: Run this when application starts to ensure schema is up-to-date
-- =============================================================================

-- Check schema version (for future migrations)
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))
);

-- Insert initial version
INSERT OR IGNORE INTO schema_version (version) VALUES (1);

-- =============================================================================
-- NOTES
-- =============================================================================

-- 1. Database location: ~/.kaido/audit.db
-- 2. File permissions: 600 (user read/write only)
-- 3. Backup strategy: Not required for MVP (logs are non-critical)
-- 4. Retention: Application deletes records older than 90 days on startup
-- 5. Concurrency: SQLite handles locking automatically (WAL mode for better concurrency)
-- 6. Truncation: stdout/stderr truncated to 10KB in application code before INSERT
-- 7. Timezone: All timestamps stored as UTC (Unix timestamp + ISO 8601 string)

-- =============================================================================
-- PRAGMA SETTINGS (to be set on connection)
-- =============================================================================

-- PRAGMA journal_mode=WAL;  -- Write-Ahead Logging for better concurrency
-- PRAGMA synchronous=NORMAL;  -- Balance between safety and performance
-- PRAGMA foreign_keys=ON;  -- Enable foreign key constraints (none in this schema)
-- PRAGMA temp_store=MEMORY;  -- Use memory for temporary tables


