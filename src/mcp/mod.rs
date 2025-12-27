// MCP (Model Context Protocol) Server Module
//
// Exposes Kaido tools via MCP for integration with Claude Code
// and other MCP-compatible clients.
//
// Tools exposed:
// - kaido_diagnose: AI-powered problem diagnosis
// - kaido_execute: Command execution with risk assessment
// - kaido_explain: Educational command explanations
// - kaido_get_context: System context information
// - kaido_list_tools: Available tools listing
// - kaido_check_risk: Command risk assessment

pub mod server;
pub mod tools;
pub mod types;

pub use server::McpServer;
pub use tools::KaidoTools;
pub use types::*;
