use anyhow::Result;
use crate::tools::{
    ToolRegistry, ToolContext, Translation, ExecutionResult, RiskLevel, LLMBackend,
    ErrorExplanation,
};
use crate::audit::{AuditLogger, AuditContext, UserAction};

/// Command processing result
#[derive(Debug)]
pub enum CommandResult {
    /// Command was executed successfully
    Executed {
        translation: Translation,
        execution: ExecutionResult,
    },
    /// Command was cancelled by user
    Cancelled {
        translation: Translation,
    },
    /// Error explanation provided
    ErrorExplained {
        explanation: ErrorExplanation,
    },
}

/// Universal command processing engine
pub struct CommandEngine {
    registry: ToolRegistry,
    audit_logger: Option<AuditLogger>,
}

impl CommandEngine {
    /// Create new command engine
    pub fn new() -> Self {
        Self {
            registry: ToolRegistry::new(),
            audit_logger: None,
        }
    }
    
    /// Create with audit logging enabled
    pub fn with_audit(audit_logger: AuditLogger) -> Self {
        Self {
            registry: ToolRegistry::new(),
            audit_logger: Some(audit_logger),
        }
    }
    
    /// Get reference to tool registry
    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }
    
    /// Process user input (natural language → command)
    /// 
    /// Main workflow:
    /// 1. Detect tool from input
    /// 2. Translate to command using LLM
    /// 3. Validate required files
    /// 4. Classify risk level
    /// 5. Get confirmation if needed (handled by caller)
    /// 6. Execute command
    /// 7. Log to audit
    /// 8. Return result
    pub async fn process_input(
        &self,
        input: &str,
        context: &ToolContext,
        llm: &dyn LLMBackend,
    ) -> Result<Translation> {
        // 1. Detect tool
        let tool = self.registry.detect_tool(input)
            .ok_or_else(|| anyhow::anyhow!(
                "Cannot detect tool. Please be more specific (e.g., 'kubectl get pods', 'docker ps', 'show databases')"
            ))?;
        
        log::info!("Detected tool: {}", tool.name());
        
        // 2. Translate to command
        let translation = tool.translate(input, context, llm).await?;
        
        log::info!(
            "Translated: '{}' → '{}' (confidence: {}%)",
            input,
            translation.command,
            translation.confidence
        );
        
        // 3. Validate required files
        self.validate_required_files(&translation.requires_files)?;
        
        Ok(translation)
    }
    
    /// Execute a translated command
    pub async fn execute_command(
        &self,
        translation: &Translation,
        context: &ToolContext,
    ) -> Result<ExecutionResult> {
        // Get the tool
        let tool = self.registry.get_tool(&translation.tool_name)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", translation.tool_name))?;
        
        // Log execution context
        log::info!(
            "Executing {} command in directory: {}",
            translation.tool_name,
            context.working_directory.display()
        );
        
        // Execute
        let result = tool.execute(&translation.command).await?;
        
        log::info!(
            "Execution complete: exit_code={}, duration={:?}",
            result.exit_code,
            result.duration
        );
        
        Ok(result)
    }
    
    /// Classify risk level of a command
    pub fn classify_risk(
        &self,
        translation: &Translation,
        context: &ToolContext,
    ) -> Result<RiskLevel> {
        let tool = self.registry.get_tool(&translation.tool_name)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", translation.tool_name))?;
        
        let risk = tool.classify_risk(&translation.command, context);
        
        log::info!(
            "Risk classification: {} → {}",
            translation.command,
            risk
        );
        
        Ok(risk)
    }
    
    /// Log command execution to audit
    pub fn log_execution(
        &self,
        translation: &Translation,
        context: &ToolContext,
        result: &ExecutionResult,
        risk_level: RiskLevel,
        user_action: UserAction,
    ) -> Result<()> {
        let Some(logger) = &self.audit_logger else {
            return Ok(()); // Audit logging not enabled
        };
        
        // Extract context info (kubectl-specific for now)
        let (environment, cluster, namespace) = if let Some(kubectl_ctx) = &context.kubectl_context {
            (
                kubectl_ctx.name.as_str(),
                kubectl_ctx.cluster.as_str(),
                kubectl_ctx.namespace.as_deref(),
            )
        } else {
            ("unknown", "unknown", None)
        };
        
        // Convert RiskLevel to kubectl::RiskLevel for audit
        let kubectl_risk = convert_risk_level(risk_level);
        
        // Build audit context
        let audit_ctx = AuditContext {
            natural_language: "", // Will be provided by caller
            kubectl_command: &translation.command,
            confidence_score: Some(translation.confidence),
            risk_level: kubectl_risk,
            environment,
            cluster,
            namespace,
        };
        
        // Create audit entry
        let entry = crate::audit::audit_entry_from_execution(
            audit_ctx,
            &convert_execution_result_for_audit(result),
            user_action,
        );
        
        // Log
        logger.log_execution(entry)?;
        
        Ok(())
    }
    
    /// Log cancelled command
    pub fn log_cancelled(
        &self,
        translation: &Translation,
        context: &ToolContext,
        risk_level: RiskLevel,
    ) -> Result<()> {
        let Some(logger) = &self.audit_logger else {
            return Ok(());
        };
        
        let (environment, cluster, namespace) = if let Some(kubectl_ctx) = &context.kubectl_context {
            (
                kubectl_ctx.name.as_str(),
                kubectl_ctx.cluster.as_str(),
                kubectl_ctx.namespace.as_deref(),
            )
        } else {
            ("unknown", "unknown", None)
        };
        
        let kubectl_risk = convert_risk_level(risk_level);
        
        let entry = crate::audit::audit_entry_cancelled(
            "",
            &translation.command,
            Some(translation.confidence),
            kubectl_risk,
            environment,
            cluster,
            namespace,
        );
        
        logger.log_execution(entry)?;
        
        Ok(())
    }
    
    /// Explain an error message
    pub async fn explain_error(
        &self,
        error_text: &str,
        context: &ToolContext,
        llm: &dyn LLMBackend,
    ) -> Result<ErrorExplanation> {
        // Try to detect tool from error message
        let tool = self.detect_tool_from_error(error_text);
        
        // Try tool-specific error explanation
        if let Some(tool) = tool {
            if let Some(explanation) = tool.explain_error(error_text) {
                log::info!("Tool-specific error explanation: {}", tool.name());
                return Ok(explanation);
            }
        }
        
        // Use generic ErrorExplainer with LLM
        log::info!("Using LLM for error explanation");
        let explainer = crate::error::explainer::ErrorExplainer::new();
        explainer.explain(error_text, context, llm).await
    }
    
    /// Validate that required files exist
    fn validate_required_files(&self, files: &[std::path::PathBuf]) -> Result<()> {
        for file in files {
            if !file.exists() {
                return Err(anyhow::anyhow!(
                    "Required file not found: {}",
                    file.display()
                ));
            }
        }
        Ok(())
    }
    
    /// Detect tool from error message
    fn detect_tool_from_error(&self, error: &str) -> Option<&dyn crate::tools::Tool> {
        let error_lower = error.to_lowercase();
        
        // Check for tool names in error
        if error_lower.contains("kubectl") || error_lower.contains("kubernetes") {
            return self.registry.get_tool("kubectl");
        }
        
        if error_lower.contains("docker") {
            return self.registry.get_tool("docker");
        }
        
        if error_lower.contains("mysql") || error_lower.contains("error 1064") {
            return self.registry.get_tool("mysql");
        }
        
        if error_lower.contains("drush") {
            return self.registry.get_tool("drush");
        }
        
        None
    }
}

impl Default for CommandEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert tools::ExecutionResult to kubectl::ExecutionResult for audit
fn convert_execution_result_for_audit(result: &ExecutionResult) -> crate::kubectl::ExecutionResult {
    crate::kubectl::ExecutionResult {
        exit_code: Some(result.exit_code),
        stdout: result.stdout.clone(),
        stderr: result.stderr.clone(),
        execution_duration_ms: result.duration.as_millis() as i64,
    }
}

/// Convert tools::RiskLevel to kubectl::RiskLevel for audit
fn convert_risk_level(risk: RiskLevel) -> crate::kubectl::RiskLevel {
    match risk {
        RiskLevel::Low => crate::kubectl::RiskLevel::Low,
        RiskLevel::Medium => crate::kubectl::RiskLevel::Medium,
        RiskLevel::High => crate::kubectl::RiskLevel::High,
        RiskLevel::Critical => crate::kubectl::RiskLevel::High, // Map Critical to High for kubectl
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{LLMResponse};
    use async_trait::async_trait;
    
    struct MockLLM;
    
    #[async_trait]
    impl LLMBackend for MockLLM {
        async fn infer(&self, _prompt: &str) -> Result<LLMResponse> {
            Ok(LLMResponse {
                command: "kubectl get pods".to_string(),
                confidence: 95,
                reasoning: "Standard pod listing command".to_string(),
            })
        }
    }
    
    #[tokio::test]
    async fn test_process_input_kubectl() {
        let engine = CommandEngine::new();
        let context = ToolContext::default();
        let llm = MockLLM;
        
        let result = engine.process_input("list all pods", &context, &llm).await;
        
        // Should fail because no kubectl context
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_process_input_docker() {
        let engine = CommandEngine::new();
        let context = ToolContext::default();
        let llm = MockLLM;
        
        let result = engine.process_input("docker ps", &context, &llm).await;
        
        // Should succeed (docker doesn't need context)
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_detect_tool_from_error() {
        let engine = CommandEngine::new();
        
        let tool = engine.detect_tool_from_error("kubectl error: pod not found");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name(), "kubectl");
        
        let tool = engine.detect_tool_from_error("ERROR 1064: SQL syntax error");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name(), "mysql");
        
        let tool = engine.detect_tool_from_error("Cannot connect to Docker daemon");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name(), "docker");
    }
}

