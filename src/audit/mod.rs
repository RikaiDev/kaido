// Audit module for command logging and history
//
// This module provides:
// - schema.rs: SQLite schema initialization
// - logger.rs: Write audit log entries
// - query.rs: Query audit log (today, last week, production)

pub mod agent_logger;
pub mod logger;
pub mod query;
pub mod schema;

pub use agent_logger::{AgentAuditLogger, AgentSessionDetail, AgentSessionSummary};
pub use logger::{
    audit_entry_cancelled, audit_entry_from_execution, AuditContext, AuditLogger, UserAction,
};
