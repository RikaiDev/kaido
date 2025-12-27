use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub cmd: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSequence {
    pub task: String,
    pub commands: Vec<Command>,
}

/// Parse LLM JSON output into CommandSequence
pub fn parse_llm_output(output: &str) -> Result<CommandSequence, serde_json::Error> {
    serde_json::from_str(output)
}

/// Fallback: Convert single command string to CommandSequence
pub fn single_command_fallback(cmd: &str) -> CommandSequence {
    CommandSequence {
        task: "Execute command".to_string(),
        commands: vec![Command {
            cmd: cmd.to_string(),
            description: "Direct command execution".to_string(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_json() {
        let json = r#"{"task":"list files","commands":[{"cmd":"ls -la","description":"list all files"}]}"#;
        let result = parse_llm_output(json);
        assert!(result.is_ok());
        let seq = result.unwrap();
        assert_eq!(seq.task, "list files");
        assert_eq!(seq.commands.len(), 1);
        assert_eq!(seq.commands[0].cmd, "ls -la");
    }

    #[test]
    fn test_single_command_fallback() {
        let seq = single_command_fallback("ls -la");
        assert_eq!(seq.commands.len(), 1);
        assert_eq!(seq.commands[0].cmd, "ls -la");
    }
}

