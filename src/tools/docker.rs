use super::{Tool, Translation, ExecutionResult, ToolContext, RiskLevel, LLMBackend, ErrorExplanation};
use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use std::time::Instant;

/// Docker tool implementation
pub struct DockerTool {
    docker_cli_path: PathBuf,
    compose_available: bool,
}

impl DockerTool {
    pub fn new() -> Self {
        let compose_available = which::which("docker-compose").is_ok()
            || std::process::Command::new("docker")
                .args(["compose", "version"])
                .output()
                .map(|out| out.status.success())
                .unwrap_or(false);
        
        Self {
            docker_cli_path: which::which("docker").unwrap_or_else(|_| PathBuf::from("docker")),
            compose_available,
        }
    }
    
    /// Get docker CLI path
    pub fn cli_path(&self) -> &PathBuf {
        &self.docker_cli_path
    }
    
    /// Check if docker-compose is available
    pub fn is_compose_available(&self) -> bool {
        self.compose_available
    }
    
    /// Parse docker-compose.yml to extract port mappings
    pub async fn parse_compose_ports(compose_file: &str) -> Result<Vec<PortMapping>> {
        let content = tokio::fs::read_to_string(compose_file).await?;
        let mut mappings = Vec::new();
        
        // Simple regex-based parsing (for production, use proper YAML parser)
        for line in content.lines() {
            if line.trim().starts_with("- ") && line.contains(':') {
                // Format: "- 8080:80" or "- '8080:80'"
                let port_str = line.trim().trim_start_matches("- ").trim_matches(|c| c == '"' || c == '\'');
                if let Some((host, container)) = port_str.split_once(':') {
                    if let (Ok(host_port), Ok(container_port)) = (host.parse::<u16>(), container.parse::<u16>()) {
                        mappings.push(PortMapping {
                            host_port,
                            container_port,
                            protocol: "tcp".to_string(),
                        });
                    }
                }
            }
        }
        
        Ok(mappings)
    }
    
    /// Check for port conflicts with docker-compose configuration
    pub async fn check_compose_port_conflicts(compose_file: &str) -> Result<Vec<PortConflict>> {
        let mappings = Self::parse_compose_ports(compose_file).await?;
        let mut conflicts = Vec::new();
        
        // Get currently listening ports
        let netstat_output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg("netstat -tuln 2>/dev/null || ss -tuln")
            .output()
            .await?;
        
        let netstat_str = String::from_utf8_lossy(&netstat_output.stdout);
        
        for mapping in mappings {
            let port_str = format!(":{}", mapping.host_port);
            if netstat_str.contains(&port_str) {
                // Port is in use
                conflicts.push(PortConflict {
                    port: mapping.host_port,
                    service: "unknown".to_string(),
                });
            }
        }
        
        Ok(conflicts)
    }
    
    /// Get docker-compose service status
    pub async fn get_compose_status(compose_file: Option<&str>) -> Result<String> {
        let mut cmd = tokio::process::Command::new("docker-compose");
        
        if let Some(file) = compose_file {
            cmd.args(["-f", file]);
        }
        
        cmd.arg("ps");
        
        let output = cmd.output().await?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    /// Get docker network information
    pub async fn inspect_network(network_name: &str) -> Result<String> {
        let output = tokio::process::Command::new("docker")
            .args(["network", "inspect", network_name])
            .output()
            .await?;
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// Port mapping from docker-compose
#[derive(Debug, Clone)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: String,
}

/// Port conflict information
#[derive(Debug, Clone)]
pub struct PortConflict {
    pub port: u16,
    pub service: String,
}

impl Default for DockerTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DockerTool {
    fn name(&self) -> &'static str {
        "docker"
    }
    
    fn detect_intent(&self, input: &str) -> f32 {
        let lower = input.to_lowercase();
        
        // Explicit docker command → 100%
        if lower.starts_with("docker ") || lower.contains(" docker ") {
            return 1.0;
        }
        
        // Docker keywords
        let docker_keywords = [
            "container", "image", "volume", "network", 
            "compose", "dockerfile", "registry"
        ];
        
        let matches = docker_keywords.iter()
            .filter(|k| lower.contains(*k))
            .count();
        
        if matches > 0 {
            return (matches as f32 / docker_keywords.len() as f32) * 0.8;
        }
        
        0.0
    }
    
    async fn translate(
        &self,
        input: &str,
        context: &ToolContext,
        llm: &dyn LLMBackend,
    ) -> Result<Translation> {
        let prompt = format!(r#"
Translate the following natural language to a Docker command.

User Input: {input}

Context:
- Working Directory: {pwd}
- Docker Host: {docker_host}

Common Docker operations:
- ps: list containers
- images: list images
- run: create and start container
- exec: execute command in running container
- logs: view container logs
- stop/start/restart: container lifecycle
- rm/rmi: remove containers/images
- build: build image from Dockerfile
- pull/push: registry operations

Output JSON format:
{{
  "command": "exact docker command",
  "confidence": 0-100,
  "reasoning": "explanation"
}}
"#,
            input = input,
            pwd = context.working_directory.display(),
            docker_host = context.docker_host.as_deref().unwrap_or("default"),
        );
        
        let result = llm.infer(&prompt).await?;
        
        Ok(Translation {
            command: result.command,
            confidence: result.confidence,
            reasoning: result.reasoning,
            tool_name: "docker".to_string(),
            requires_files: vec![],
        })
    }
    
    fn classify_risk(&self, command: &str, context: &ToolContext) -> RiskLevel {
        let cmd = command.to_lowercase();
        
        // Log Docker host if configured
        if let Some(docker_host) = &context.docker_host {
            log::debug!("Docker command targeting host: {}", docker_host);
        }
        
        // CRITICAL: Batch deletion with command substitution
        if cmd.contains("rm") && (cmd.contains("$(") || cmd.contains("`")) {
            return RiskLevel::Critical;
        }
        
        // HIGH: Deletion operations
        if cmd.contains(" rm ") || cmd.contains(" rmi ") 
            || cmd.contains("system prune")
            || cmd.contains("volume rm")
            || cmd.contains("network rm") {
            return RiskLevel::High;
        }
        
        // MEDIUM: State-modifying operations
        if cmd.contains(" run ") || cmd.contains(" create ")
            || cmd.contains(" restart ") || cmd.contains(" stop ")
            || cmd.contains(" kill ") || cmd.contains(" build ")
            || cmd.contains(" push ") {
            return RiskLevel::Medium;
        }
        
        // LOW: Read-only operations
        RiskLevel::Low
    }
    
    async fn execute(&self, command: &str) -> Result<ExecutionResult> {
        let start = Instant::now();
        
        // Parse command into parts
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty command"));
        }
        
        // Use docker_cli_path for execution
        let docker_cmd = if parts[0] == "docker" {
            self.docker_cli_path.as_os_str()
        } else {
            std::ffi::OsStr::new(&parts[0])
        };
        
        // Execute command
        let output = tokio::process::Command::new(docker_cmd)
            .args(&parts[1..])
            .output()
            .await?;
        
        let duration = start.elapsed();
        
        Ok(ExecutionResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration,
        })
    }
    
    fn explain_error(&self, error: &str) -> Option<ErrorExplanation> {
        // Use PatternMatcher for intelligent error matching
        let matcher = crate::error::PatternMatcher::new();
        matcher.match_pattern(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_detection() {
        let tool = DockerTool::new();
        
        assert_eq!(tool.detect_intent("docker ps"), 1.0);
        assert_eq!(tool.detect_intent("list containers"), 0.11428572); // 1/7 keywords
        assert!(tool.detect_intent("show images") > 0.0);
        assert_eq!(tool.detect_intent("kubectl get pods"), 0.0);
    }

    #[test]
    fn test_docker_risk_classification() {
        let tool = DockerTool::new();
        let ctx = ToolContext::default();
        
        assert_eq!(
            tool.classify_risk("docker ps", &ctx),
            RiskLevel::Low
        );
        
        assert_eq!(
            tool.classify_risk("docker rm nginx", &ctx),
            RiskLevel::High
        );
        
        assert_eq!(
            tool.classify_risk("docker rm $(docker ps -aq)", &ctx),
            RiskLevel::Critical
        );
        
        assert_eq!(
            tool.classify_risk("docker run nginx", &ctx),
            RiskLevel::Medium
        );
    }
}

