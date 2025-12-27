use super::{Tool, Translation, ExecutionResult, ToolContext, RiskLevel, LLMBackend, ErrorExplanation, Solution};
use anyhow::Result;
use async_trait::async_trait;
// use std::time::{Duration, Instant};

/// SQL dialect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SQLDialect {
    MySQL,
    PostgreSQL,
}

impl SQLDialect {
    /// Get dialect name
    pub fn name(&self) -> &'static str {
        match self {
            SQLDialect::MySQL => "MySQL",
            SQLDialect::PostgreSQL => "PostgreSQL",
        }
    }
    
    /// Get CLI command
    pub fn cli_command(&self) -> &'static str {
        match self {
            SQLDialect::MySQL => "mysql",
            SQLDialect::PostgreSQL => "psql",
        }
    }
}

/// SQL tool implementation (MySQL/PostgreSQL)
pub struct SQLTool {
    dialect: SQLDialect,
}

impl SQLTool {
    pub fn new(dialect: SQLDialect) -> Self {
        Self { dialect }
    }
    
    /// Get SQL dialect
    pub fn dialect(&self) -> &SQLDialect {
        &self.dialect
    }
}

#[async_trait]
impl Tool for SQLTool {
    fn name(&self) -> &'static str {
        match self.dialect {
            SQLDialect::MySQL => "mysql",
            SQLDialect::PostgreSQL => "postgresql",
        }
    }
    
    fn detect_intent(&self, input: &str) -> f32 {
        let lower = input.to_lowercase();
        
        // SQL keywords and common patterns
        let sql_keywords = [
            "select", "insert", "update", "delete", "create", "drop",
            "alter", "show", "describe", "database", "table", "from", "where"
        ];
        
        let matches = sql_keywords.iter()
            .filter(|k| lower.contains(*k))
            .count();
        
        if matches > 0 {
            // Base confidence on keyword density
            // 1 keyword: 0.4, 2 keywords: 0.6, 3+ keywords: 0.9
            return (matches as f32 * 0.3).min(0.9);
        }
        
        0.0
    }
    
    async fn translate(
        &self,
        input: &str,
        context: &ToolContext,
        llm: &dyn LLMBackend,
    ) -> Result<Translation> {
        // Check if database connection is configured
        let db_context = if let Some(db_conn) = &context.db_connection {
            format!("Database: {} on {}:{}", db_conn.database, db_conn.host, db_conn.port)
        } else {
            "No database connection configured".to_string()
        };
        
        let prompt = format!(r#"
Translate the following natural language to a SQL command.

User Input: {input}

Dialect: {dialect:?}
Context: {db_context}

Common SQL operations:
- SELECT: query data
- INSERT: add new records
- UPDATE: modify existing records
- DELETE: remove records
- CREATE: create database/table
- DROP: remove database/table
- SHOW: list databases/tables
- DESCRIBE: show table structure

Output JSON format:
{{
  "command": "exact SQL command",
  "confidence": 0-100,
  "reasoning": "explanation"
}}
"#,
            input = input,
            dialect = self.dialect,
        );
        
        let result = llm.infer(&prompt).await?;
        
        log::info!("SQL translation: {} ({})", self.name(), db_context);
        
        Ok(Translation {
            command: result.command,
            confidence: result.confidence,
            reasoning: result.reasoning,
            tool_name: self.name().to_string(),
            requires_files: vec![],
        })
    }
    
    fn classify_risk(&self, command: &str, context: &ToolContext) -> RiskLevel {
        let cmd = command.to_lowercase();
        
        // Check if production database
        let is_production = context.db_connection
            .as_ref()
            .map(|conn| conn.is_production)
            .unwrap_or(false);
        
        if is_production {
            log::warn!("Production database detected for SQL command");
        }
        
        // CRITICAL: DROP DATABASE, DELETE FROM without WHERE
        if cmd.contains("drop database") || cmd.contains("drop schema") {
            return RiskLevel::Critical;
        }
        
        if cmd.contains("delete from") && !cmd.contains("where") {
            return RiskLevel::Critical;
        }
        
        if cmd.contains("truncate") && !cmd.contains("where") {
            return RiskLevel::Critical;
        }
        
        // HIGH: DROP TABLE, TRUNCATE with WHERE
        if cmd.contains("drop table") {
            return RiskLevel::High;
        }
        
        if cmd.contains("truncate") && cmd.contains("where") {
            return RiskLevel::High;
        }
        
        // MEDIUM: INSERT, UPDATE, DELETE (with WHERE), ALTER
        if cmd.contains("insert") || cmd.contains("update") 
            || (cmd.contains("delete") && cmd.contains("where"))
            || cmd.contains("alter") || cmd.contains("create") {
            return RiskLevel::Medium;
        }
        
        // LOW: SELECT, SHOW, DESCRIBE
        RiskLevel::Low
    }
    
    async fn execute(&self, command: &str) -> Result<ExecutionResult> {
        // Note: For MVP, we don't execute SQL directly for safety reasons.
        // Instead, we provide the correct CLI command for the user to run.
        
        let cli_command = match self.dialect {
            SQLDialect::MySQL => format!("echo '{}' | mysql", command),
            SQLDialect::PostgreSQL => format!("echo '{}' | psql", command),
        };
        
        Err(anyhow::anyhow!(
            "Direct SQL execution not available in MVP for safety.\n\
            Please use the following command:\n  {}",
            cli_command
        ))
    }
    
    fn explain_error(&self, error: &str) -> Option<ErrorExplanation> {
        // MySQL ERROR 1064: Syntax error
        if error.contains("ERROR 1064") {
            return Some(ErrorExplanation {
                error_type: "SQL Syntax Error".to_string(),
                reason: "SQL 語法錯誤，資料庫無法解析您的 SQL 語句".to_string(),
                possible_causes: vec![
                    "SQL 關鍵字拼寫錯誤".to_string(),
                    "缺少必要的語法元素（如 WHERE, FROM）".to_string(),
                    "使用了保留字作為標識符未加引號".to_string(),
                ],
                solutions: vec![
                    Solution {
                        description: "檢查 SQL 語法，確保關鍵字正確".to_string(),
                        command: None,
                        risk_level: RiskLevel::Low,
                    },
                    Solution {
                        description: "如果使用保留字，請用反引號包裹：`table`".to_string(),
                        command: None,
                        risk_level: RiskLevel::Low,
                    },
                ],
                recommended_solution: 0,
                documentation_links: vec![
                    "https://dev.mysql.com/doc/refman/8.0/en/sql-syntax.html".to_string(),
                ],
            });
        }
        
        // MySQL ERROR 1045: Access denied
        if error.contains("ERROR 1045") || error.contains("Access denied") {
            return Some(ErrorExplanation {
                error_type: "MySQL Authentication Failed".to_string(),
                reason: "用戶名或密碼錯誤，無法連接資料庫".to_string(),
                possible_causes: vec![
                    "密碼不正確".to_string(),
                    "用戶不存在或沒有權限".to_string(),
                    "主機名限制（用戶只允許從特定主機連接）".to_string(),
                ],
                solutions: vec![
                    Solution {
                        description: "檢查用戶名和密碼".to_string(),
                        command: None,
                        risk_level: RiskLevel::Low,
                    },
                    Solution {
                        description: "查看用戶權限".to_string(),
                        command: Some("SELECT user, host FROM mysql.user;".to_string()),
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
    fn test_sql_detection() {
        let tool = SQLTool::new(SQLDialect::MySQL);
        
        assert!(tool.detect_intent("select * from users") > 0.5);
        assert!(tool.detect_intent("show databases") > 0.0);
        assert_eq!(tool.detect_intent("kubectl get pods"), 0.0);
    }

    #[test]
    fn test_sql_risk_classification() {
        let tool = SQLTool::new(SQLDialect::MySQL);
        let ctx = ToolContext::default();
        
        assert_eq!(
            tool.classify_risk("SELECT * FROM users", &ctx),
            RiskLevel::Low
        );
        
        assert_eq!(
            tool.classify_risk("DELETE FROM users WHERE id = 1", &ctx),
            RiskLevel::Medium
        );
        
        assert_eq!(
            tool.classify_risk("DELETE FROM users", &ctx),
            RiskLevel::Critical
        );
        
        assert_eq!(
            tool.classify_risk("DROP DATABASE production", &ctx),
            RiskLevel::Critical
        );
    }
}

