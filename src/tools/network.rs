use anyhow::Result;
use async_trait::async_trait;
use std::time::Instant;

use super::{Tool, ToolContext, Translation, ExecutionResult, RiskLevel, LLMBackend, ErrorExplanation, Solution};

/// Network diagnostic tool
/// Provides network troubleshooting commands: netstat, ss, lsof, iptables, ufw, etc.
pub struct NetworkTool;

impl NetworkTool {
    pub fn new() -> Self {
        Self
    }
    
    /// Get all listening TCP ports
    pub async fn get_listening_ports() -> Result<String> {
        // Try ss first (modern), fallback to netstat
        let ss_output = tokio::process::Command::new("ss")
            .args(["-tlnp"])
            .output()
            .await;
        
        if let Ok(output) = ss_output {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
        
        // Fallback to netstat
        let netstat_output = tokio::process::Command::new("netstat")
            .args(["-tlnp"])
            .output()
            .await?;
        
        Ok(String::from_utf8_lossy(&netstat_output.stdout).to_string())
    }
    
    /// Check specific port usage
    pub async fn check_port(port: u16) -> Result<String> {
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "lsof -i :{} -P -n 2>/dev/null || ss -tlnp | grep :{} || netstat -tlnp | grep :{}",
                port, port, port
            ))
            .output()
            .await?;
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    /// Get firewall status (iptables or ufw)
    pub async fn get_firewall_status() -> Result<String> {
        // Try ufw first (Ubuntu/Debian)
        let ufw_output = tokio::process::Command::new("ufw")
            .args(["status", "verbose"])
            .output()
            .await;
        
        if let Ok(output) = ufw_output {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
        
        // Fallback to iptables
        let iptables_output = tokio::process::Command::new("iptables")
            .args(["-L", "-n", "-v"])
            .output()
            .await?;
        
        Ok(String::from_utf8_lossy(&iptables_output.stdout).to_string())
    }
    
    /// Test TCP connection to host:port
    pub async fn test_connection(host: &str, port: u16) -> Result<String> {
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "timeout 5 bash -c 'cat < /dev/null > /dev/tcp/{}/{}' && echo 'Connection successful' || echo 'Connection failed'",
                host, port
            ))
            .output()
            .await?;
        
        Ok(format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
    
    /// Get network interfaces
    pub async fn get_interfaces() -> Result<String> {
        let output = tokio::process::Command::new("ip")
            .args(["addr", "show"])
            .output()
            .await?;
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    /// DNS lookup
    pub async fn dns_lookup(domain: &str) -> Result<String> {
        // Try dig first, fallback to nslookup
        let dig_output = tokio::process::Command::new("dig")
            .args(["+short", domain])
            .output()
            .await;
        
        if let Ok(output) = dig_output {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
        
        let nslookup_output = tokio::process::Command::new("nslookup")
            .arg(domain)
            .output()
            .await?;
        
        Ok(String::from_utf8_lossy(&nslookup_output.stdout).to_string())
    }
}

impl Default for NetworkTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for NetworkTool {
    fn name(&self) -> &'static str {
        "network"
    }
    
    fn detect_intent(&self, input: &str) -> f32 {
        let input_lower = input.to_lowercase();
        
        let network_keywords = [
            "netstat", "ss ", "lsof", "port", "firewall", "iptables", "ufw",
            "connection", "network", "ping", "telnet", "nc ", "netcat",
            "dns", "nslookup", "dig", "route", "ip addr",
        ];
        
        for keyword in &network_keywords {
            if input_lower.contains(keyword) {
                return 0.9;
            }
        }
        
        // Port-related keywords
        if input_lower.contains("listening") || input_lower.contains("bind") {
            return 0.7;
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
            "Translate this natural language request into a network diagnostic command.\n\
            User request: {}\n\n\
            Common network commands:\n\
            - ss -tlnp (show listening TCP ports)\n\
            - netstat -tuln (show all TCP/UDP connections)\n\
            - lsof -i :PORT (check what's using a port)\n\
            - iptables -L (list firewall rules)\n\
            - ufw status (check UFW firewall)\n\
            - ping HOST (test connectivity)\n\
            - dig DOMAIN (DNS lookup)\n\
            - ip addr show (show network interfaces)\n\n\
            Respond ONLY with JSON:\n\
            {{\"command\": \"ss -tlnp\", \"confidence\": 90, \"reasoning\": \"Check listening ports\"}}\n\n\
            Your response:",
            input
        );
        
        let llm_response = llm.infer(&prompt).await?;
        
        #[derive(serde::Deserialize)]
        struct NetworkResponse {
            command: String,
            confidence: u8,
            reasoning: String,
        }
        
        let parsed: NetworkResponse = serde_json::from_str(&llm_response.reasoning)
            .unwrap_or(NetworkResponse {
                command: llm_response.command.clone(),
                confidence: llm_response.confidence,
                reasoning: llm_response.reasoning.clone(),
            });
        
        Ok(Translation {
            command: parsed.command,
            confidence: parsed.confidence,
            reasoning: parsed.reasoning,
            tool_name: "network".to_string(),
            requires_files: vec![],
        })
    }
    
    fn classify_risk(&self, command: &str, _context: &ToolContext) -> RiskLevel {
        let cmd_lower = command.to_lowercase();
        
        // Read-only diagnostic commands
        if cmd_lower.contains("netstat")
            || cmd_lower.contains("ss ")
            || cmd_lower.contains("lsof")
            || cmd_lower.contains("ip addr")
            || cmd_lower.contains("ip route")
            || cmd_lower.contains("ping")
            || cmd_lower.contains("dig")
            || cmd_lower.contains("nslookup")
            || cmd_lower.contains("iptables -L")
            || cmd_lower.contains("ufw status")
        {
            return RiskLevel::Low;
        }
        
        // Firewall rule modifications (high risk)
        if cmd_lower.contains("iptables -A")
            || cmd_lower.contains("iptables -D")
            || cmd_lower.contains("iptables -I")
            || cmd_lower.contains("ufw allow")
            || cmd_lower.contains("ufw deny")
        {
            return RiskLevel::High;
        }
        
        // Dangerous firewall operations (critical)
        if cmd_lower.contains("iptables -F")
            || cmd_lower.contains("iptables --flush")
            || cmd_lower.contains("ufw disable")
            || cmd_lower.contains("ufw reset")
        {
            return RiskLevel::Critical;
        }
        
        // Network interface modifications
        if cmd_lower.contains("ip link set")
            || cmd_lower.contains("ifconfig")
            || cmd_lower.contains("ip route add")
        {
            return RiskLevel::High;
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
        
        if error_lower.contains("connection refused") {
            return Some(ErrorExplanation {
                error_type: "Connection Refused".to_string(),
                reason: "Target service is not listening on the specified port".to_string(),
                possible_causes: vec![
                    "Service is not running".to_string(),
                    "Service is listening on a different port".to_string(),
                    "Firewall is blocking the connection".to_string(),
                ],
                solutions: vec![
                    Solution {
                        description: "Check if service is running".to_string(),
                        command: Some("systemctl status <service>".to_string()),
                        risk_level: RiskLevel::Low,
                    },
                    Solution {
                        description: "Check listening ports".to_string(),
                        command: Some("ss -tlnp".to_string()),
                        risk_level: RiskLevel::Low,
                    },
                ],
                recommended_solution: 0,
                documentation_links: vec![],
            });
        }
        
        if error_lower.contains("network is unreachable") || error_lower.contains("no route to host") {
            return Some(ErrorExplanation {
                error_type: "Network Unreachable".to_string(),
                reason: "Cannot reach the target host".to_string(),
                possible_causes: vec![
                    "Network interface is down".to_string(),
                    "Routing table is incorrect".to_string(),
                    "Firewall is blocking traffic".to_string(),
                ],
                solutions: vec![
                    Solution {
                        description: "Check network interfaces".to_string(),
                        command: Some("ip addr show".to_string()),
                        risk_level: RiskLevel::Low,
                    },
                    Solution {
                        description: "Check routing table".to_string(),
                        command: Some("ip route show".to_string()),
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
        let tool = NetworkTool::new();
        assert_eq!(tool.detect_intent("check listening ports"), 0.9);
        assert_eq!(tool.detect_intent("netstat -tuln"), 0.9);
        assert_eq!(tool.detect_intent("kubectl get pods"), 0.0);
    }
    
    #[test]
    fn test_classify_risk() {
        let tool = NetworkTool::new();
        let ctx = ToolContext::default();
        
        assert_eq!(tool.classify_risk("netstat -tuln", &ctx), RiskLevel::Low);
        assert_eq!(tool.classify_risk("ss -tlnp", &ctx), RiskLevel::Low);
        assert_eq!(tool.classify_risk("iptables -A INPUT -p tcp --dport 22 -j ACCEPT", &ctx), RiskLevel::High);
        assert_eq!(tool.classify_risk("iptables -F", &ctx), RiskLevel::Critical);
        assert_eq!(tool.classify_risk("ufw disable", &ctx), RiskLevel::Critical);
    }
}

