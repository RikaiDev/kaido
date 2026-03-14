pub struct AIProcessor;

impl AIProcessor {
    pub fn new() -> Self {
        Self
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
