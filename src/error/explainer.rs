use super::PatternMatcher;
use crate::tools::{ErrorExplanation, LLMBackend, ToolContext};
use anyhow::Result;

/// Error explainer engine
///
/// Provides intelligent error explanation through:
/// 1. Pattern matching (fast path) - matches common error patterns
/// 2. LLM inference (slow path) - uses AI for unknown errors
pub struct ErrorExplainer {
    pattern_matcher: PatternMatcher,
}

impl ErrorExplainer {
    pub fn new() -> Self {
        Self {
            pattern_matcher: PatternMatcher::new(),
        }
    }

    /// Explain an error message
    pub async fn explain(
        &self,
        error_text: &str,
        context: &ToolContext,
        llm: &dyn LLMBackend,
    ) -> Result<ErrorExplanation> {
        // 1. Try pattern matching (fast path)
        if let Some(explanation) = self.pattern_matcher.match_pattern(error_text) {
            log::info!("Pattern match found for error");
            return Ok(explanation);
        }

        // 2. Use LLM inference (slow path)
        log::info!("No pattern match, using LLM for error explanation");
        let prompt = self.build_error_explanation_prompt(error_text, context);
        let llm_result = llm.infer(&prompt).await?;

        // 3. Parse LLM output
        self.parse_llm_explanation(&llm_result.reasoning)
    }

    /// Build prompt for LLM error explanation
    fn build_error_explanation_prompt(&self, error: &str, context: &ToolContext) -> String {
        format!(
            r#"
You are an expert DevOps engineer. Explain the following error and provide solutions.

Error Message:
```
{error}
```

Context:
- Working Directory: {pwd}
- User: {user}

Provide your response in JSON format:
{{
  "error_type": "Brief error classification",
  "reason": "Human-readable explanation in Traditional Chinese",
  "possible_causes": ["cause 1 (繁體中文)", "cause 2 (繁體中文)", "cause 3 (繁體中文)"],
  "solutions": [
    {{
      "description": "Solution description (繁體中文)",
      "command": "exact command to run (if applicable)",
      "risk_level": "low|medium|high"
    }}
  ],
  "recommended_solution": 0
}}
"#,
            error = error,
            pwd = context.working_directory.display(),
            user = context.user,
        )
    }

    /// Parse LLM explanation response
    fn parse_llm_explanation(&self, llm_response: &str) -> Result<ErrorExplanation> {
        // Try to parse JSON
        let parsed: serde_json::Value = serde_json::from_str(llm_response)
            .map_err(|e| anyhow::anyhow!("Failed to parse LLM response as JSON: {e}"))?;

        let error_type = parsed["error_type"]
            .as_str()
            .unwrap_or("Unknown Error")
            .to_string();

        let reason = parsed["reason"]
            .as_str()
            .unwrap_or("無法解釋此錯誤")
            .to_string();

        let possible_causes: Vec<String> = parsed["possible_causes"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let solutions: Vec<crate::tools::Solution> = parsed["solutions"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|sol| {
                        Some(crate::tools::Solution {
                            description: sol["description"].as_str()?.to_string(),
                            command: sol["command"].as_str().map(String::from),
                            risk_level: match sol["risk_level"].as_str()? {
                                "low" => crate::tools::RiskLevel::Low,
                                "medium" => crate::tools::RiskLevel::Medium,
                                "high" => crate::tools::RiskLevel::High,
                                "critical" => crate::tools::RiskLevel::Critical,
                                _ => crate::tools::RiskLevel::Low,
                            },
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let recommended_solution = parsed["recommended_solution"].as_u64().unwrap_or(0) as usize;

        Ok(ErrorExplanation {
            error_type,
            reason,
            possible_causes,
            solutions,
            recommended_solution,
            documentation_links: vec![],
        })
    }
}

impl Default for ErrorExplainer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_explainer_creation() {
        let explainer = ErrorExplainer::new();
        // Just verify it can be created
        assert!(explainer
            .pattern_matcher
            .match_pattern("kubectl error")
            .is_none());
    }
}
