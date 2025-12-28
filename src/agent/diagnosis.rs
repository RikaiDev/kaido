use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context containing all diagnostic information collected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemContext {
    /// Original problem description
    pub problem_description: String,

    /// Service/component involved
    pub service: Option<String>,

    /// Environment (production, staging, development)
    pub environment: Option<String>,

    /// Diagnostic data collected (key: source, value: data)
    pub diagnostic_data: HashMap<String, String>,

    /// Error messages observed
    pub error_messages: Vec<String>,

    /// System state snapshots
    pub system_state: SystemState,

    /// Identified root cause
    pub root_cause: Option<RootCause>,
}

impl ProblemContext {
    pub fn new(problem_description: String) -> Self {
        Self {
            problem_description,
            service: None,
            environment: None,
            diagnostic_data: HashMap::new(),
            error_messages: Vec::new(),
            system_state: SystemState::default(),
            root_cause: None,
        }
    }

    /// Add diagnostic data from a tool/command
    pub fn add_diagnostic_data(&mut self, source: String, data: String) {
        self.diagnostic_data.insert(source, data);
    }

    /// Add error message
    pub fn add_error(&mut self, error: String) {
        if !self.error_messages.contains(&error) {
            self.error_messages.push(error);
        }
    }

    /// Set identified root cause
    pub fn set_root_cause(&mut self, root_cause: RootCause) {
        self.root_cause = Some(root_cause);
    }
}

/// System state information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemState {
    /// Running services
    pub running_services: Vec<String>,

    /// Port bindings (port -> process)
    pub port_bindings: HashMap<u16, String>,

    /// Resource usage
    pub disk_usage: Option<String>,
    pub memory_usage: Option<String>,
    pub cpu_usage: Option<String>,

    /// Network connectivity
    pub network_interfaces: Vec<String>,

    /// Active containers (Docker)
    pub containers: Vec<ContainerInfo>,
}

/// Container information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub ports: Vec<String>,
}

/// Identified root cause of a problem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCause {
    /// Category of the root cause
    pub category: RootCauseCategory,

    /// Human-readable description
    pub description: String,

    /// Evidence supporting this root cause
    pub evidence: Vec<String>,

    /// Confidence level (0-100)
    pub confidence: u8,

    /// Related services/components
    pub affected_components: Vec<String>,
}

/// Categories of root causes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RootCauseCategory {
    /// Port already in use
    PortConflict,

    /// Configuration error
    ConfigurationError,

    /// Permission denied
    PermissionError,

    /// Service not running
    ServiceDown,

    /// Network connectivity issue
    NetworkIssue,

    /// Resource exhaustion (disk, memory, CPU)
    ResourceExhaustion,

    /// Dependency missing or failed
    DependencyFailure,

    /// Invalid credentials
    AuthenticationFailure,

    /// Other/unknown
    Unknown,
}

impl RootCauseCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            RootCauseCategory::PortConflict => "Port Conflict",
            RootCauseCategory::ConfigurationError => "Configuration Error",
            RootCauseCategory::PermissionError => "Permission Error",
            RootCauseCategory::ServiceDown => "Service Down",
            RootCauseCategory::NetworkIssue => "Network Issue",
            RootCauseCategory::ResourceExhaustion => "Resource Exhaustion",
            RootCauseCategory::DependencyFailure => "Dependency Failure",
            RootCauseCategory::AuthenticationFailure => "Authentication Failure",
            RootCauseCategory::Unknown => "Unknown",
        }
    }
}

/// Strategy for diagnosing different types of problems
pub trait DiagnosisStrategy: Send + Sync {
    /// Name of this diagnosis strategy
    fn name(&self) -> &'static str;

    /// Check if this strategy applies to the given problem
    fn applies_to(&self, problem: &ProblemContext) -> bool;

    /// Get list of diagnostic commands to run
    fn diagnostic_commands(&self, problem: &ProblemContext) -> Vec<DiagnosticCommand>;

    /// Analyze collected data and identify root cause
    fn analyze(&self, problem: &ProblemContext) -> Option<RootCause>;
}

/// Diagnostic command to execute
#[derive(Debug, Clone)]
pub struct DiagnosticCommand {
    /// Tool to use
    pub tool: String,

    /// Command to execute
    pub command: String,

    /// Description of what this checks
    pub purpose: String,

    /// Whether this is safe to auto-execute (no destructive operations)
    pub safe_to_auto_execute: bool,
}

impl DiagnosticCommand {
    pub fn new(tool: &str, command: &str, purpose: &str) -> Self {
        Self {
            tool: tool.to_string(),
            command: command.to_string(),
            purpose: purpose.to_string(),
            safe_to_auto_execute: true,
        }
    }
}

/// Root cause analyzer - combines multiple strategies
pub struct RootCauseAnalyzer {
    strategies: Vec<Box<dyn DiagnosisStrategy>>,
}

impl RootCauseAnalyzer {
    pub fn new() -> Self {
        Self {
            strategies: vec![
                Box::new(PortConflictStrategy),
                Box::new(ServiceDownStrategy),
                Box::new(ConfigErrorStrategy),
            ],
        }
    }

    /// Find applicable strategies for a problem
    pub fn get_applicable_strategies(
        &self,
        problem: &ProblemContext,
    ) -> Vec<&dyn DiagnosisStrategy> {
        self.strategies
            .iter()
            .filter(|s| s.applies_to(problem))
            .map(|s| s.as_ref())
            .collect()
    }

    /// Get all diagnostic commands from applicable strategies
    pub fn get_diagnostic_commands(&self, problem: &ProblemContext) -> Vec<DiagnosticCommand> {
        let mut commands = Vec::new();

        for strategy in self.get_applicable_strategies(problem) {
            commands.extend(strategy.diagnostic_commands(problem));
        }

        commands
    }

    /// Analyze problem with all applicable strategies and return best root cause
    pub fn analyze(&self, problem: &ProblemContext) -> Option<RootCause> {
        let mut root_causes = Vec::new();

        for strategy in self.get_applicable_strategies(problem) {
            if let Some(cause) = strategy.analyze(problem) {
                root_causes.push(cause);
            }
        }

        // Return root cause with highest confidence
        root_causes.into_iter().max_by_key(|c| c.confidence)
    }
}

impl Default for RootCauseAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

// ===== Diagnosis Strategies =====

/// Strategy for diagnosing port conflicts
struct PortConflictStrategy;

impl DiagnosisStrategy for PortConflictStrategy {
    fn name(&self) -> &'static str {
        "Port Conflict Diagnosis"
    }

    fn applies_to(&self, problem: &ProblemContext) -> bool {
        let desc = problem.problem_description.to_lowercase();
        desc.contains("port")
            || desc.contains("address already in use")
            || desc.contains("bind")
            || desc.contains("eaddrinuse")
    }

    fn diagnostic_commands(&self, _problem: &ProblemContext) -> Vec<DiagnosticCommand> {
        vec![
            DiagnosticCommand::new("netstat", "netstat -tuln", "Check all listening ports"),
            DiagnosticCommand::new(
                "ss",
                "ss -tlnp",
                "Check listening TCP ports with process info",
            ),
            DiagnosticCommand::new(
                "lsof",
                "lsof -i -P -n | grep LISTEN",
                "List processes listening on ports",
            ),
        ]
    }

    fn analyze(&self, problem: &ProblemContext) -> Option<RootCause> {
        // Look for port conflict evidence in diagnostic data
        for (source, data) in &problem.diagnostic_data {
            if source.contains("netstat") || source.contains("ss") || source.contains("lsof") {
                // Simple heuristic: if we see multiple processes on common ports
                if data.contains(":80 ") || data.contains(":443 ") || data.contains(":8080 ") {
                    return Some(RootCause {
                        category: RootCauseCategory::PortConflict,
                        description: "Port is already in use by another process".to_string(),
                        evidence: vec![format!(
                            "Found from {}: {}",
                            source,
                            data.lines().take(3).collect::<Vec<_>>().join("; ")
                        )],
                        confidence: 85,
                        affected_components: vec![problem
                            .service
                            .clone()
                            .unwrap_or_else(|| "unknown".to_string())],
                    });
                }
            }
        }

        None
    }
}

/// Strategy for diagnosing service down issues
struct ServiceDownStrategy;

impl DiagnosisStrategy for ServiceDownStrategy {
    fn name(&self) -> &'static str {
        "Service Down Diagnosis"
    }

    fn applies_to(&self, problem: &ProblemContext) -> bool {
        let desc = problem.problem_description.to_lowercase();
        desc.contains("not running")
            || desc.contains("down")
            || desc.contains("failed")
            || desc.contains("inactive")
            || desc.contains("cannot start")
    }

    fn diagnostic_commands(&self, problem: &ProblemContext) -> Vec<DiagnosticCommand> {
        let mut commands = vec![DiagnosticCommand::new(
            "systemctl",
            "systemctl status",
            "Check systemd service status",
        )];

        if let Some(service) = &problem.service {
            commands.push(DiagnosticCommand::new(
                "systemctl",
                &format!("systemctl status {service}"),
                &format!("Check {service} status"),
            ));

            commands.push(DiagnosticCommand::new(
                "journalctl",
                &format!("journalctl -u {service} -n 50 --no-pager"),
                &format!("Check {service} recent logs"),
            ));
        }

        commands
    }

    fn analyze(&self, problem: &ProblemContext) -> Option<RootCause> {
        for (source, data) in &problem.diagnostic_data {
            if source.contains("systemctl status")
                && (data.contains("inactive") || data.contains("failed"))
            {
                return Some(RootCause {
                    category: RootCauseCategory::ServiceDown,
                    description: "Service is not running or has failed".to_string(),
                    evidence: vec![format!(
                        "systemctl status shows: {}",
                        data.lines().take(2).collect::<Vec<_>>().join(" ")
                    )],
                    confidence: 90,
                    affected_components: vec![problem
                        .service
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string())],
                });
            }
        }

        None
    }
}

/// Strategy for configuration errors
struct ConfigErrorStrategy;

impl DiagnosisStrategy for ConfigErrorStrategy {
    fn name(&self) -> &'static str {
        "Configuration Error Diagnosis"
    }

    fn applies_to(&self, problem: &ProblemContext) -> bool {
        let desc = problem.problem_description.to_lowercase();
        desc.contains("config")
            || desc.contains("configuration")
            || desc.contains("syntax")
            || desc.contains("invalid")
    }

    fn diagnostic_commands(&self, problem: &ProblemContext) -> Vec<DiagnosticCommand> {
        let mut commands = Vec::new();

        // Add service-specific config validation
        if let Some(service) = &problem.service {
            match service.as_str() {
                "nginx" => {
                    commands.push(DiagnosticCommand::new(
                        "nginx",
                        "nginx -t",
                        "Validate nginx configuration",
                    ));
                }
                "apache2" | "httpd" => {
                    commands.push(DiagnosticCommand::new(
                        "apache2",
                        "apache2ctl configtest",
                        "Validate Apache configuration",
                    ));
                }
                _ => {}
            }
        }

        commands
    }

    fn analyze(&self, problem: &ProblemContext) -> Option<RootCause> {
        for (source, data) in &problem.diagnostic_data {
            if data.contains("syntax error")
                || data.contains("invalid")
                || data.contains("parse error")
                || data.contains("configuration test failed")
            {
                return Some(RootCause {
                    category: RootCauseCategory::ConfigurationError,
                    description: "Configuration file contains errors".to_string(),
                    evidence: vec![format!(
                        "From {}: {}",
                        source,
                        data.lines().take(3).collect::<Vec<_>>().join("; ")
                    )],
                    confidence: 95,
                    affected_components: vec![problem
                        .service
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string())],
                });
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_problem_context_creation() {
        let ctx = ProblemContext::new("Test problem".to_string());
        assert_eq!(ctx.problem_description, "Test problem");
        assert!(ctx.diagnostic_data.is_empty());
    }

    #[test]
    fn test_port_conflict_strategy_applies() {
        let strategy = PortConflictStrategy;
        let mut problem = ProblemContext::new("Port 80 already in use".to_string());
        assert!(strategy.applies_to(&problem));

        problem.problem_description = "Service failed to start".to_string();
        assert!(!strategy.applies_to(&problem));
    }

    #[test]
    fn test_root_cause_analyzer() {
        let analyzer = RootCauseAnalyzer::new();
        let problem = ProblemContext::new("nginx port conflict".to_string());

        let strategies = analyzer.get_applicable_strategies(&problem);
        assert!(!strategies.is_empty());

        let commands = analyzer.get_diagnostic_commands(&problem);
        assert!(!commands.is_empty());
    }
}
