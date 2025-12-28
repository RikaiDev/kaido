use crate::tools::{ErrorExplanation, RiskLevel, Solution};
use regex::Regex;

/// Error pattern for matching
pub struct ErrorPattern {
    pub regex: Regex,
    pub tool: String,
    pub error_type: String,
    pub explanation_template: String,
    pub solutions: Vec<Solution>,
}

/// Pattern matcher for common errors
pub struct PatternMatcher {
    patterns: Vec<ErrorPattern>,
}

impl PatternMatcher {
    pub fn new() -> Self {
        let mut matcher = Self { patterns: vec![] };
        matcher.init_patterns();
        matcher
    }

    /// Initialize all error patterns
    /// IMPORTANT: More specific patterns should come first!
    fn init_patterns(&mut self) {
        // Drush sqlq 檔案錯誤 (user's reported case) - MUST BE FIRST
        // More specific than generic ERROR 1064
        // Matches: ERROR 1064 + 'database.mysql' in error message
        self.add_pattern(ErrorPattern {
            regex: Regex::new(r"(?i)ERROR\s+1064.*database\.(mysql|sql)").unwrap(),
            tool: "drush".to_string(),
            error_type: "Drush SQL File Execution Error".to_string(),
            explanation_template: "drush sqlq 期望 SQL 語句，不能直接讀取檔案".to_string(),
            solutions: vec![
                Solution {
                    description: "使用 sql:cli 搭配重定向（推薦）".to_string(),
                    command: Some("vendor/bin/drush sql:cli < {filename}".to_string()),
                    risk_level: RiskLevel::Medium,
                },
                Solution {
                    description: "使用 cat + pipe".to_string(),
                    command: Some("cat {filename} | vendor/bin/drush sqlq".to_string()),
                    risk_level: RiskLevel::Medium,
                },
                Solution {
                    description: "使用 mysql 直接導入".to_string(),
                    command: Some("mysql -u user -p database < {filename}".to_string()),
                    risk_level: RiskLevel::Medium,
                },
            ],
        });

        // MySQL ERROR 1064: SQL syntax error (AFTER drush pattern)
        self.add_pattern(ErrorPattern {
            regex: Regex::new(r"(?i)ERROR\s+1064").unwrap(),
            tool: "mysql".to_string(),
            error_type: "SQL Syntax Error".to_string(),
            explanation_template: "SQL 語法錯誤，MySQL 無法解析您的 SQL 語句".to_string(),
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
        });

        // kubectl permission denied
        self.add_pattern(ErrorPattern {
            regex: Regex::new(r"(?i)(forbidden|User.*cannot)").unwrap(),
            tool: "kubectl".to_string(),
            error_type: "Kubernetes RBAC Permission Denied".to_string(),
            explanation_template: "您沒有執行此操作的權限".to_string(),
            solutions: vec![
                Solution {
                    description: "檢查您的 RBAC 權限".to_string(),
                    command: Some("kubectl auth can-i {verb} {resource}".to_string()),
                    risk_level: RiskLevel::Low,
                },
                Solution {
                    description: "聯繫集群管理員申請權限".to_string(),
                    command: None,
                    risk_level: RiskLevel::Low,
                },
            ],
        });

        // Docker daemon not running
        self.add_pattern(ErrorPattern {
            regex: Regex::new(r"Cannot connect to the Docker daemon").unwrap(),
            tool: "docker".to_string(),
            error_type: "Docker Daemon Not Running".to_string(),
            explanation_template: "Docker daemon 未執行或無法連接".to_string(),
            solutions: vec![
                Solution {
                    description: "啟動 Docker daemon（macOS）".to_string(),
                    command: Some("open -a Docker".to_string()),
                    risk_level: RiskLevel::Low,
                },
                Solution {
                    description: "啟動 Docker daemon（Linux）".to_string(),
                    command: Some("sudo systemctl start docker".to_string()),
                    risk_level: RiskLevel::Medium,
                },
                Solution {
                    description: "檢查 Docker daemon 狀態".to_string(),
                    command: Some("docker info".to_string()),
                    risk_level: RiskLevel::Low,
                },
            ],
        });

        // MySQL ERROR 1045: Access denied
        self.add_pattern(ErrorPattern {
            regex: Regex::new(r"ERROR 1045|Access denied").unwrap(),
            tool: "mysql".to_string(),
            error_type: "MySQL Authentication Failed".to_string(),
            explanation_template: "用戶名或密碼錯誤，無法連接資料庫".to_string(),
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
        });

        // kubectl context not set
        self.add_pattern(ErrorPattern {
            regex: Regex::new(r"current-context is not set").unwrap(),
            tool: "kubectl".to_string(),
            error_type: "Kubectl Context Not Set".to_string(),
            explanation_template: "kubectl 沒有設定當前上下文".to_string(),
            solutions: vec![
                Solution {
                    description: "查看可用的上下文".to_string(),
                    command: Some("kubectl config get-contexts".to_string()),
                    risk_level: RiskLevel::Low,
                },
                Solution {
                    description: "設定當前上下文".to_string(),
                    command: Some("kubectl config use-context {context-name}".to_string()),
                    risk_level: RiskLevel::Low,
                },
            ],
        });

        // Docker image not found
        self.add_pattern(ErrorPattern {
            regex: Regex::new(r"Unable to find image|No such image").unwrap(),
            tool: "docker".to_string(),
            error_type: "Docker Image Not Found".to_string(),
            explanation_template: "找不到指定的 Docker 映像".to_string(),
            solutions: vec![
                Solution {
                    description: "拉取映像".to_string(),
                    command: Some("docker pull {image-name}".to_string()),
                    risk_level: RiskLevel::Low,
                },
                Solution {
                    description: "查看本地映像".to_string(),
                    command: Some("docker images".to_string()),
                    risk_level: RiskLevel::Low,
                },
            ],
        });
    }

    pub fn add_pattern(&mut self, pattern: ErrorPattern) {
        self.patterns.push(pattern);
    }

    /// Match error against patterns
    pub fn match_pattern(&self, error: &str) -> Option<ErrorExplanation> {
        for pattern in &self.patterns {
            if pattern.regex.is_match(error) {
                log::info!("Matched error pattern: {}", pattern.error_type);

                // Extract filename from error if present (for drush sqlq case)
                let filename = extract_filename_from_drush_error(error);

                // Clone and replace {filename} placeholder in solutions
                let solutions: Vec<Solution> = pattern
                    .solutions
                    .iter()
                    .map(|sol| {
                        let command = sol.command.as_ref().map(|cmd| {
                            if let Some(ref fname) = filename {
                                cmd.replace("{filename}", fname)
                            } else {
                                cmd.clone()
                            }
                        });

                        Solution {
                            description: sol.description.clone(),
                            command,
                            risk_level: sol.risk_level,
                        }
                    })
                    .collect();

                return Some(ErrorExplanation {
                    error_type: pattern.error_type.clone(),
                    reason: pattern.explanation_template.clone(),
                    possible_causes: vec![
                        format!("工具：{}", pattern.tool),
                        "命令格式不正確".to_string(),
                        "環境配置問題".to_string(),
                    ],
                    solutions,
                    recommended_solution: 0,
                    documentation_links: vec![],
                });
            }
        }
        None
    }
}

/// Extract filename from drush sqlq error message
fn extract_filename_from_drush_error(error: &str) -> Option<String> {
    // Look for patterns like "database.mysql" or "*.sql"
    use regex::Regex;
    let re = Regex::new(r"(?:^|\s)([a-zA-Z0-9_-]+\.(mysql|sql))").ok()?;
    re.captures(error)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mysql_error_1064() {
        let matcher = PatternMatcher::new();
        let error = "ERROR 1064 (42000) at line 1: You have an error in your SQL syntax";

        let explanation = matcher.match_pattern(error);
        assert!(explanation.is_some());

        let exp = explanation.unwrap();
        assert_eq!(exp.error_type, "SQL Syntax Error");
        assert!(!exp.solutions.is_empty());
    }

    #[test]
    fn test_drush_sqlq_error() {
        let matcher = PatternMatcher::new();
        let error = "ERROR 1064 at line 1: drush sqlq database.mysql";

        let explanation = matcher.match_pattern(error);
        assert!(explanation.is_some());

        let exp = explanation.unwrap();
        assert_eq!(exp.error_type, "Drush SQL File Execution Error");
        assert_eq!(exp.solutions.len(), 3);
        assert!(exp.solutions[0]
            .command
            .as_ref()
            .unwrap()
            .contains("sql:cli"));
    }

    #[test]
    fn test_kubectl_permission_error() {
        let matcher = PatternMatcher::new();
        let error =
            "Error from server (Forbidden): kubectl get pods forbidden: User cannot list resource";

        let explanation = matcher.match_pattern(error);
        assert!(explanation.is_some());

        let exp = explanation.unwrap();
        assert_eq!(exp.error_type, "Kubernetes RBAC Permission Denied");
    }

    #[test]
    fn test_docker_daemon_error() {
        let matcher = PatternMatcher::new();
        let error = "Cannot connect to the Docker daemon at unix:///var/run/docker.sock";

        let explanation = matcher.match_pattern(error);
        assert!(explanation.is_some());

        let exp = explanation.unwrap();
        assert_eq!(exp.error_type, "Docker Daemon Not Running");
        assert!(exp.solutions.len() >= 2);
    }

    #[test]
    fn test_no_match() {
        let matcher = PatternMatcher::new();
        let error = "Some random error that doesn't match any pattern";

        let explanation = matcher.match_pattern(error);
        assert!(explanation.is_none());
    }
}
