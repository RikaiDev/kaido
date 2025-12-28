// MCP Server Implementation
// Handles JSON-RPC 2.0 communication over stdio

use super::tools::KaidoTools;
use super::types::*;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use tokio::runtime::Runtime;

/// MCP Server for Kaido
pub struct McpServer {
    tools: KaidoTools,
    initialized: bool,
    runtime: Runtime,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new() -> Self {
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");
        Self {
            tools: KaidoTools::new(),
            initialized: false,
            runtime,
        }
    }

    /// Run the server, processing stdin and writing to stdout
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();
        let reader = BufReader::new(stdin.lock());

        eprintln!("[kaido-mcp] Server started, waiting for requests...");

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("[kaido-mcp] Error reading input: {e}");
                    continue;
                }
            };

            if line.trim().is_empty() {
                continue;
            }

            eprintln!("[kaido-mcp] Received: {}", &line[..line.len().min(100)]);

            let response = self.handle_message(&line);

            if let Some(resp) = response {
                let json_str = serde_json::to_string(&resp)?;
                eprintln!(
                    "[kaido-mcp] Sending: {}",
                    &json_str[..json_str.len().min(100)]
                );
                writeln!(stdout, "{json_str}")?;
                stdout.flush()?;
            }
        }

        Ok(())
    }

    /// Handle a single JSON-RPC message
    fn handle_message(&mut self, message: &str) -> Option<JsonRpcResponse> {
        // Parse JSON
        let request: JsonRpcRequest = match serde_json::from_str(message) {
            Ok(req) => req,
            Err(e) => {
                return Some(JsonRpcResponse::error(
                    None,
                    JsonRpcError::parse_error(&e.to_string()),
                ));
            }
        };

        // Route to handler
        let result = self.handle_request(&request);

        // Build response
        Some(match result {
            Ok(value) => JsonRpcResponse::success(request.id.clone(), value),
            Err(error) => JsonRpcResponse::error(request.id.clone(), error),
        })
    }

    /// Handle a parsed request
    fn handle_request(&mut self, request: &JsonRpcRequest) -> Result<Value, JsonRpcError> {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(&request.params),
            "initialized" => Ok(json!({})),
            "tools/list" => self.handle_tools_list(),
            "tools/call" => self.handle_tool_call(&request.params),
            "ping" => Ok(json!({})),
            "shutdown" => {
                eprintln!("[kaido-mcp] Shutdown requested");
                std::process::exit(0);
            }
            method => Err(JsonRpcError::method_not_found(method)),
        }
    }

    /// Handle initialize request
    fn handle_initialize(&mut self, params: &Option<Value>) -> Result<Value, JsonRpcError> {
        if let Some(p) = params {
            if let Ok(init_params) = serde_json::from_value::<InitializeParams>(p.clone()) {
                eprintln!(
                    "[kaido-mcp] Client: {} v{}",
                    init_params.client_info.name, init_params.client_info.version
                );
            }
        }

        self.initialized = true;

        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: false,
                }),
            },
            server_info: ServerInfo {
                name: "kaido-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }

    /// Handle tools/list request
    fn handle_tools_list(&self) -> Result<Value, JsonRpcError> {
        let definitions = self.tools.get_definitions();
        let result = ToolsListResult { tools: definitions };

        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }

    /// Handle tools/call request
    fn handle_tool_call(&self, params: &Option<Value>) -> Result<Value, JsonRpcError> {
        let params = params
            .as_ref()
            .ok_or_else(|| JsonRpcError::invalid_params("Missing params"))?;

        let call_params: ToolCallParams = serde_json::from_value(params.clone())
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        eprintln!(
            "[kaido-mcp] Tool call: {} with args: {:?}",
            call_params.name, call_params.arguments
        );

        // Execute tool call in async context
        let result = self.runtime.block_on(async {
            self.tools
                .call(&call_params.name, &call_params.arguments)
                .await
        });

        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = McpServer::new();
        assert!(!server.initialized);
    }

    #[test]
    fn test_handle_initialize() {
        let mut server = McpServer::new();

        let params = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        });

        let result = server.handle_initialize(&Some(params));
        assert!(result.is_ok());
        assert!(server.initialized);

        let value = result.unwrap();
        assert_eq!(value["serverInfo"]["name"], "kaido-mcp");
    }

    #[test]
    fn test_handle_tools_list() {
        let server = McpServer::new();
        let result = server.handle_tools_list();

        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value["tools"].is_array());
        assert!(!value["tools"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_handle_unknown_method() {
        let mut server = McpServer::new();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "unknown/method".to_string(),
            params: None,
        };

        let result = server.handle_request(&request);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.code, -32601); // Method not found
    }

    #[test]
    fn test_handle_tool_call() {
        let server = McpServer::new();

        let params = json!({
            "name": "kaido_list_tools",
            "arguments": {}
        });

        let result = server.handle_tool_call(&Some(params));
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value["content"].is_array());
        assert!(!value["isError"].as_bool().unwrap_or(true));
    }
}
