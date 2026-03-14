pub struct AIProcessor;

impl AIProcessor {
    pub fn new() -> Self {
        Self
    }

    pub fn explain_error(&self, error: &str) -> String {
        if error.contains("bind()") && error.contains(":80") {
            return "Port 80 is already in use. Another service (like Apache) may be running."
                .to_string();
        }
        if error.contains("EADDRINUSE") {
            return "Address already in use. The port is occupied by another process.".to_string();
        }
        if error.contains("Permission denied") {
            return "Permission denied. You may need sudo for this operation.".to_string();
        }
        if error.contains("command not found") {
            return "Command not found. Check if the command is installed.".to_string();
        }
        format!(
            "Error: {}. Try searching for this error message online.",
            error
        )
    }

    pub fn is_natural_language(&self, input: &str) -> bool {
        let known_commands = [
            "ls",
            "cd",
            "grep",
            "cat",
            "echo",
            "pwd",
            "rm",
            "cp",
            "mv",
            "mkdir",
            "chmod",
            "chown",
            "ps",
            "kill",
            "docker",
            "kubectl",
            "systemctl",
            "nginx",
            "apt",
            "yum",
            "pip",
            "npm",
            "node",
            "git",
            "find",
            "tar",
            "curl",
            "wget",
            "ssh",
            "sudo",
            "python",
            "python3",
            "ruby",
            "go",
            "make",
            "cmake",
        ];

        let first_word = input.split_whitespace().next().unwrap_or("");
        if known_commands.contains(&first_word) {
            return false;
        }

        input.split_whitespace().count() > 1
    }
}

#[derive(Debug)]
pub struct Translation {
    pub original: String,
    pub intent: String,
    pub command: String,
    pub explanation: String,
}

impl Translation {
    pub fn to_display_string(&self) -> String {
        format!(
            "→ Intent: {}\n→ Translate: {}\n→ {}",
            self.intent, self.command, self.explanation
        )
    }
}
