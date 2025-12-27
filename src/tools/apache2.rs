use anyhow::Result;
use async_trait::async_trait;
use std::time::Instant;

use super::{Tool, ToolContext, Translation, ExecutionResult, RiskLevel, LLMBackend, ErrorExplanation, Solution};

/// Apache2/httpd web server tool
pub struct Apache2Tool;

impl Apache2Tool {
    pub fn new() -> Self {
        Self
    }
    
    /// Check if apache2/httpd is installed
    pub async fn is_installed() -> bool {
        // Try both apache2 (Debian/Ubuntu) and httpd (RHEL/CentOS)
        let apache2_check = tokio::process::Command::new("which")
            .arg("apache2")
            .output()
            .await
            .map(|out| out.status.success())
            .unwrap_or(false);
        
        if apache2_check {
            return true;
        }
        
        tokio::process::Command::new("which")
            .arg("httpd")
            .output()
            .await
            .map(|out| out.status.success())
            .unwrap_or(false)
    }
    
    /// Get apache command (apache2 or httpd)
    async fn get_apache_cmd() -> &'static str {
        if tokio::process::Command::new("which")
            .arg("apache2")
            .output()
            .await
            .map(|out| out.status.success())
            .unwrap_or(false)
        {
            "apache2"
        } else {
            "httpd"
        }
    }
    
    /// Validate apache configuration
    pub async fn validate_config() -> Result<String> {
        let cmd = Self::get_apache_cmd().await;
        let output = tokio::process::Command::new(if cmd == "apache2" { "apache2ctl" } else { "apachectl" })
            .args(["configtest"])
            .output()
            .await?;
        
        Ok(format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
    
    /// Get apache version
    pub async fn get_version() -> Result<String> {
        let cmd = Self::get_apache_cmd().await;
        let output = tokio::process::Command::new(cmd)
            .args(["-v"])
            .output()
            .await?;
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    /// Get loaded modules
    pub async fn get_modules() -> Result<String> {
        let cmd = Self::get_apache_cmd().await;
        let output = tokio::process::Command::new(if cmd == "apache2" { "apache2ctl" } else { "apachectl" })
            .args(["-M"])
            .output()
            .await?;
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    /// Get virtual hosts configuration
    pub async fn get_vhosts() -> Result<String> {
        let cmd = Self::get_apache_cmd().await;
        let output = tokio::process::Command::new(if cmd == "apache2" { "apache2ctl" } else { "apachectl" })
            .args(["-S"])
            .output()
            .await?;
        
        Ok(format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

impl Default for Apache2Tool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for Apache2Tool {
    fn name(&self) -> &'static str {
        "apache2"
    }
    
    fn detect_intent(&self, input: &str) -> f32 {
        let input_lower = input.to_lowercase();
        
        if input_lower.contains("apache") || input_lower.contains("httpd") {
            return 1.0;
        }
        
        if input_lower.contains("virtualhost") || input_lower.contains("vhost") {
            return 0.8;
        }
        
        0.0
    }
    
    async fn translate(
        &self,
        input: &str,
        _context: &ToolContext,
        llm: &dyn LLMBackend,
    ) -> Result<Translation> {
        let prompt = format!(
            "Translate this natural language request into an apache2/httpd command.\n\
            User request: {}\n\n\
            Common apache2 commands:\n\
            - apache2ctl configtest (test configuration)\n\
            - apache2ctl -M (list modules)\n\
            - apache2ctl -S (list virtual hosts)\n\
            - systemctl status apache2 (check status)\n\n\
            Respond ONLY with JSON:\n\
            {{\"command\": \"apache2ctl configtest\", \"confidence\": 90, \"reasoning\": \"Test apache configuration\"}}\n\n\
            Your response:",
            input
        );
        
        let llm_response = llm.infer(&prompt).await?;
        
        #[derive(serde::Deserialize)]
        struct ApacheResponse {
            command: String,
            confidence: u8,
            reasoning: String,
        }
        
        let parsed: ApacheResponse = serde_json::from_str(&llm_response.reasoning)
            .unwrap_or(ApacheResponse {
                command: llm_response.command.clone(),
                confidence: llm_response.confidence,
                reasoning: llm_response.reasoning.clone(),
            });
        
        Ok(Translation {
            command: parsed.command,
            confidence: parsed.confidence,
            reasoning: parsed.reasoning,
            tool_name: "apache2".to_string(),
            requires_files: vec![],
        })
    }
    
    fn classify_risk(&self, command: &str, _context: &ToolContext) -> RiskLevel {
        let cmd_lower = command.to_lowercase();
        
        // Read-only commands
        if cmd_lower.contains("configtest")
            || cmd_lower.contains("-v")
            || cmd_lower.contains("-M")
            || cmd_lower.contains("-S")
            || cmd_lower.contains("status")
        {
            return RiskLevel::Low;
        }
        
        // Graceful reload
        if cmd_lower.contains("graceful") {
            return RiskLevel::Medium;
        }
        
        // Restart/reload
        if cmd_lower.contains("restart") || cmd_lower.contains("reload") {
            return RiskLevel::Medium;
        }
        
        // Stop
        if cmd_lower.contains("stop") {
            return RiskLevel::High;
        }
        
        // Uninstall or dangerous operations
        if cmd_lower.contains("remove") || cmd_lower.contains("purge") || cmd_lower.contains("uninstall") {
            return RiskLevel::Critical;
        }
        
        RiskLevel::Medium
    }
    
    async fn execute(&self, command: &str) -> Result<ExecutionResult> {
        let start = Instant::now();
        
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
        
        if error_lower.contains("address already in use") || error_lower.contains("bind") {
            return Some(ErrorExplanation {
                error_type: "Port Conflict".to_string(),
                reason: "Apache is trying to use a port that's already in use".to_string(),
                possible_causes: vec![
                    "Another web server (nginx, another Apache instance) is running".to_string(),
                    "Different application using the same port".to_string(),
                ],
                solutions: vec![
                    Solution {
                        description: "Check what's using the port".to_string(),
                        command: Some("lsof -i :80 -P -n".to_string()),
                        risk_level: RiskLevel::Low,
                    },
                    Solution {
                        description: "Stop Apache and check for zombie processes".to_string(),
                        command: Some("systemctl stop apache2 && ps aux | grep apache2".to_string()),
                        risk_level: RiskLevel::Medium,
                    },
                ],
                recommended_solution: 0,
                documentation_links: vec!["https://httpd.apache.org/docs/".to_string()],
            });
        }
        
        if error_lower.contains("syntax error") {
            return Some(ErrorExplanation {
                error_type: "Configuration Syntax Error".to_string(),
                reason: "Apache configuration has syntax errors".to_string(),
                possible_causes: vec![
                    "Missing or misplaced directives".to_string(),
                    "Invalid VirtualHost configuration".to_string(),
                ],
                solutions: vec![
                    Solution {
                        description: "Test configuration".to_string(),
                        command: Some("apache2ctl configtest".to_string()),
                        risk_level: RiskLevel::Low,
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
        let tool = Apache2Tool::new();
        assert_eq!(tool.detect_intent("apache2 status"), 1.0);
        assert_eq!(tool.detect_intent("check virtualhost"), 0.8);
        assert_eq!(tool.detect_intent("kubectl get pods"), 0.0);
    }
    
    #[test]
    fn test_classify_risk() {
        let tool = Apache2Tool::new();
        let ctx = ToolContext::default();
        
        assert_eq!(tool.classify_risk("apache2ctl configtest", &ctx), RiskLevel::Low);
        assert_eq!(tool.classify_risk("systemctl restart apache2", &ctx), RiskLevel::Medium);
        assert_eq!(tool.classify_risk("systemctl stop apache2", &ctx), RiskLevel::High);
    }
}

