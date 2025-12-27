// Kaido MCP Tools
// Exposes Kaido capabilities as MCP tools for Claude Code integration

use super::types::{ToolCallResult, ToolDefinition};
use crate::ai::CommandExplainer;
use crate::kubectl::EnvironmentType;
use crate::tools::{RiskLevel, ToolContext, ToolRegistry};
use serde_json::{json, Value};
use std::process::Command;

/// Kaido MCP tool handler
pub struct KaidoTools {
    registry: ToolRegistry,
}

impl KaidoTools {
    pub fn new() -> Self {
        Self {
            registry: ToolRegistry::new(),
        }
    }

    /// Get all tool definitions
    pub fn get_definitions(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "kaido_diagnose".to_string(),
                description: "Diagnose a DevOps problem by analyzing the situation and suggesting solutions. \
                              Kaido will use its AI-powered diagnosis to understand the issue and provide \
                              step-by-step guidance.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "problem": {
                            "type": "string",
                            "description": "Description of the problem to diagnose (e.g., 'nginx is returning 502', 'pod keeps crashing')"
                        }
                    },
                    "required": ["problem"]
                }),
            },
            ToolDefinition {
                name: "kaido_execute".to_string(),
                description: "Execute a command using a specific Kaido tool (kubectl, docker, nginx, network, etc.). \
                              Returns the command output.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The command to execute"
                        },
                        "tool": {
                            "type": "string",
                            "description": "Tool name (kubectl, docker, nginx, apache2, network, mysql, drush)",
                            "enum": ["kubectl", "docker", "nginx", "apache2", "network", "mysql", "drush", "shell"]
                        }
                    },
                    "required": ["command"]
                }),
            },
            ToolDefinition {
                name: "kaido_explain".to_string(),
                description: "Get an educational explanation of a command. Breaks down the command into \
                              components, explains flags and arguments, and provides context on when to use it.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The command to explain (e.g., 'kubectl get pods -n kube-system')"
                        }
                    },
                    "required": ["command"]
                }),
            },
            ToolDefinition {
                name: "kaido_get_context".to_string(),
                description: "Get current system context including Kubernetes cluster, Docker status, \
                              and available tools. Useful for understanding the environment before taking actions.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
            ToolDefinition {
                name: "kaido_list_tools".to_string(),
                description: "List all available Kaido tools and their capabilities.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
            ToolDefinition {
                name: "kaido_check_risk".to_string(),
                description: "Assess the risk level of a command before execution. Returns LOW, MEDIUM, HIGH, or CRITICAL \
                              with an explanation of potential impacts.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The command to assess"
                        },
                        "tool": {
                            "type": "string",
                            "description": "Tool name (optional, auto-detected if not provided)"
                        }
                    },
                    "required": ["command"]
                }),
            },
        ]
    }

    /// Handle a tool call
    pub async fn call(&self, name: &str, arguments: &Value) -> ToolCallResult {
        match name {
            "kaido_diagnose" => self.diagnose(arguments).await,
            "kaido_execute" => self.execute(arguments).await,
            "kaido_explain" => self.explain(arguments).await,
            "kaido_get_context" => self.get_context().await,
            "kaido_list_tools" => self.list_tools(),
            "kaido_check_risk" => self.check_risk(arguments),
            _ => ToolCallResult::error(format!("Unknown tool: {}", name)),
        }
    }

    /// Diagnose a problem
    async fn diagnose(&self, arguments: &Value) -> ToolCallResult {
        let problem = arguments
            .get("problem")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if problem.is_empty() {
            return ToolCallResult::error("Missing required parameter: problem");
        }

        // Build diagnostic information
        let mut diagnosis = String::new();
        diagnosis.push_str(&format!("# Kaido Diagnosis: {}\n\n", problem));

        // Detect relevant tool
        if let Some(tool) = self.registry.detect_tool(problem) {
            diagnosis.push_str(&format!("**Detected Tool:** {}\n\n", tool.name()));
        }

        // Run some diagnostic commands based on keywords
        let diagnostics = self.get_diagnostic_commands(problem);

        if !diagnostics.is_empty() {
            diagnosis.push_str("## Diagnostic Results\n\n");

            for (cmd_name, cmd) in diagnostics {
                diagnosis.push_str(&format!("### {}\n", cmd_name));
                diagnosis.push_str(&format!("```\n$ {}\n", cmd));

                match self.run_command(&cmd) {
                    Ok(output) => {
                        let truncated = if output.len() > 2000 {
                            format!("{}...\n(truncated)", &output[..2000])
                        } else {
                            output
                        };
                        diagnosis.push_str(&truncated);
                    }
                    Err(e) => {
                        diagnosis.push_str(&format!("Error: {}", e));
                    }
                }
                diagnosis.push_str("\n```\n\n");
            }
        }

        // Provide suggestions
        diagnosis.push_str("## Suggested Next Steps\n\n");
        diagnosis.push_str(&self.get_suggestions(problem));

        ToolCallResult::success(diagnosis)
    }

    /// Execute a command
    async fn execute(&self, arguments: &Value) -> ToolCallResult {
        let command = arguments
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if command.is_empty() {
            return ToolCallResult::error("Missing required parameter: command");
        }

        let tool_name = arguments
            .get("tool")
            .and_then(|v| v.as_str());

        // Validate tool if specified
        if let Some(name) = tool_name {
            if name != "shell" && self.registry.get_tool(name).is_none() {
                return ToolCallResult::error(format!("Unknown tool: {}", name));
            }
        }

        // Check risk level first
        let risk = self.assess_risk(command, tool_name);

        if matches!(risk, RiskLevel::Critical) {
            return ToolCallResult::error(format!(
                "Command has CRITICAL risk level and cannot be auto-executed.\n\
                 Command: {}\n\n\
                 This command could cause significant damage. Please review carefully \
                 and execute manually if intended.",
                command
            ));
        }

        // Execute the command
        match self.run_command(command) {
            Ok(output) => {
                let result = format!(
                    "$ {}\n\n{}",
                    command,
                    if output.is_empty() { "(no output)" } else { &output }
                );
                ToolCallResult::success(result)
            }
            Err(e) => ToolCallResult::error(format!("Execution failed: {}", e)),
        }
    }

    /// Explain a command
    async fn explain(&self, arguments: &Value) -> ToolCallResult {
        let command = arguments
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if command.is_empty() {
            return ToolCallResult::error("Missing required parameter: command");
        }

        // Get tool name from command
        let tool_name = command
            .split_whitespace()
            .next()
            .unwrap_or("unknown");

        // Generate explanation using pattern-based explainer
        let explanation = CommandExplainer::explain_sync(command, tool_name);

        ToolCallResult::success(explanation)
    }

    /// Get system context
    async fn get_context(&self) -> ToolCallResult {
        let mut context = String::new();
        context.push_str("# Kaido System Context\n\n");

        // Kubernetes context
        context.push_str("## Kubernetes\n");
        if let Ok(output) = self.run_command("kubectl config current-context") {
            context.push_str(&format!("- Current Context: `{}`\n", output.trim()));
        } else {
            context.push_str("- Kubernetes: Not configured or kubectl not found\n");
        }

        if let Ok(output) = self.run_command("kubectl config view --minify -o jsonpath='{.contexts[0].context.namespace}'") {
            let ns = output.trim().trim_matches('\'');
            if !ns.is_empty() {
                context.push_str(&format!("- Default Namespace: `{}`\n", ns));
            }
        }

        // Docker status
        context.push_str("\n## Docker\n");
        if let Ok(output) = self.run_command("docker info --format '{{.ServerVersion}}'") {
            context.push_str(&format!("- Docker Version: `{}`\n", output.trim().trim_matches('\'')));

            if let Ok(containers) = self.run_command("docker ps -q | wc -l") {
                context.push_str(&format!("- Running Containers: {}\n", containers.trim()));
            }
        } else {
            context.push_str("- Docker: Not running or not installed\n");
        }

        // System info
        context.push_str("\n## System\n");
        context.push_str(&format!("- Working Directory: `{}`\n",
            std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "unknown".to_string())
        ));
        context.push_str(&format!("- User: `{}`\n",
            users::get_current_username()
                .and_then(|u| u.into_string().ok())
                .unwrap_or_else(|| "unknown".to_string())
        ));

        // Available tools
        context.push_str("\n## Available Kaido Tools\n");
        for tool in self.registry.list_tools() {
            context.push_str(&format!("- `{}`\n", tool));
        }

        ToolCallResult::success(context)
    }

    /// List available tools
    fn list_tools(&self) -> ToolCallResult {
        let mut output = String::new();
        output.push_str("# Kaido Available Tools\n\n");

        let tools_info = vec![
            ("kubectl", "Kubernetes cluster management - pods, deployments, services, etc."),
            ("docker", "Container management - images, containers, networks, volumes"),
            ("nginx", "Nginx web server - config testing, reload, status"),
            ("apache2", "Apache web server - config testing, modules, status"),
            ("network", "Network diagnostics - ports, connections, DNS, curl"),
            ("mysql", "MySQL database operations - queries, status, variables"),
            ("drush", "Drupal management - cache, config, database operations"),
        ];

        for (name, desc) in tools_info {
            let available = self.registry.get_tool(name).is_some();
            let status = if available { "available" } else { "registered" };
            output.push_str(&format!("## {}\n", name));
            output.push_str(&format!("- **Status:** {}\n", status));
            output.push_str(&format!("- **Description:** {}\n\n", desc));
        }

        ToolCallResult::success(output)
    }

    /// Check risk level of a command
    fn check_risk(&self, arguments: &Value) -> ToolCallResult {
        let command = arguments
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if command.is_empty() {
            return ToolCallResult::error("Missing required parameter: command");
        }

        let tool_name = arguments
            .get("tool")
            .and_then(|v| v.as_str());

        let risk = self.assess_risk(command, tool_name);
        let ctx = ToolContext::default();

        let mut output = String::new();
        output.push_str(&format!("# Risk Assessment\n\n"));
        output.push_str(&format!("**Command:** `{}`\n\n", command));
        output.push_str(&format!("**Risk Level:** {}\n\n", risk.as_str()));

        output.push_str("## Risk Explanation\n\n");
        match risk {
            RiskLevel::Low => {
                output.push_str("This is a **read-only** operation that does not modify any state.\n");
                output.push_str("- Safe to execute automatically\n");
                output.push_str("- No confirmation required\n");
            }
            RiskLevel::Medium => {
                output.push_str("This operation **modifies state** but can typically be reversed.\n");
                output.push_str("- Requires confirmation before execution\n");
                output.push_str("- Changes can usually be rolled back\n");
            }
            RiskLevel::High => {
                output.push_str("This is a **destructive** operation that may be difficult to reverse.\n");
                output.push_str("- Requires explicit confirmation\n");
                output.push_str("- Consider backup before proceeding\n");
                if ctx.kubectl_context.as_ref().map(|k| k.environment_type == EnvironmentType::Production).unwrap_or(false) {
                    output.push_str("- **WARNING:** Production environment detected!\n");
                }
            }
            RiskLevel::Critical => {
                output.push_str("This is a **critical** operation with potentially severe consequences.\n");
                output.push_str("- Requires typed confirmation (type the resource name)\n");
                output.push_str("- Cannot be automatically executed\n");
                output.push_str("- Strongly recommend backup and review\n");
            }
        }

        ToolCallResult::success(output)
    }

    // Helper methods

    fn run_command(&self, command: &str) -> Result<String, String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty command".to_string());
        }

        let output = Command::new(parts[0])
            .args(&parts[1..])
            .output()
            .map_err(|e| format!("Failed to execute: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            Ok(stdout.to_string())
        } else if !stderr.is_empty() {
            Ok(format!("{}\n{}", stdout, stderr))
        } else {
            Ok(stdout.to_string())
        }
    }

    fn assess_risk(&self, command: &str, tool_name: Option<&str>) -> RiskLevel {
        let ctx = ToolContext::default();

        // Try to get tool from name or detect from command
        let tool = tool_name
            .and_then(|name| self.registry.get_tool(name))
            .or_else(|| self.registry.detect_tool(command));

        if let Some(t) = tool {
            t.classify_risk(command, &ctx)
        } else {
            // Default risk assessment for unknown commands
            let cmd_lower = command.to_lowercase();
            if cmd_lower.contains("rm ") || cmd_lower.contains("delete") ||
               cmd_lower.contains("drop ") || cmd_lower.contains("truncate") {
                RiskLevel::High
            } else if cmd_lower.contains("update") || cmd_lower.contains("insert") ||
                      cmd_lower.contains("create") || cmd_lower.contains("modify") {
                RiskLevel::Medium
            } else {
                RiskLevel::Low
            }
        }
    }

    fn get_diagnostic_commands(&self, problem: &str) -> Vec<(&'static str, String)> {
        let problem_lower = problem.to_lowercase();
        let mut commands = Vec::new();

        // Kubernetes diagnostics
        if problem_lower.contains("pod") || problem_lower.contains("kubernetes") ||
           problem_lower.contains("k8s") || problem_lower.contains("deployment") {
            commands.push(("Pod Status", "kubectl get pods --all-namespaces".to_string()));
            if problem_lower.contains("crash") || problem_lower.contains("restart") {
                commands.push(("Recent Events", "kubectl get events --sort-by=.lastTimestamp | tail -20".to_string()));
            }
        }

        // Nginx diagnostics
        if problem_lower.contains("nginx") || problem_lower.contains("502") ||
           problem_lower.contains("504") || problem_lower.contains("web server") {
            commands.push(("Nginx Status", "systemctl status nginx".to_string()));
            commands.push(("Nginx Config Test", "nginx -t".to_string()));
        }

        // Docker diagnostics
        if problem_lower.contains("docker") || problem_lower.contains("container") {
            commands.push(("Docker Containers", "docker ps -a".to_string()));
            commands.push(("Docker System", "docker system df".to_string()));
        }

        // Network diagnostics
        if problem_lower.contains("port") || problem_lower.contains("connection") ||
           problem_lower.contains("network") || problem_lower.contains("bind") {
            commands.push(("Listening Ports", "ss -tlnp".to_string()));
        }

        // If no specific diagnostics, provide general system info
        if commands.is_empty() {
            commands.push(("System Load", "uptime".to_string()));
            commands.push(("Disk Usage", "df -h".to_string()));
        }

        commands
    }

    fn get_suggestions(&self, problem: &str) -> String {
        let problem_lower = problem.to_lowercase();
        let mut suggestions = String::new();

        if problem_lower.contains("502") || problem_lower.contains("bad gateway") {
            suggestions.push_str("1. Check if the upstream service is running\n");
            suggestions.push_str("2. Verify nginx proxy_pass configuration\n");
            suggestions.push_str("3. Check upstream service logs\n");
            suggestions.push_str("4. Verify network connectivity between nginx and upstream\n");
        } else if problem_lower.contains("crash") || problem_lower.contains("restart") {
            suggestions.push_str("1. Check pod logs: `kubectl logs <pod-name> --previous`\n");
            suggestions.push_str("2. Describe pod for events: `kubectl describe pod <pod-name>`\n");
            suggestions.push_str("3. Check resource limits (OOMKilled?)\n");
            suggestions.push_str("4. Verify liveness/readiness probes\n");
        } else if problem_lower.contains("port") && problem_lower.contains("use") {
            suggestions.push_str("1. Find process using port: `lsof -i :<port>`\n");
            suggestions.push_str("2. Kill the process or use a different port\n");
            suggestions.push_str("3. Check for zombie processes\n");
        } else {
            suggestions.push_str("1. Review the diagnostic results above\n");
            suggestions.push_str("2. Check relevant service logs\n");
            suggestions.push_str("3. Verify configuration files\n");
            suggestions.push_str("4. Test connectivity to dependencies\n");
        }

        suggestions
    }
}

impl Default for KaidoTools {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::ToolContent;

    #[test]
    fn test_tool_definitions() {
        let tools = KaidoTools::new();
        let definitions = tools.get_definitions();

        assert_eq!(definitions.len(), 6);

        let names: Vec<_> = definitions.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"kaido_diagnose"));
        assert!(names.contains(&"kaido_execute"));
        assert!(names.contains(&"kaido_explain"));
        assert!(names.contains(&"kaido_get_context"));
        assert!(names.contains(&"kaido_list_tools"));
        assert!(names.contains(&"kaido_check_risk"));
    }

    #[test]
    fn test_risk_assessment() {
        let tools = KaidoTools::new();

        // Low risk
        assert!(matches!(
            tools.assess_risk("kubectl get pods", None),
            RiskLevel::Low
        ));

        // High risk
        assert!(matches!(
            tools.assess_risk("kubectl delete pod nginx", None),
            RiskLevel::High
        ));

        // Unknown command with delete
        assert!(matches!(
            tools.assess_risk("some-tool delete everything", None),
            RiskLevel::High
        ));
    }

    #[test]
    fn test_list_tools() {
        let tools = KaidoTools::new();
        let result = tools.list_tools();

        assert!(!result.is_error);
        if let ToolContent::Text { text } = &result.content[0] {
            assert!(text.contains("kubectl"));
            assert!(text.contains("docker"));
            assert!(text.contains("nginx"));
        }
    }
}
