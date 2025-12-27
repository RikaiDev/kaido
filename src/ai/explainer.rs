//! Command Explainer for Explain Mode
//!
//! Generates educational explanations for commands to help users learn
//! what each command does and why it's useful.

use anyhow::Result;
use crate::tools::LLMBackend;

/// Generates educational explanations for commands
pub struct CommandExplainer;

impl CommandExplainer {
    /// Generate an educational explanation for a command
    ///
    /// # Arguments
    /// * `command` - The command to explain (e.g., "lsof -i :80 -P -n")
    /// * `tool` - The tool category (e.g., "network", "nginx", "kubectl")
    /// * `llm` - The LLM backend to use for generation
    ///
    /// # Returns
    /// A formatted explanation string suitable for terminal display
    pub async fn explain(
        command: &str,
        tool: &str,
        llm: &dyn LLMBackend,
    ) -> Result<String> {
        let prompt = Self::build_explain_prompt(command, tool);
        let response = llm.infer(&prompt).await?;

        // The LLM response comes in the reasoning field
        Ok(Self::format_explanation(&response.reasoning))
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
Focus on teaching the CONCEPT, not just describing syntax."#,
            tool = tool,
            command = command
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
