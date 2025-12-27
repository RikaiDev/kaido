use anyhow::Result;
use async_trait::async_trait;
use std::time::Instant;

use super::{Tool, ToolContext, Translation, ExecutionResult, RiskLevel, LLMBackend, ErrorExplanation, Solution};

/// Nginx web server tool
pub struct NginxTool;

impl NginxTool {
    pub fn new() -> Self {
        Self
    }
    
    /// Check if nginx is installed
    pub async fn is_installed() -> bool {
        tokio::process::Command::new("which")
            .arg("nginx")
            .output()
            .await
            .map(|out| out.status.success())
            .unwrap_or(false)
    }
    
    /// Validate nginx configuration
    pub async fn validate_config() -> Result<String> {
        let output = tokio::process::Command::new("nginx")
            .args(["-t"])
            .output()
            .await?;
        
        Ok(format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
    
    /// Get nginx version
    pub async fn get_version() -> Result<String> {
        let output = tokio::process::Command::new("nginx")
            .args(["-v"])
            .output()
            .await?;
        
        Ok(String::from_utf8_lossy(&output.stderr).to_string())
    }
    
    /// Check nginx status via systemctl
    pub async fn check_status() -> Result<String> {
        let output = tokio::process::Command::new("systemctl")
            .args(["status", "nginx"])
            .output()
            .await?;
        
        Ok(format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
    
    /// Get listening ports for nginx
    pub async fn get_listening_ports() -> Result<Vec<u16>> {
        let output = tokio::process::Command::new("ss")
            .args(["-tlnp"])
            .output()
            .await?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut ports = Vec::new();
        
        for line in stdout.lines() {
            if line.contains("nginx") {
                // Parse port from lines like "0.0.0.0:80" or "*:443"
                if let Some(port_str) = line.split_whitespace()
                    .find(|s| s.contains(':'))
                    .and_then(|s| s.split(':').last())
                {
                    if let Ok(port) = port_str.parse::<u16>() {
                        ports.push(port);
                    }
                }
            }
        }
        
        Ok(ports)
    }
    
    /// Check what's using a specific port
    pub async fn check_port_usage(port: u16) -> Result<String> {
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(format!("lsof -i :{} -P -n || ss -tlnp | grep :{}", port, port))
            .output()
            .await?;
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    /// Read nginx error log (last N lines)
    pub async fn read_error_log(lines: usize) -> Result<String> {
        let log_paths = vec![
            "/var/log/nginx/error.log",
            "/usr/local/var/log/nginx/error.log",
            "/opt/homebrew/var/log/nginx/error.log",
        ];
        
        for path in log_paths {
            let output = tokio::process::Command::new("tail")
                .args(["-n", &lines.to_string(), path])
                .output()
                .await;
            
            if let Ok(out) = output {
                if out.status.success() {
                    return Ok(String::from_utf8_lossy(&out.stdout).to_string());
                }
            }
        }
        
        Err(anyhow::anyhow!("Could not find nginx error log"))
    }
}

impl Default for NginxTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for NginxTool {
    fn name(&self) -> &'static str {
        "nginx"
    }
    
    fn detect_intent(&self, input: &str) -> f32 {
        let input_lower = input.to_lowercase();
        
        // Exact match keywords
        if input_lower.contains("nginx") {
            return 1.0;
        }
        
        // Common nginx operations
        let nginx_keywords = [
            "web server",
            "reverse proxy",
            "http server",
            "port 80",
            "port 443",
            "ssl certificate",
        ];
        
        for keyword in &nginx_keywords {
            if input_lower.contains(keyword) {
                return 0.7;
            }
        }
        
        0.0
    }
    
    async fn translate(
        &self,
        input: &str,
        _context: &ToolContext,
        llm: &dyn LLMBackend,
    ) -> Result<Translation> {
        // Build prompt for nginx command translation
        let prompt = format!(
            "Translate this natural language request into an nginx-related command.\n\
            User request: {}\n\n\
            Common nginx commands:\n\
            - nginx -t (test configuration)\n\
            - nginx -s reload (reload configuration)\n\
            - systemctl status nginx (check status)\n\
            - systemctl start/stop/restart nginx (control service)\n\
            - nginx -V (show version and configuration)\n\n\
            Respond ONLY with JSON:\n\
            {{\"command\": \"nginx -t\", \"confidence\": 90, \"reasoning\": \"Testing nginx configuration\"}}\n\n\
            Your response:",
            input
        );
        
        let llm_response = llm.infer(&prompt).await?;
        
        // Parse JSON response
        #[derive(serde::Deserialize)]
        struct NginxResponse {
            command: String,
            confidence: u8,
            reasoning: String,
        }
        
        let parsed: NginxResponse = serde_json::from_str(&llm_response.reasoning)
            .unwrap_or(NginxResponse {
                command: llm_response.command.clone(),
                confidence: llm_response.confidence,
                reasoning: llm_response.reasoning.clone(),
            });
        
        Ok(Translation {
            command: parsed.command,
            confidence: parsed.confidence,
            reasoning: parsed.reasoning,
            tool_name: "nginx".to_string(),
            requires_files: vec![],
        })
    }
    
    fn classify_risk(&self, command: &str, _context: &ToolContext) -> RiskLevel {
        let cmd_lower = command.to_lowercase();
        
        // Read-only/diagnostic commands
        if cmd_lower.contains("nginx -t")
            || cmd_lower.contains("nginx -v")
            || cmd_lower.contains("nginx -V")
            || cmd_lower.contains("status nginx")
            || cmd_lower.contains("cat /")
            || cmd_lower.contains("tail ")
        {
            return RiskLevel::Low;
        }
        
        // Reload (medium risk - no downtime but config changes)
        if cmd_lower.contains("reload") {
            return RiskLevel::Medium;
        }
        
        // Start/restart (medium-high risk - potential downtime)
        if cmd_lower.contains("start") || cmd_lower.contains("restart") {
            return RiskLevel::Medium;
        }
        
        // Stop (high risk - service downtime)
        if cmd_lower.contains("stop") {
            return RiskLevel::High;
        }
        
        // Uninstall or force operations (critical)
        if cmd_lower.contains("remove")
            || cmd_lower.contains("purge")
            || cmd_lower.contains("uninstall")
            || cmd_lower.contains("-f ")
            || cmd_lower.contains("--force")
        {
            return RiskLevel::Critical;
        }
        
        // Default to medium for unknown nginx commands
        RiskLevel::Medium
    }
    
    async fn execute(&self, command: &str) -> Result<ExecutionResult> {
        let start = Instant::now();
        
        // Execute command via shell
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
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
        let error_lower = error.to_lowercase();
        
        // Port already in use
        if error_lower.contains("address already in use")
            || error_lower.contains("bind() to 0.0.0.0:")
        {
            return Some(ErrorExplanation {
                error_type: "Port Conflict".to_string(),
                reason: "The port nginx is trying to use is already occupied by another process".to_string(),
                possible_causes: vec![
                    "Another web server (Apache, another nginx instance) is running".to_string(),
                    "A different application is using the same port".to_string(),
                    "Previous nginx process didn't shut down cleanly".to_string(),
                ],
                solutions: vec![
                    Solution {
                        description: "Check what's using the port".to_string(),
                        command: Some("lsof -i :80 -P -n".to_string()),
                        risk_level: RiskLevel::Low,
                    },
                    Solution {
                        description: "Stop conflicting service (if Apache)".to_string(),
                        command: Some("systemctl stop apache2".to_string()),
                        risk_level: RiskLevel::High,
                    },
                    Solution {
                        description: "Change nginx port in configuration".to_string(),
                        command: Some("# Edit /etc/nginx/sites-available/default and change 'listen 80' to 'listen 8080'".to_string()),
                        risk_level: RiskLevel::Medium,
                    },
                ],
                recommended_solution: 0,
                documentation_links: vec![
                    "https://nginx.org/en/docs/".to_string(),
                ],
            });
        }
        
        // Configuration syntax error
        if error_lower.contains("syntax") || error_lower.contains("unexpected") {
            return Some(ErrorExplanation {
                error_type: "Configuration Syntax Error".to_string(),
                reason: "Nginx configuration file contains syntax errors".to_string(),
                possible_causes: vec![
                    "Missing semicolon at end of directive".to_string(),
                    "Unclosed braces or quotes".to_string(),
                    "Invalid directive name or parameters".to_string(),
                ],
                solutions: vec![
                    Solution {
                        description: "Test configuration to see exact error".to_string(),
                        command: Some("nginx -t".to_string()),
                        risk_level: RiskLevel::Low,
                    },
                    Solution {
                        description: "Check nginx error log for details".to_string(),
                        command: Some("tail -50 /var/log/nginx/error.log".to_string()),
                        risk_level: RiskLevel::Low,
                    },
                ],
                recommended_solution: 0,
                documentation_links: vec![
                    "https://nginx.org/en/docs/beginners_guide.html".to_string(),
                ],
            });
        }
        
        // Permission denied
        if error_lower.contains("permission denied") {
            return Some(ErrorExplanation {
                error_type: "Permission Error".to_string(),
                reason: "Nginx doesn't have permission to access a file or directory".to_string(),
                possible_causes: vec![
                    "Nginx worker process user doesn't have read permissions".to_string(),
                    "SELinux or AppArmor blocking access".to_string(),
                    "File ownership is incorrect".to_string(),
                ],
                solutions: vec![
                    Solution {
                        description: "Check nginx worker user".to_string(),
                        command: Some("ps aux | grep nginx".to_string()),
                        risk_level: RiskLevel::Low,
                    },
                    Solution {
                        description: "Fix file permissions".to_string(),
                        command: Some("# chmod 644 /path/to/file && chown nginx:nginx /path/to/file".to_string()),
                        risk_level: RiskLevel::Medium,
                    },
                ],
                recommended_solution: 0,
                documentation_links: vec![],
            });
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_intent() {
        let tool = NginxTool::new();
        
        assert_eq!(tool.detect_intent("nginx status"), 1.0);
        assert_eq!(tool.detect_intent("check web server"), 0.7);
        assert_eq!(tool.detect_intent("kubectl get pods"), 0.0);
    }
    
    #[test]
    fn test_classify_risk() {
        let tool = NginxTool::new();
        let ctx = ToolContext::default();
        
        assert_eq!(tool.classify_risk("nginx -t", &ctx), RiskLevel::Low);
        assert_eq!(tool.classify_risk("nginx -s reload", &ctx), RiskLevel::Medium);
        assert_eq!(tool.classify_risk("systemctl stop nginx", &ctx), RiskLevel::High);
        assert_eq!(tool.classify_risk("apt remove nginx", &ctx), RiskLevel::Critical);
    }
    
    #[test]
    fn test_explain_port_conflict() {
        let tool = NginxTool::new();
        let error = "bind() to 0.0.0.0:80 failed (98: Address already in use)";
        
        let explanation = tool.explain_error(error);
        assert!(explanation.is_some());
        
        let exp = explanation.unwrap();
        assert_eq!(exp.error_type, "Port Conflict");
        assert!(!exp.solutions.is_empty());
    }
}

