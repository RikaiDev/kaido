use super::{Tool, Translation, ExecutionResult, ToolContext, RiskLevel, LLMBackend, ErrorExplanation, Solution};
use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use std::time::Instant;

/// Drush tool implementation (Drupal CLI)
pub struct DrushTool {
    drush_path: PathBuf,
}

impl DrushTool {
    pub fn new() -> Self {
        Self {
            drush_path: which::which("drush")
                .unwrap_or_else(|_| PathBuf::from("vendor/bin/drush")),
        }
    }
    
    /// Get drush CLI path
    pub fn cli_path(&self) -> &PathBuf {
        &self.drush_path
    }
}

impl Default for DrushTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DrushTool {
    fn name(&self) -> &'static str {
        "drush"
    }
    
    fn detect_intent(&self, input: &str) -> f32 {
        // Explicit drush command → 100%
        if input.contains("drush") || input.contains("vendor/bin/drush") {
            return 1.0;
        }
        
        // Drupal keywords → 60%
        if input.to_lowercase().contains("drupal") {
            return 0.6;
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
Translate the following natural language to a Drush command.

User Input: {input}

Context:
- Working Directory: {pwd}

Common Drush operations:
- sql:cli: open SQL CLI
- sqlq: execute SQL query
- sql:connect: show connection string
- cr: clear cache
- cex: export configuration
- cim: import configuration
- uli: generate login link

Output JSON format:
{{
  "command": "exact drush command",
  "confidence": 0-100,
  "reasoning": "explanation"
}}
"#,
            input = input,
            pwd = context.working_directory.display(),
        );
        
        let result = llm.infer(&prompt).await?;
        
        Ok(Translation {
            command: result.command,
            confidence: result.confidence,
            reasoning: result.reasoning,
            tool_name: "drush".to_string(),
            requires_files: vec![],
        })
    }
    
    fn classify_risk(&self, command: &str, context: &ToolContext) -> RiskLevel {
        let cmd = command.to_lowercase();
        
        // Log working directory for Drush context
        log::debug!("Drush command in directory: {}", context.working_directory.display());
        
        // HIGH: Database operations, cache clear
        if cmd.contains("sql:drop") || cmd.contains("sql-drop") {
            return RiskLevel::High;
        }
        
        // MEDIUM: Import/export, cache operations
        if cmd.contains("cim") || cmd.contains("sql:cli") 
            || cmd.contains("sqlq") || cmd.contains("cr") {
            return RiskLevel::Medium;
        }
        
        // LOW: Read-only operations
        RiskLevel::Low
    }
    
    async fn execute(&self, command: &str) -> Result<ExecutionResult> {
        let start = Instant::now();
        
        // Parse command
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty command"));
        }
        
        // Use drush_path for execution if this is a drush command
        let cmd_path = if parts[0].contains("drush") {
            self.drush_path.as_os_str()
        } else {
            std::ffi::OsStr::new(&parts[0])
        };
        
        // Execute
        let output = tokio::process::Command::new(cmd_path)
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
        if let Some(explanation) = matcher.match_pattern(error) {
            return Some(explanation);
        }
        
        // Fallback: Case: drush sqlq database.mysql (your reported issue)
        if error.contains("ERROR 1064") && (error.contains(".mysql") || error.contains(".sql")) {
            // Extract filename
            let filename = extract_filename_from_error(error)
                .unwrap_or("database.mysql");
            
            return Some(ErrorExplanation {
                error_type: "Drush SQL File Execution Error".to_string(),
                reason: "drush sqlq 期望 SQL 語句作為參數，不能直接讀取檔案".to_string(),
                possible_causes: vec![
                    "您嘗試將檔名作為 SQL 語句執行".to_string(),
                    "drush sqlq 不支援直接讀取檔案參數".to_string(),
                ],
                solutions: vec![
                    Solution {
                        description: "使用 sql:cli 搭配重定向（推薦）".to_string(),
                        command: Some(format!("vendor/bin/drush sql:cli < {}", filename)),
                        risk_level: RiskLevel::Medium,
                    },
                    Solution {
                        description: "使用 cat + pipe".to_string(),
                        command: Some(format!("cat {} | vendor/bin/drush sqlq", filename)),
                        risk_level: RiskLevel::Medium,
                    },
                    Solution {
                        description: "使用 mysql 直接導入".to_string(),
                        command: Some(format!("mysql -u user -p database < {}", filename)),
                        risk_level: RiskLevel::Medium,
                    },
                ],
                recommended_solution: 0,  // sql:cli is most reliable
                documentation_links: vec![
                    "https://www.drush.org/latest/commands/sql_cli/".to_string(),
                ],
            });
        }
        
        None
    }
}

/// Extract filename from error message
fn extract_filename_from_error(error: &str) -> Option<&str> {
    // "drush sqlq database.mysql" → "database.mysql"
    error.split_whitespace()
        .find(|s| s.ends_with(".mysql") || s.ends_with(".sql"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drush_detection() {
        let tool = DrushTool::new();
        
        assert_eq!(tool.detect_intent("drush cr"), 1.0);
        assert_eq!(tool.detect_intent("vendor/bin/drush sqlq"), 1.0);
        assert_eq!(tool.detect_intent("drupal cache clear"), 0.6);
        assert_eq!(tool.detect_intent("kubectl get pods"), 0.0);
    }

    #[test]
    fn test_extract_filename() {
        assert_eq!(
            extract_filename_from_error("drush sqlq database.mysql"),
            Some("database.mysql")
        );
        assert_eq!(
            extract_filename_from_error("drush sqlq backup.sql"),
            Some("backup.sql")
        );
        assert_eq!(
            extract_filename_from_error("drush cr"),
            None
        );
    }

    #[test]
    fn test_drush_error_explanation() {
        let tool = DrushTool::new();
        let error = "ERROR 1064 at line 1: drush sqlq database.mysql";
        
        let explanation = tool.explain_error(error);
        assert!(explanation.is_some());
        
        let exp = explanation.unwrap();
        assert_eq!(exp.error_type, "Drush SQL File Execution Error");
        assert_eq!(exp.solutions.len(), 3);
        assert!(exp.solutions[0].command.as_ref().unwrap().contains("sql:cli"));
    }
}

