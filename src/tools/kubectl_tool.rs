use super::{Tool, Translation, ExecutionResult, ToolContext, RiskLevel, LLMBackend, ErrorExplanation};
use anyhow::Result;
use async_trait::async_trait;

/// Kubectl tool implementation
pub struct KubectlTool {}

impl KubectlTool {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for KubectlTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for KubectlTool {
    fn name(&self) -> &'static str {
        "kubectl"
    }
    
    fn detect_intent(&self, input: &str) -> f32 {
        let keywords = ["pod", "deployment", "service", "namespace", "cluster", "node"];
        let lower = input.to_lowercase();
        
        // Explicit kubectl command → 100%
        if lower.contains("kubectl") {
            return 1.0;
        }
        
        // Contains k8s keywords → 70-90%
        let matches = keywords.iter().filter(|k| lower.contains(*k)).count();
        if matches > 0 {
            return (matches as f32 / keywords.len() as f32) * 0.9;
        }
        
        0.0
    }
    
    async fn translate(
        &self,
        input: &str,
        context: &ToolContext,
        llm: &dyn LLMBackend,
    ) -> Result<Translation> {
        // Get kubectl context
        let kubectl_ctx = context.kubectl_context
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No kubectl context configured"))?;
        
        // Build prompt for kubectl translation
        let prompt = format!(r#"
Translate the following natural language to a kubectl command.

User Input: {input}

Current Context:
- Cluster: {cluster}
- Namespace: {namespace}
- Environment: {env:?}

Common kubectl operations:
- get: list resources (pods, deployments, services, nodes)
- describe: detailed information about resources
- logs: view pod logs
- exec: execute command in container
- apply: apply configuration
- delete: remove resources
- scale: scale replicas
- port-forward: forward local port to pod

Output JSON format:
{{
  "command": "exact kubectl command",
  "confidence": 0-100,
  "reasoning": "explanation"
}}
"#,
            input = input,
            cluster = kubectl_ctx.cluster,
            namespace = kubectl_ctx.namespace.as_deref().unwrap_or("default"),
            env = kubectl_ctx.environment_type,
        );
        
        // Call LLM
        let result = llm.infer(&prompt).await?;
        
        Ok(Translation {
            command: result.command,
            confidence: result.confidence,
            reasoning: result.reasoning,
            tool_name: "kubectl".to_string(),
            requires_files: vec![],
        })
    }
    
    fn classify_risk(&self, command: &str, context: &ToolContext) -> RiskLevel {
        // Reuse existing risk classifier logic
        let cmd_lower = command.to_lowercase();
        
        // Check if production environment for enhanced safety
        let is_production = context.kubectl_context
            .as_ref()
            .map(|ctx| ctx.environment_type == crate::kubectl::EnvironmentType::Production)
            .unwrap_or(false);
        
        if is_production {
            log::warn!("Production environment detected for kubectl command");
        }
        
        // CRITICAL: Batch operations
        if (cmd_lower.contains("delete") && cmd_lower.contains("--all"))
            || (cmd_lower.contains("delete") && cmd_lower.contains("namespace")) {
            return RiskLevel::Critical;
        }
        
        // HIGH: Destructive operations
        if cmd_lower.contains("delete") || cmd_lower.contains("drain") {
            return RiskLevel::High;
        }
        
        // Special case: scale to 0 replicas
        if cmd_lower.contains("scale") && (
            cmd_lower.contains("--replicas=0") || 
            cmd_lower.contains("--replicas 0")
        ) {
            return RiskLevel::High;
        }
        
        // MEDIUM: State-modifying operations
        if cmd_lower.contains("apply") 
            || cmd_lower.contains("create")
            || cmd_lower.contains("patch")
            || cmd_lower.contains("edit")
            || cmd_lower.contains("scale")
            || cmd_lower.contains("rollout")
            || cmd_lower.contains("restart")
            || cmd_lower.contains("label")
            || cmd_lower.contains("annotate")
        {
            return RiskLevel::Medium;
        }
        
        // LOW: Read-only operations (default)
        RiskLevel::Low
    }
    
    async fn execute(&self, command: &str) -> Result<ExecutionResult> {
        // Reuse existing kubectl executor (sync function)
        let kubectl_result = crate::kubectl::execute_kubectl(command)?;
        
        // Convert kubectl::ExecutionResult to tools::ExecutionResult
        Ok(ExecutionResult {
            exit_code: kubectl_result.exit_code.unwrap_or(-1),
            stdout: kubectl_result.stdout,
            stderr: kubectl_result.stderr,
            duration: std::time::Duration::from_millis(kubectl_result.execution_duration_ms as u64),
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
    fn test_kubectl_detection() {
        let tool = KubectlTool::new();
        
        assert_eq!(tool.detect_intent("kubectl get pods"), 1.0);
        assert_eq!(tool.detect_intent("list all pods"), 0.15); // 1/6 keywords
        assert!(tool.detect_intent("show deployments") > 0.0);
        assert_eq!(tool.detect_intent("docker ps"), 0.0);
    }

    #[test]
    fn test_kubectl_risk_classification() {
        let tool = KubectlTool::new();
        let ctx = ToolContext::default();
        
        assert_eq!(
            tool.classify_risk("kubectl get pods", &ctx),
            RiskLevel::Low
        );
        
        assert_eq!(
            tool.classify_risk("kubectl delete deployment nginx", &ctx),
            RiskLevel::High
        );
        
        assert_eq!(
            tool.classify_risk("kubectl delete namespace production", &ctx),
            RiskLevel::Critical
        );
        
        assert_eq!(
            tool.classify_risk("kubectl apply -f deployment.yaml", &ctx),
            RiskLevel::Medium
        );
    }
}

