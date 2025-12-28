// OpenAI API integration for kubectl translation
// Based on contracts/openai-api-contract.md
// NOTE: Currently not used - Gemini API is used instead

#![allow(dead_code)]

use crate::config::OpenAIConfig;
use crate::kubectl::{KubectlContext, TranslationResult};
use serde::{Deserialize, Serialize};

/// OpenAI API request message
#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

/// OpenAI API request body
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
    response_format: ResponseFormat,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
}

/// OpenAI API response
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChoiceMessage {
    content: String,
}

/// Translation content parsed from OpenAI response
#[derive(Debug, Deserialize)]
struct TranslationContent {
    command: String,
    confidence: u8,
    reasoning: String,
}

/// Build system prompt per contracts/openai-api-contract.md
pub fn build_system_prompt(context: &KubectlContext) -> String {
    format!(
        r#"You are a kubectl expert assistant. Translate natural language requests into valid kubectl commands.

CURRENT CONTEXT:
- Cluster: {}
- Namespace: {}
- Environment: {}

SUPPORTED OPERATIONS:
get, describe, logs, delete, scale, apply, create, patch, edit, exec, port-forward, drain, cordon, uncordon, top, rollout, label, annotate, cp, auth

RULES:
1. Return ONLY valid JSON with this exact structure:
   {{
     "command": "kubectl [subcommand] [args]",
     "confidence": <0-100>,
     "reasoning": "<explanation>"
   }}

2. If the request is ambiguous (missing pod name, namespace, resource type), set confidence below 70 and include "NEEDS_CLARIFICATION: [specific question]" in reasoning.

3. Always use the current namespace unless user explicitly specifies another with "-n" or "--namespace".

4. Never return commands that:
   - Use absolute paths or file references
   - Require interactive input (kubectl exec -it is OK, but note it's interactive)
   - Include shell pipes or redirects

5. For destructive operations (delete, drain), ensure resource name is explicitly provided. If not, set confidence below 70.

EXAMPLES:
User: "show all pods"
Response: {{"command": "kubectl get pods -n {}", "confidence": 95, "reasoning": "Standard pod listing in current namespace"}}

User: "delete deployment nginx"
Response: {{"command": "kubectl delete deployment nginx -n {}", "confidence": 90, "reasoning": "Explicit deployment deletion with resource name provided"}}

User: "show logs"
Response: {{"command": "kubectl logs", "confidence": 40, "reasoning": "NEEDS_CLARIFICATION: Which pod? Command requires pod name (e.g., 'kubectl logs <pod-name>')"}}

User: "scale my api to 5"
Response: {{"command": "kubectl scale deployment api --replicas=5 -n {}", "confidence": 75, "reasoning": "Assuming 'api' is deployment name. If it's a different resource type, please specify."}}

Now translate the following request:"#,
        context.cluster,
        context.effective_namespace(),
        context.environment_type.as_str(),
        context.effective_namespace(),
        context.effective_namespace(),
        context.effective_namespace(),
    )
}

/// Build user prompt
pub fn build_user_prompt(input: &str, context: &KubectlContext) -> String {
    format!(
        r#"Natural language request: "{}"

Context reminder:
- Current cluster: {}
- Current namespace: {}
- Environment type: {}

Provide your response as JSON with command, confidence, and reasoning fields."#,
        input,
        context.cluster,
        context.effective_namespace(),
        context.environment_type.as_str(),
    )
}

/// Translate natural language to kubectl command via OpenAI API
pub async fn translate_to_kubectl(
    input: &str,
    context: &KubectlContext,
    config: &OpenAIConfig,
) -> anyhow::Result<TranslationResult> {
    // Build prompts
    let system_prompt = build_system_prompt(context);
    let user_prompt = build_user_prompt(input, context);

    // Create HTTP client with timeout
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout_seconds))
        .build()?;

    // Build request
    let request_body = OpenAIRequest {
        model: config.model.clone(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt,
            },
            Message {
                role: "user".to_string(),
                content: user_prompt,
            },
        ],
        temperature: 0.3,
        max_tokens: 500,
        response_format: ResponseFormat {
            format_type: "json_object".to_string(),
        },
    };

    // Make API call with retry on network error
    let mut last_error = None;
    for attempt in 0..=1 {
        if attempt > 0 {
            log::info!("Retrying OpenAI API call (attempt {}/2)", attempt + 1);
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }

        match make_openai_request(&client, &config.base_url, &config.api_key, &request_body).await {
            Ok(response) => {
                // Parse response
                return parse_translation_response(&response);
            }
            Err(e) => {
                last_error = Some(e);
                // Only retry on network errors, not API errors
                if attempt == 0 && is_retryable_error(last_error.as_ref().unwrap()) {
                    continue;
                } else {
                    break;
                }
            }
        }
    }

    // If all retries failed, return error with fallback suggestion
    Err(handle_openai_error(last_error.unwrap()))
}

/// Make OpenAI API request
async fn make_openai_request(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    request_body: &OpenAIRequest,
) -> Result<OpenAIResponse, anyhow::Error> {
    let response = client
        .post(format!("{base_url}/chat/completions"))
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(request_body)
        .send()
        .await?;

    let status = response.status();

    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow::anyhow!("OpenAI API error ({status}): {error_text}"));
    }

    let openai_response: OpenAIResponse = response.json().await?;
    Ok(openai_response)
}

/// Parse OpenAI response to TranslationResult
fn parse_translation_response(response: &OpenAIResponse) -> anyhow::Result<TranslationResult> {
    let content_json = &response
        .choices
        .first()
        .ok_or_else(|| anyhow::anyhow!("No choices in OpenAI response"))?
        .message
        .content;

    let translation: TranslationContent = serde_json::from_str(content_json)
        .map_err(|e| anyhow::anyhow!("Failed to parse OpenAI JSON response: {e}"))?;

    // Validate command starts with "kubectl "
    if !translation.command.trim().starts_with("kubectl ") {
        return Err(anyhow::anyhow!(
            "Invalid command from AI: does not start with 'kubectl '. Got: {}",
            translation.command
        ));
    }

    // Validate confidence range
    if translation.confidence > 100 {
        return Err(anyhow::anyhow!(
            "Invalid confidence score: {} (must be 0-100)",
            translation.confidence
        ));
    }

    Ok(TranslationResult::new(
        translation.command,
        translation.confidence,
        translation.reasoning,
    ))
}

/// Check if error is retryable (network errors only)
fn is_retryable_error(error: &anyhow::Error) -> bool {
    let error_string = error.to_string().to_lowercase();
    error_string.contains("timeout")
        || error_string.contains("connection")
        || error_string.contains("network")
}

/// Handle OpenAI API errors with user-friendly messages
fn handle_openai_error(error: anyhow::Error) -> anyhow::Error {
    let error_string = error.to_string();

    if error_string.contains("401") || error_string.contains("Incorrect API key") {
        return anyhow::anyhow!(
            "Invalid OpenAI API key. Check your configuration in ~/.kaido/config.toml\nOriginal error: {error}"
        );
    }

    if error_string.contains("429") || error_string.contains("Rate limit") {
        return anyhow::anyhow!(
            "OpenAI API rate limit exceeded. Try again in 60 seconds or enter kubectl command manually.\nOriginal error: {error}"
        );
    }

    if error_string.contains("500") || error_string.contains("503") {
        return anyhow::anyhow!(
            "OpenAI service unavailable. Enter kubectl command manually.\nOriginal error: {error}"
        );
    }

    if error_string.contains("timeout") {
        return anyhow::anyhow!(
            "OpenAI request timed out. Enter kubectl command manually.\nOriginal error: {error}"
        );
    }

    // Generic error with fallback suggestion
    anyhow::anyhow!(
        "OpenAI translation failed. Enter kubectl command manually.\nOriginal error: {error}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_system_prompt() {
        let context = KubectlContext::new(
            "prod-cluster".to_string(),
            "production".to_string(),
            Some("my-namespace".to_string()),
            "admin".to_string(),
        );

        let prompt = build_system_prompt(&context);

        assert!(prompt.contains("Cluster: production"));
        assert!(prompt.contains("Namespace: my-namespace"));
        assert!(prompt.contains("Environment: production"));
        assert!(prompt.contains("SUPPORTED OPERATIONS:"));
    }

    #[test]
    fn test_build_user_prompt() {
        let context = KubectlContext::new(
            "dev-cluster".to_string(),
            "development".to_string(),
            None,
            "user".to_string(),
        );

        let prompt = build_user_prompt("show pods", &context);

        assert!(prompt.contains("show pods"));
        assert!(prompt.contains("Current cluster: development"));
        assert!(prompt.contains("Current namespace: default"));
    }

    #[test]
    fn test_parse_valid_translation() {
        let json = r#"{"command": "kubectl get pods -n default", "confidence": 95, "reasoning": "Standard listing"}"#;
        let response = OpenAIResponse {
            choices: vec![Choice {
                message: ChoiceMessage {
                    content: json.to_string(),
                },
            }],
        };

        let result = parse_translation_response(&response).unwrap();
        assert_eq!(result.kubectl_command, "kubectl get pods -n default");
        assert_eq!(result.confidence_score, 95);
        assert!(result.is_valid_command());
    }

    #[test]
    fn test_parse_invalid_command() {
        let json = r#"{"command": "docker ps", "confidence": 50, "reasoning": "Wrong tool"}"#;
        let response = OpenAIResponse {
            choices: vec![Choice {
                message: ChoiceMessage {
                    content: json.to_string(),
                },
            }],
        };

        let result = parse_translation_response(&response);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("does not start with 'kubectl '"));
    }

    #[test]
    fn test_is_retryable_error() {
        let timeout_err = anyhow::anyhow!("Request timeout");
        assert!(is_retryable_error(&timeout_err));

        let network_err = anyhow::anyhow!("Network connection failed");
        assert!(is_retryable_error(&network_err));

        let api_err = anyhow::anyhow!("401 Unauthorized");
        assert!(!is_retryable_error(&api_err));
    }
}
