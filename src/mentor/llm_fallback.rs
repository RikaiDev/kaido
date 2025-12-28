// LLM Fallback for Mentor System
//
// When pattern matching doesn't find a match, use the LLM to generate
// educational guidance for unknown errors.

use anyhow::Result;
use serde::Deserialize;

use super::guidance::{GuidanceSource, MentorGuidance, NextStep};
use super::types::ErrorInfo;
use crate::tools::LLMBackend;

/// LLM-based mentor guidance generator
pub struct LLMMentor;

/// Response structure expected from LLM
#[derive(Debug, Deserialize)]
struct LLMResponse {
    key_message: String,
    explanation: String,
    #[serde(default)]
    search_keywords: Vec<String>,
    #[serde(default)]
    next_steps: Vec<LLMNextStep>,
    #[serde(default)]
    related_concepts: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LLMNextStep {
    description: String,
    command: Option<String>,
}

impl LLMMentor {
    /// Generate mentor guidance using LLM
    pub async fn generate(error: &ErrorInfo, llm: &dyn LLMBackend) -> Result<MentorGuidance> {
        let prompt = Self::build_prompt(error);
        let response = llm.infer(&prompt).await?;

        // Try to parse as JSON
        Self::parse_response(&response.reasoning, error)
    }

    /// Build the prompt for the LLM
    fn build_prompt(error: &ErrorInfo) -> String {
        // Truncate output if too long
        let output = if error.full_output.len() > 1000 {
            format!("{}...(truncated)", &error.full_output[..1000])
        } else {
            error.full_output.clone()
        };

        format!(
            r#"You are a patient mentor teaching a beginner about command-line errors.

The user ran a command that failed:
- Command: {command}
- Exit code: {exit_code}
- Error type: {error_type}
- Error output:
```
{output}
```

Provide educational guidance in this exact JSON format (no markdown, just raw JSON):
{{
  "key_message": "The most important part of the error (1 sentence)",
  "explanation": "What this error means in simple terms (2-3 sentences)",
  "search_keywords": ["keyword1", "keyword2"],
  "next_steps": [
    {{"description": "What to do first", "command": "actual command or null"}},
    {{"description": "What to do next", "command": null}}
  ],
  "related_concepts": ["concept to learn about"]
}}

Important:
- Focus on TEACHING, not just fixing
- Explain WHY this error happens
- Give practical, actionable steps
- Keep explanations simple for beginners
- Include 2-3 next steps
- Include 1-2 search keywords
- Return ONLY valid JSON, no other text"#,
            command = error.command,
            exit_code = error.exit_code,
            error_type = error.error_type.name(),
            output = output
        )
    }

    /// Parse the LLM response into MentorGuidance
    fn parse_response(response: &str, error: &ErrorInfo) -> Result<MentorGuidance> {
        // Try to extract JSON from the response
        let json_str = Self::extract_json(response);

        match serde_json::from_str::<LLMResponse>(&json_str) {
            Ok(parsed) => {
                let next_steps: Vec<NextStep> = parsed
                    .next_steps
                    .into_iter()
                    .map(|s| {
                        if let Some(cmd) = s.command {
                            NextStep::with_command(s.description, cmd)
                        } else {
                            NextStep::new(s.description)
                        }
                    })
                    .collect();

                Ok(MentorGuidance {
                    key_message: parsed.key_message,
                    explanation: parsed.explanation,
                    search_keywords: parsed.search_keywords,
                    next_steps,
                    related_concepts: parsed.related_concepts,
                    source: GuidanceSource::LLM,
                })
            }
            Err(e) => {
                log::warn!("Failed to parse LLM response as JSON: {e}");
                log::debug!("Response was: {response}");

                // Return a basic guidance with the raw response as explanation
                Ok(MentorGuidance {
                    key_message: error.key_message.clone(),
                    explanation: Self::extract_explanation(response),
                    search_keywords: Vec::new(),
                    next_steps: Vec::new(),
                    related_concepts: Vec::new(),
                    source: GuidanceSource::LLM,
                })
            }
        }
    }

    /// Try to extract JSON from a response that might have extra text
    fn extract_json(response: &str) -> String {
        let response = response.trim();

        // If it starts with {, assume it's JSON
        if response.starts_with('{') {
            // Find matching closing brace
            let mut depth = 0;
            let mut end = 0;
            for (i, c) in response.chars().enumerate() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end = i + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if end > 0 {
                return response[..end].to_string();
            }
        }

        // Try to find JSON in markdown code block
        if let Some(start) = response.find("```json") {
            let json_start = start + 7;
            if let Some(end) = response[json_start..].find("```") {
                return response[json_start..json_start + end].trim().to_string();
            }
        }

        // Try to find JSON in generic code block
        if let Some(start) = response.find("```") {
            let block_start = start + 3;
            // Skip language identifier if present
            let content_start = response[block_start..]
                .find('\n')
                .map(|i| block_start + i + 1)
                .unwrap_or(block_start);
            if let Some(end) = response[content_start..].find("```") {
                return response[content_start..content_start + end]
                    .trim()
                    .to_string();
            }
        }

        // Try to find { ... } anywhere
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                if end > start {
                    return response[start..=end].to_string();
                }
            }
        }

        response.to_string()
    }

    /// Extract a usable explanation from non-JSON response
    fn extract_explanation(response: &str) -> String {
        let response = response.trim();

        // Take first paragraph or first 200 chars
        let first_para = response.split("\n\n").next().unwrap_or(response);

        if first_para.len() > 200 {
            format!("{}...", &first_para[..200])
        } else {
            first_para.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mentor::types::ErrorType;

    fn create_test_error() -> ErrorInfo {
        ErrorInfo::new(
            ErrorType::CommandNotFound,
            127,
            "command not found: foo",
            "foo --bar",
        )
        .with_output("bash: foo: command not found")
    }

    #[test]
    fn test_build_prompt() {
        let error = create_test_error();
        let prompt = LLMMentor::build_prompt(&error);

        assert!(prompt.contains("foo --bar"));
        assert!(prompt.contains("127"));
        assert!(prompt.contains("Command Not Found"));
        assert!(prompt.contains("JSON"));
    }

    #[test]
    fn test_extract_json_direct() {
        let response = r#"{"key_message": "test", "explanation": "test"}"#;
        let json = LLMMentor::extract_json(response);
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
    }

    #[test]
    fn test_extract_json_from_markdown() {
        let response = r#"Here's the guidance:
```json
{"key_message": "test", "explanation": "test"}
```
"#;
        let json = LLMMentor::extract_json(response);
        assert!(json.contains("key_message"));
    }

    #[test]
    fn test_extract_json_with_extra_text() {
        let response = r#"Let me help you:
{"key_message": "test", "explanation": "test"}
Hope this helps!"#;
        let json = LLMMentor::extract_json(response);
        assert!(json.starts_with('{'));
    }

    #[test]
    fn test_parse_valid_response() {
        let error = create_test_error();
        let response = r#"{
            "key_message": "command not found",
            "explanation": "The command foo is not installed",
            "search_keywords": ["install foo"],
            "next_steps": [
                {"description": "Install foo", "command": "brew install foo"}
            ],
            "related_concepts": ["PATH"]
        }"#;

        let guidance = LLMMentor::parse_response(response, &error).unwrap();
        assert_eq!(guidance.key_message, "command not found");
        assert_eq!(guidance.source, GuidanceSource::LLM);
        assert_eq!(guidance.next_steps.len(), 1);
    }

    #[test]
    fn test_parse_invalid_response_fallback() {
        let error = create_test_error();
        let response = "This is not valid JSON but contains useful information.";

        let guidance = LLMMentor::parse_response(response, &error).unwrap();
        // Should fall back gracefully
        assert_eq!(guidance.source, GuidanceSource::LLM);
        assert!(!guidance.explanation.is_empty());
    }

    #[test]
    fn test_extract_explanation() {
        let response = "First paragraph with explanation.\n\nSecond paragraph.";
        let explanation = LLMMentor::extract_explanation(response);
        assert_eq!(explanation, "First paragraph with explanation.");
    }
}
