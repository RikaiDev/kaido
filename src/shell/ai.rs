pub struct AIProcessor {
    ollama_url: String,
    model: String,
}

impl AIProcessor {
    pub fn new() -> Self {
        Self {
            ollama_url: "http://localhost:11434".to_string(),
            model: "qwen2.5:1.5b".to_string(),
        }
    }

    pub fn with_model(model: &str) -> Self {
        Self {
            ollama_url: "http://localhost:11434".to_string(),
            model: model.to_string(),
        }
    }

    pub async fn explain_error_with_context(
        &self,
        cmd: &str,
        error: &str,
        diagnostics: &[crate::shell::plugin::DiagnosticContext],
        skill_context: Option<&crate::shell::skills::Skill>,
    ) -> String {
        let system_prompt = r#"You are a DevOps mentor. Your role is to GUIDE users to solve problems themselves, NOT to give them the answer directly.

Rules:
1. NEVER just explain what the error means
2. ALWAYS suggest specific diagnostic commands they should run
3. Explain WHY each diagnostic command will help
4. After they run the commands, ask what they found before suggesting next steps
5. Be brief - 3-4 sentences max
6. Use "$" prefix for commands
7. Never run commands for them - guide them to run it themselves
8. If skill knowledge is provided, use it to give more accurate guidance"#;

        let mut user_prompt = format!("Command that failed: {}\nError: {}", cmd, error);

        if let Some(skill) = skill_context {
            user_prompt.push_str("\n\nRelevant skill knowledge:");
            user_prompt.push_str(&format!("\nPattern: {}", skill.pattern));
            if !skill.causes.is_empty() {
                user_prompt.push_str("\nPossible causes:");
                for cause in &skill.causes {
                    user_prompt.push_str(&format!("\n- {}", cause));
                }
            }
            if !skill.diagnosis.is_empty() {
                user_prompt.push_str("\nDiagnosis steps:");
                for step in &skill.diagnosis {
                    user_prompt.push_str(&format!("\n- {}", step));
                }
            }
            if !skill.teaches.is_empty() {
                user_prompt.push_str("\nThings this teaches:");
                for teach in &skill.teaches {
                    user_prompt.push_str(&format!("\n- {}", teach));
                }
            }
        }

        if !diagnostics.is_empty() {
            user_prompt.push_str("\n\nDiagnostic suggestions from plugins:");
            for ctx in diagnostics {
                user_prompt.push_str(&format!("\n- {}: {}", ctx.category, ctx.explanation));
                for diag_cmd in &ctx.commands {
                    user_prompt.push_str(&format!("\n  * $ {} (to: {})", diag_cmd.cmd, diag_cmd.purpose));
                }
            }
        }

        user_prompt.push_str("\n\nWhat diagnostic command should I run first?");

        let request = serde_json::json!({
            "model": self.model,
            "system": system_prompt,
            "prompt": user_prompt,
            "stream": false
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .ok();

        if let Some(client) = client {
            if let Ok(response) = client
                .post(format!("{}/api/generate", self.ollama_url))
                .json(&request)
                .send()
                .await
            {
                if let Ok(json) = response.json::<serde_json::Value>().await {
                    if let Some(text) = json.get("response").and_then(|r| r.as_str()) {
                        // Clean markdown formatting for terminal display
                        let cleaned = text
                            .trim()
                            .replace("```bash", "$ ")
                            .replace("```", "")
                            .replace("`", "");
                        return cleaned;
                    }
                }
            }
        }

        // Fallback to pattern matching
        self.explain_error(error)
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

    pub async fn explain_error_async(&self, error: &str) -> String {
        let system_prompt = r#"You are a DevOps mentor. Explain errors to help beginners learn.
Be brief (2-3 sentences), explain WHAT went wrong and WHY, then suggest ONE command to diagnose.
Use "$" prefix for commands."#;

        let user_prompt = format!("Explain this error for a beginner:\n{}", error);

        let request = serde_json::json!({
            "model": self.model,
            "system": system_prompt,
            "prompt": user_prompt,
            "stream": false
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .ok();

        if let Some(client) = client {
            if let Ok(response) = client
                .post(format!("{}/api/generate", self.ollama_url))
                .json(&request)
                .send()
                .await
            {
                if let Ok(json) = response.json::<serde_json::Value>().await {
                    if let Some(text) = json.get("response").and_then(|r| r.as_str()) {
                        let cleaned = text
                            .trim()
                            .replace("```bash", "$ ")
                            .replace("```", "")
                            .replace("`", "");
                        return cleaned;
                    }
                }
            }
        }

        self.explain_error(error)
    }

    pub async fn explain_command(&self, cmd: &str) -> String {
        let system_prompt = r#"You are a DevOps mentor. Explain shell commands to help beginners learn.
Break down WHAT each part of the command does, explain the flags/options, and give a real-world use case.
Use "$" prefix for example commands. Keep it beginner-friendly."#;

        let user_prompt = format!("Explain this command for a beginner: {}", cmd);

        let request = serde_json::json!({
            "model": self.model,
            "system": system_prompt,
            "prompt": user_prompt,
            "stream": false
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .ok();

        if let Some(client) = client {
            if let Ok(response) = client
                .post(format!("{}/api/generate", self.ollama_url))
                .json(&request)
                .send()
                .await
            {
                if let Ok(json) = response.json::<serde_json::Value>().await {
                    if let Some(text) = json.get("response").and_then(|r| r.as_str()) {
                        let cleaned = text
                            .trim()
                            .replace("```bash", "$ ")
                            .replace("```", "")
                            .replace("`", "");
                        return cleaned;
                    }
                }
            }
        }

        format!("Could not explain command: {}. Make sure Ollama is running.", cmd)
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
