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
        ];

        let first_word = input.split_whitespace().next().unwrap_or("");
        if known_commands.contains(&first_word) {
            return false;
        }

        input.split_whitespace().count() > 1
    }
}
