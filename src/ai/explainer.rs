//! Command Explainer for Explain Mode
//!
//! Generates educational explanations for commands to help users learn
//! what each command does and why it's useful.

use crate::tools::LLMBackend;
use anyhow::Result;

/// Generates educational explanations for commands
pub struct CommandExplainer;

impl CommandExplainer {
    /// Generate an educational explanation for a command (async with LLM)
    ///
    /// # Arguments
    /// * `command` - The command to explain (e.g., "lsof -i :80 -P -n")
    /// * `tool` - The tool category (e.g., "network", "nginx", "kubectl")
    /// * `llm` - The LLM backend to use for generation
    ///
    /// # Returns
    /// A formatted explanation string suitable for terminal display
    pub async fn explain(command: &str, tool: &str, llm: &dyn LLMBackend) -> Result<String> {
        let prompt = Self::build_explain_prompt(command, tool);
        let response = llm.infer(&prompt).await?;

        // The LLM response comes in the reasoning field
        Ok(Self::format_explanation(&response.reasoning))
    }

    /// Generate a pattern-based explanation (sync, no LLM required)
    ///
    /// Uses built-in knowledge of common commands for fast explanations.
    /// Falls back to generic description for unknown commands.
    pub fn explain_sync(command: &str, tool: &str) -> String {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let base_cmd = parts.first().copied().unwrap_or("");

        let mut explanation = String::new();

        // Base command explanation
        let base_desc = Self::get_base_command_desc(base_cmd);
        explanation.push_str(&format!("{base_cmd} = \"{base_desc}\"\n\n"));

        // Tool-specific context
        explanation.push_str(Self::get_tool_context(tool));
        explanation.push_str("\n\n");

        // Flag explanations
        let flags = Self::explain_flags(command, base_cmd);
        if !flags.is_empty() {
            explanation.push_str("Flags:\n");
            for (flag, desc) in flags {
                explanation.push_str(&format!("  {flag} -> {desc}\n"));
            }
            explanation.push('\n');
        }

        // When to use
        explanation.push_str(&format!("When to use: {}", Self::get_use_case(base_cmd)));

        explanation
    }

    fn get_base_command_desc(cmd: &str) -> &'static str {
        match cmd {
            "kubectl" => "Kubernetes command-line tool",
            "docker" => "Container management tool",
            "lsof" => "list open files",
            "ss" => "socket statistics",
            "netstat" => "network statistics",
            "curl" => "transfer data from/to server",
            "nginx" => "web server control",
            "systemctl" => "system service manager",
            "journalctl" => "query systemd journal",
            "ps" => "process status",
            "top" | "htop" => "process monitor",
            "df" => "disk space usage",
            "du" => "disk usage by directory",
            "ls" => "list directory contents",
            "cat" => "concatenate and display files",
            "grep" => "search text patterns",
            "find" => "search for files",
            "chmod" => "change file permissions",
            "chown" => "change file ownership",
            "tar" => "archive files",
            "git" => "version control system",
            "mysql" => "MySQL database client",
            "psql" => "PostgreSQL database client",
            "drush" => "Drupal command-line tool",
            _ => "command-line tool",
        }
    }

    fn get_tool_context(tool: &str) -> &'static str {
        match tool {
            "kubectl" => "Kubernetes manages containerized applications across clusters.",
            "docker" => "Docker runs applications in isolated containers.",
            "network" => "Network tools help diagnose connectivity issues.",
            "nginx" => "Nginx is a high-performance web server and reverse proxy.",
            "apache2" => "Apache is a widely-used web server.",
            "mysql" => "MySQL is a relational database management system.",
            "drush" => "Drush manages Drupal sites from the command line.",
            _ => "This tool helps manage system operations.",
        }
    }

    fn explain_flags(command: &str, base_cmd: &str) -> Vec<(String, &'static str)> {
        let mut flags = Vec::new();

        match base_cmd {
            "kubectl" => {
                if command.contains(" get ") {
                    flags.push(("get".to_string(), "retrieve resources"));
                }
                if command.contains(" describe ") {
                    flags.push(("describe".to_string(), "show detailed info"));
                }
                if command.contains(" logs ") {
                    flags.push(("logs".to_string(), "fetch container logs"));
                }
                if command.contains(" delete ") {
                    flags.push(("delete".to_string(), "remove resources"));
                }
                if command.contains(" apply ") {
                    flags.push(("apply".to_string(), "create/update resources"));
                }
                if command.contains(" -n ") {
                    flags.push(("-n".to_string(), "specify namespace"));
                }
                if command.contains(" -o wide") {
                    flags.push(("-o wide".to_string(), "show more columns"));
                }
                if command.contains(" -o yaml") {
                    flags.push(("-o yaml".to_string(), "output as YAML"));
                }
                if command.contains(" -o json") {
                    flags.push(("-o json".to_string(), "output as JSON"));
                }
                if command.contains(" -f ") {
                    flags.push(("-f".to_string(), "use file"));
                }
                if command.contains(" --all-namespaces") {
                    flags.push(("--all-namespaces".to_string(), "across all namespaces"));
                }
            }
            "docker" => {
                if command.contains(" ps") {
                    flags.push(("ps".to_string(), "list containers"));
                }
                if command.contains(" images") {
                    flags.push(("images".to_string(), "list images"));
                }
                if command.contains(" logs ") {
                    flags.push(("logs".to_string(), "fetch container logs"));
                }
                if command.contains(" exec ") {
                    flags.push(("exec".to_string(), "run command in container"));
                }
                if command.contains(" -a") {
                    flags.push(("-a".to_string(), "show all (including stopped)"));
                }
                if command.contains(" -it ") {
                    flags.push(("-it".to_string(), "interactive terminal"));
                }
                if command.contains(" -d") {
                    flags.push(("-d".to_string(), "run in background"));
                }
            }
            "lsof" => {
                if command.contains(" -i") {
                    flags.push(("-i".to_string(), "filter by network connection"));
                }
                if command.contains(" -P") {
                    flags.push(("-P".to_string(), "show port numbers (not names)"));
                }
                if command.contains(" -n") {
                    flags.push(("-n".to_string(), "skip DNS lookup (faster)"));
                }
            }
            "ss" => {
                if command.contains(" -t") {
                    flags.push(("-t".to_string(), "TCP connections only"));
                }
                if command.contains(" -l") {
                    flags.push(("-l".to_string(), "listening sockets only"));
                }
                if command.contains(" -n") {
                    flags.push(("-n".to_string(), "numeric output"));
                }
                if command.contains(" -p") {
                    flags.push(("-p".to_string(), "show process info"));
                }
            }
            "nginx" => {
                if command.contains(" -t") {
                    flags.push(("-t".to_string(), "test configuration"));
                }
                if command.contains(" -T") {
                    flags.push(("-T".to_string(), "test and dump config"));
                }
                if command.contains(" -s reload") {
                    flags.push(("-s reload".to_string(), "reload configuration"));
                }
            }
            _ => {}
        }

        flags
    }

    fn get_use_case(cmd: &str) -> &'static str {
        match cmd {
            "kubectl" => "Managing Kubernetes deployments, debugging pods, viewing logs.",
            "docker" => "Running containers, building images, managing container lifecycle.",
            "lsof" => "Finding port conflicts, identifying which process uses a file/port.",
            "ss" | "netstat" => "Checking open ports, debugging network connections.",
            "curl" => "Testing APIs, downloading files, debugging HTTP issues.",
            "nginx" => "Managing web server, testing config, reloading after changes.",
            "systemctl" => "Starting/stopping services, checking service status.",
            "journalctl" => "Reading logs, debugging service failures.",
            "ps" => "Finding running processes, checking resource usage.",
            "df" => "Checking available disk space.",
            "du" => "Finding what's using disk space.",
            "grep" => "Searching logs, filtering command output.",
            "find" => "Locating files by name, type, or age.",
            _ => "Various system administration tasks.",
        }
    }

    /// Build the prompt for explanation generation
    fn build_explain_prompt(command: &str, tool: &str) -> String {
        format!(
            r#"You are an expert ops instructor teaching a beginner who has never used a terminal before.

Explain this command in a way that teaches the user:

Tool: {tool}
Command: {command}

Your explanation should:
1. Start with what the base command means (e.g., "lsof = list open files")
2. Explain WHY this is useful (the concept behind it)
3. Break down each flag/argument with "→" arrows
4. End with "When to use:" followed by practical scenarios

Format your response EXACTLY like this example:

lsof = "list open files"

In Unix, network connections are treated as files.
This command finds which process is using a port.

Flags:
  -i :80  → filter by port 80
  -P      → show port numbers (not names)
  -n      → skip DNS lookup (faster)

When to use: Finding port conflicts, identifying
which service is listening on a port.

Keep it concise (6-10 lines max). No markdown, no code blocks.
Focus on teaching the CONCEPT, not just describing syntax."#
        )
    }

    /// Format the explanation for terminal display
    fn format_explanation(raw: &str) -> String {
        // Clean up the response - remove any markdown artifacts
        let cleaned = raw
            .trim()
            .replace("```", "")
            .replace("**", "")
            .replace("*", "");

        // Ensure consistent formatting
        let mut lines: Vec<&str> = cleaned.lines().collect();

        // Limit to reasonable length
        if lines.len() > 12 {
            lines.truncate(12);
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_explain_prompt() {
        let prompt = CommandExplainer::build_explain_prompt("lsof -i :80", "network");

        assert!(prompt.contains("lsof -i :80"));
        assert!(prompt.contains("network"));
        assert!(prompt.contains("beginner"));
    }

    #[test]
    fn test_format_explanation() {
        let raw = "```\nlsof = list open files\n\nUseful for finding ports.\n```";
        let formatted = CommandExplainer::format_explanation(raw);

        assert!(!formatted.contains("```"));
        assert!(formatted.contains("lsof"));
    }
}
