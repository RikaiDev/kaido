// Mentor Engine
//
// Core engine that generates educational guidance for errors.
// Uses pattern matching first (fast), falls back to LLM for unknown errors.

use std::path::PathBuf;

use super::cache::GuidanceCache;
use super::display::MentorDisplay;
use super::guidance::{GuidanceSource, MentorGuidance, NextStep};
use super::llm_fallback::LLMMentor;
use super::types::{ErrorInfo, ErrorType};
use crate::tools::LLMBackend;

/// Configuration for the mentor engine
#[derive(Debug, Clone)]
pub struct MentorConfig {
    /// Enable LLM fallback for unknown errors
    pub enable_llm: bool,
    /// Cache database path (None = in-memory)
    pub cache_path: Option<PathBuf>,
    /// Cache retention in days
    pub cache_retention_days: u32,
}

impl Default for MentorConfig {
    fn default() -> Self {
        Self {
            enable_llm: true,
            cache_path: dirs::home_dir().map(|h| h.join(".kaido").join("mentor_cache.db")),
            cache_retention_days: 30,
        }
    }
}

/// The main mentor engine
pub struct MentorEngine {
    config: MentorConfig,
    cache: Option<GuidanceCache>,
    display: MentorDisplay,
}

impl MentorEngine {
    /// Create a new mentor engine with default config
    pub fn new() -> Self {
        Self::with_config(MentorConfig::default())
    }

    /// Create a new mentor engine with custom config
    pub fn with_config(config: MentorConfig) -> Self {
        // Try to create cache
        let cache = config
            .cache_path
            .as_ref()
            .and_then(|path| {
                // Ensure parent directory exists
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                GuidanceCache::new(path).ok()
            })
            .or_else(|| GuidanceCache::in_memory().ok());

        // Clean old cache entries
        if let Some(ref cache) = cache {
            let _ = cache.clean_old_entries(config.cache_retention_days);
        }

        Self {
            config,
            cache,
            display: MentorDisplay::new(),
        }
    }

    /// Generate guidance for an error (pattern matching only, sync)
    pub fn generate_sync(&self, error: &ErrorInfo) -> MentorGuidance {
        // 1. Check cache first
        if let Some(ref cache) = self.cache {
            if let Some(cached) = cache.get(error) {
                log::debug!("Mentor guidance cache hit for: {}", error.key_message);
                return cached;
            }
        }

        // 2. Use pattern-based guidance
        self.generate_from_pattern(error)
    }

    /// Generate guidance for an error (with LLM fallback, async)
    pub async fn generate(
        &self,
        error: &ErrorInfo,
        llm: Option<&dyn LLMBackend>,
    ) -> MentorGuidance {
        // 1. Check cache first
        if let Some(ref cache) = self.cache {
            if let Some(cached) = cache.get(error) {
                log::debug!("Mentor guidance cache hit for: {}", error.key_message);
                return cached;
            }
        }

        // 2. Try pattern matching
        let pattern_guidance = self.generate_from_pattern(error);

        // 3. If pattern matched well, use it
        if pattern_guidance.source == GuidanceSource::Pattern
            && !pattern_guidance.explanation.is_empty()
            && !pattern_guidance.next_steps.is_empty()
        {
            return pattern_guidance;
        }

        // 4. Try LLM fallback if enabled and available
        if self.config.enable_llm {
            if let Some(llm) = llm {
                log::info!(
                    "Using LLM fallback for unknown error: {}",
                    error.key_message
                );
                match LLMMentor::generate(error, llm).await {
                    Ok(guidance) => {
                        // Cache the LLM response
                        if let Some(ref cache) = self.cache {
                            let _ = cache.set(error, &guidance);
                        }
                        return guidance;
                    }
                    Err(e) => {
                        log::warn!("LLM fallback failed: {e}");
                    }
                }
            }
        }

        // 5. Return pattern guidance (might be generic fallback)
        pattern_guidance
    }

    /// Generate guidance from built-in patterns
    fn generate_from_pattern(&self, error: &ErrorInfo) -> MentorGuidance {
        match error.error_type {
            ErrorType::CommandNotFound => self.guidance_command_not_found(error),
            ErrorType::PermissionDenied => self.guidance_permission_denied(error),
            ErrorType::FileNotFound => self.guidance_file_not_found(error),
            ErrorType::ConnectionRefused => self.guidance_connection_refused(error),
            ErrorType::PortInUse => self.guidance_port_in_use(error),
            ErrorType::ConfigurationError => self.guidance_configuration_error(error),
            ErrorType::SyntaxError => self.guidance_syntax_error(error),
            ErrorType::DependencyError => self.guidance_dependency_error(error),
            ErrorType::DockerError => self.guidance_docker_error(error),
            ErrorType::KubernetesError => self.guidance_kubernetes_error(error),
            ErrorType::GitError => self.guidance_git_error(error),
            _ => self.guidance_generic(error),
        }
    }

    // Pattern-specific guidance generators

    fn guidance_command_not_found(&self, error: &ErrorInfo) -> MentorGuidance {
        let cmd = Self::extract_command_name(&error.key_message);

        MentorGuidance::from_pattern(
            &error.key_message,
            format!(
                "The command '{cmd}' is not installed on this system, or it's not in your PATH."
            ),
        )
        .with_search(vec![
            format!("install {} macos", cmd),
            format!("install {} linux", cmd),
        ])
        .with_steps(vec![
            NextStep::with_command("Check if it's installed somewhere", format!("which {cmd}")),
            NextStep::with_command("Install on macOS", format!("brew install {cmd}")),
            NextStep::with_command(
                "Install on Ubuntu/Debian",
                format!("sudo apt install {cmd}"),
            ),
            NextStep::with_command("Check your PATH", "echo $PATH"),
        ])
        .with_concepts(vec![
            "PATH environment variable".to_string(),
            "Package managers (brew, apt)".to_string(),
        ])
    }

    fn guidance_permission_denied(&self, error: &ErrorInfo) -> MentorGuidance {
        MentorGuidance::from_pattern(
            &error.key_message,
            "You don't have permission to perform this action. This usually means you need \
             elevated privileges (sudo) or the file/directory permissions need to be changed.",
        )
        .with_search(vec![
            "linux file permissions".to_string(),
            "chmod tutorial".to_string(),
        ])
        .with_steps(vec![
            NextStep::with_command("Run with sudo (if appropriate)", "sudo !!"),
            NextStep::with_command("Check file permissions", "ls -la <file>"),
            NextStep::with_command("Make file executable", "chmod +x <file>"),
            NextStep::new("Check file ownership with 'ls -la'"),
        ])
        .with_concepts(vec![
            "Unix file permissions".to_string(),
            "sudo and root access".to_string(),
            "File ownership".to_string(),
        ])
    }

    fn guidance_file_not_found(&self, error: &ErrorInfo) -> MentorGuidance {
        MentorGuidance::from_pattern(
            &error.key_message,
            "The specified file or directory doesn't exist. Check the path for typos \
             or verify the file was created.",
        )
        .with_search(vec![
            "find file linux".to_string(),
            "bash tab completion".to_string(),
        ])
        .with_steps(vec![
            NextStep::with_command("List current directory", "ls -la"),
            NextStep::with_command("Show working directory", "pwd"),
            NextStep::with_command("Search for file", "find . -name '<filename>'"),
            NextStep::new("Use tab completion to verify paths"),
        ])
        .with_concepts(vec![
            "File paths (absolute vs relative)".to_string(),
            "Working directory".to_string(),
        ])
    }

    fn guidance_connection_refused(&self, error: &ErrorInfo) -> MentorGuidance {
        MentorGuidance::from_pattern(
            &error.key_message,
            "The connection was refused. The service might not be running, \
             or a firewall could be blocking the connection.",
        )
        .with_search(vec![
            "check if service is running linux".to_string(),
            "netstat listening ports".to_string(),
        ])
        .with_steps(vec![
            NextStep::with_command("Check if service is running", "systemctl status <service>"),
            NextStep::with_command("List listening ports", "netstat -tuln"),
            NextStep::with_command("Check firewall (Ubuntu)", "sudo ufw status"),
            NextStep::new("Verify the host and port are correct"),
        ])
        .with_concepts(vec![
            "Network ports and services".to_string(),
            "Systemd service management".to_string(),
        ])
    }

    fn guidance_port_in_use(&self, error: &ErrorInfo) -> MentorGuidance {
        MentorGuidance::from_pattern(
            &error.key_message,
            "Another process is already using this port. You'll need to stop that process \
             or use a different port.",
        )
        .with_search(vec![
            "find process using port".to_string(),
            "kill process linux".to_string(),
        ])
        .with_steps(vec![
            NextStep::with_command("Find process using port", "lsof -i :<port>"),
            NextStep::with_command("Or use netstat", "netstat -tuln | grep <port>"),
            NextStep::with_command("Kill process by PID", "kill <pid>"),
            NextStep::new("Or configure your service to use a different port"),
        ])
        .with_concepts(vec![
            "Network ports".to_string(),
            "Process management".to_string(),
        ])
    }

    fn guidance_configuration_error(&self, error: &ErrorInfo) -> MentorGuidance {
        let location = error
            .source_location
            .as_ref()
            .map(|l| l.to_string())
            .unwrap_or_else(|| "configuration file".to_string());

        MentorGuidance::from_pattern(
            &error.key_message,
            format!(
                "There's an error in {location}. Check the file for typos or invalid directives."
            ),
        )
        .with_search(vec!["configuration syntax".to_string()])
        .with_steps(if let Some(ref loc) = error.source_location {
            let file = loc.file.display().to_string();
            let line = loc.line.unwrap_or(1);
            vec![
                NextStep::with_command("Open file at error line", format!("vim {file} +{line}")),
                NextStep::new("Check for typos in the directive name"),
                NextStep::new("Verify syntax matches documentation"),
            ]
        } else {
            vec![
                NextStep::new("Check the configuration file for syntax errors"),
                NextStep::new("Compare with documentation examples"),
            ]
        })
        .with_concepts(vec!["Configuration file syntax".to_string()])
    }

    fn guidance_syntax_error(&self, error: &ErrorInfo) -> MentorGuidance {
        MentorGuidance::from_pattern(
            &error.key_message,
            "There's a syntax error. Check for missing quotes, brackets, or typos.",
        )
        .with_search(vec!["syntax error".to_string()])
        .with_steps(vec![
            NextStep::new("Check for missing or mismatched quotes"),
            NextStep::new("Check for missing brackets or parentheses"),
            NextStep::new("Look for typos in keywords"),
        ])
        .with_concepts(vec!["Syntax and parsing".to_string()])
    }

    fn guidance_dependency_error(&self, error: &ErrorInfo) -> MentorGuidance {
        MentorGuidance::from_pattern(
            &error.key_message,
            "A required module or dependency is missing. You may need to install it.",
        )
        .with_search(vec!["install dependency".to_string()])
        .with_steps(vec![
            NextStep::with_command("For Node.js", "npm install"),
            NextStep::with_command("For Python", "pip install -r requirements.txt"),
            NextStep::with_command("For Rust", "cargo build"),
            NextStep::new("Check if the module name is spelled correctly"),
        ])
        .with_concepts(vec![
            "Package managers".to_string(),
            "Dependencies".to_string(),
        ])
    }

    fn guidance_docker_error(&self, error: &ErrorInfo) -> MentorGuidance {
        MentorGuidance::from_pattern(
            &error.key_message,
            "A Docker error occurred. Check if Docker is running and the image/container exists.",
        )
        .with_search(vec![
            "docker troubleshooting".to_string(),
            "docker common errors".to_string(),
        ])
        .with_steps(vec![
            NextStep::with_command("Check Docker status", "docker info"),
            NextStep::with_command("List containers", "docker ps -a"),
            NextStep::with_command("List images", "docker images"),
            NextStep::with_command("View container logs", "docker logs <container>"),
        ])
        .with_concepts(vec![
            "Docker containers".to_string(),
            "Docker images".to_string(),
        ])
    }

    fn guidance_kubernetes_error(&self, error: &ErrorInfo) -> MentorGuidance {
        MentorGuidance::from_pattern(
            &error.key_message,
            "A Kubernetes error occurred. Check the resource name, namespace, and cluster connection.",
        )
        .with_search(vec![
            "kubernetes debugging".to_string(),
            "kubectl troubleshooting".to_string(),
        ])
        .with_steps(vec![
            NextStep::with_command("Check cluster connection", "kubectl cluster-info"),
            NextStep::with_command("List resources in namespace", "kubectl get all"),
            NextStep::with_command("Check all namespaces", "kubectl get all -A"),
            NextStep::with_command("Describe resource", "kubectl describe <resource> <name>"),
        ])
        .with_concepts(vec![
            "Kubernetes namespaces".to_string(),
            "Kubernetes resources".to_string(),
        ])
    }

    fn guidance_git_error(&self, error: &ErrorInfo) -> MentorGuidance {
        MentorGuidance::from_pattern(
            &error.key_message,
            "A Git error occurred. Check your repository state and remote configuration.",
        )
        .with_search(vec![
            "git common errors".to_string(),
            "git troubleshooting".to_string(),
        ])
        .with_steps(vec![
            NextStep::with_command("Check repository status", "git status"),
            NextStep::with_command("View recent commits", "git log --oneline -5"),
            NextStep::with_command("Check remotes", "git remote -v"),
            NextStep::with_command("Check branches", "git branch -a"),
        ])
        .with_concepts(vec!["Git workflow".to_string(), "Git remotes".to_string()])
    }

    fn guidance_generic(&self, error: &ErrorInfo) -> MentorGuidance {
        MentorGuidance::fallback(&error.key_message).with_steps(vec![
            NextStep::new("Check the full error output above"),
            NextStep::new("Search for the error message online"),
        ])
    }

    /// Extract command name from error message
    fn extract_command_name(msg: &str) -> String {
        // Look for common patterns
        // "command not found: foo"
        // "foo: command not found"
        // "bash: foo: command not found"

        // Split by colons and find the command
        let parts: Vec<&str> = msg.split(':').map(|s| s.trim()).collect();

        // Pattern: "bash: foo: command not found" -> foo is between shell name and error
        if parts.len() >= 3 {
            // Check if last part contains "command not found" or "not found"
            if let Some(last) = parts.last() {
                if last.contains("command not found") || last.contains("not found") {
                    // Return the part before the error message
                    // Skip first part if it's a shell name (bash, zsh, sh)
                    let skip = if parts[0] == "bash" || parts[0] == "zsh" || parts[0] == "sh" {
                        1
                    } else {
                        0
                    };
                    if let Some(cmd) = parts.get(skip) {
                        return cmd
                            .split_whitespace()
                            .next()
                            .unwrap_or("command")
                            .to_string();
                    }
                }
            }
        }

        // Pattern: "command not found: foo"
        if parts.len() == 2 {
            if parts[0].contains("command not found") {
                return parts[1]
                    .split_whitespace()
                    .next()
                    .unwrap_or("command")
                    .to_string();
            }
            // Pattern: "foo: command not found"
            if parts[1].contains("command not found") || parts[1].contains("not found") {
                return parts[0]
                    .split_whitespace()
                    .last()
                    .unwrap_or("command")
                    .to_string();
            }
        }

        // Fallback: last word
        msg.split_whitespace()
            .last()
            .unwrap_or("command")
            .to_string()
    }

    /// Render guidance as formatted output
    pub fn render(&self, guidance: &MentorGuidance) -> String {
        self.display.render_guidance(guidance)
    }
}

impl Default for MentorEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_error(error_type: ErrorType, key_message: &str) -> ErrorInfo {
        ErrorInfo::new(error_type, 1, key_message, "test command")
    }

    #[test]
    fn test_engine_creation() {
        let engine = MentorEngine::new();
        assert!(engine.config.enable_llm);
    }

    #[test]
    fn test_command_not_found_guidance() {
        let engine = MentorEngine::new();
        let error = create_test_error(ErrorType::CommandNotFound, "command not found: kubectl");

        let guidance = engine.generate_sync(&error);

        assert_eq!(guidance.source, GuidanceSource::Pattern);
        assert!(!guidance.next_steps.is_empty());
        assert!(guidance.explanation.contains("kubectl"));
    }

    #[test]
    fn test_permission_denied_guidance() {
        let engine = MentorEngine::new();
        let error = create_test_error(ErrorType::PermissionDenied, "Permission denied");

        let guidance = engine.generate_sync(&error);

        assert!(guidance.explanation.contains("permission"));
        assert!(guidance
            .next_steps
            .iter()
            .any(|s| s.command.as_ref().is_some_and(|c| c.contains("sudo"))));
    }

    #[test]
    fn test_unknown_error_fallback() {
        let engine = MentorEngine::new();
        let error = create_test_error(ErrorType::Unknown, "some unknown error");

        let guidance = engine.generate_sync(&error);

        assert_eq!(guidance.source, GuidanceSource::Fallback);
    }

    #[test]
    fn test_extract_command_name() {
        assert_eq!(
            MentorEngine::extract_command_name("command not found: kubectl"),
            "kubectl"
        );
        assert_eq!(
            MentorEngine::extract_command_name("bash: foo: command not found"),
            "foo"
        );
        assert_eq!(
            MentorEngine::extract_command_name("docker: not found"),
            "docker"
        );
    }

    #[test]
    fn test_cache_integration() {
        let config = MentorConfig {
            cache_path: None, // In-memory
            ..Default::default()
        };
        let engine = MentorEngine::with_config(config);
        let error = create_test_error(ErrorType::CommandNotFound, "command not found: test");

        // First call
        let guidance1 = engine.generate_sync(&error);
        assert_eq!(guidance1.source, GuidanceSource::Pattern);

        // Cache the result manually for testing
        if let Some(ref cache) = engine.cache {
            cache.set(&error, &guidance1).unwrap();
        }

        // Second call should hit cache
        let guidance2 = engine.generate_sync(&error);
        assert_eq!(guidance2.source, GuidanceSource::Cached);
    }
}
