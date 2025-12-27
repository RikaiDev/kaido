// Audit module for command logging and history
//
// This module provides:
// - schema.rs: SQLite schema initialization
// - logger.rs: Write audit log entries
// - query.rs: Query audit log (today, last week, production)

pub mod schema;
pub mod logger;
pub mod agent_logger;
pub mod query;

pub use logger::{
    AuditLogger, AuditContext, UserAction,
    audit_entry_from_execution, audit_entry_cancelled,
};
pub use agent_logger::{AgentAuditLogger, AgentSessionSummary, AgentSessionDetail};

