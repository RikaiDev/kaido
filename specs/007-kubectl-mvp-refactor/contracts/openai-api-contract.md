# OpenAI API Contract

**Feature**: Kubectl-Only MVP (60-Day Reality Check)
**API Version**: OpenAI Chat Completions v1
**Endpoint**: `https://api.openai.com/v1/chat/completions`

## Overview

This document defines the contract between Kaido AI Shell and the OpenAI GPT-4 API for kubectl command translation.

---

## Request Specification

### HTTP Method
```
POST /v1/chat/completions
```

### Headers
```http
Content-Type: application/json
Authorization: Bearer {API_KEY}
```

### Request Body

```json
{
  "model": "gpt-4-turbo-preview",
  "messages": [
    {
      "role": "system",
      "content": "{SYSTEM_PROMPT}"
    },
    {
      "role": "user",
      "content": "{USER_PROMPT}"
    }
  ],
  "temperature": 0.3,
  "max_tokens": 500,
  "response_format": {
    "type": "json_object"
  }
}
```

### System Prompt Template

```text
You are a kubectl expert assistant. Translate natural language requests into valid kubectl commands.

CURRENT CONTEXT:
- Cluster: {cluster_name}
- Namespace: {namespace}
- Environment: {environment_type}

SUPPORTED OPERATIONS:
get, describe, logs, delete, scale, apply, create, patch, edit, exec, port-forward, drain, cordon, uncordon, top, rollout, label, annotate, cp, auth

RULES:
1. Return ONLY valid JSON with this exact structure:
   {
     "command": "kubectl [subcommand] [args]",
     "confidence": <0-100>,
     "reasoning": "<explanation>"
   }

2. If the request is ambiguous (missing pod name, namespace, resource type), set confidence below 70 and include "NEEDS_CLARIFICATION: [specific question]" in reasoning.

3. Always use the current namespace unless user explicitly specifies another with "-n" or "--namespace".

4. Never return commands that:
   - Use absolute paths or file references
   - Require interactive input (kubectl exec -it is OK, but note it's interactive)
   - Include shell pipes or redirects

5. For destructive operations (delete, drain), ensure resource name is explicitly provided. If not, set confidence below 70.

EXAMPLES:
User: "show all pods"
Response: {"command": "kubectl get pods -n {namespace}", "confidence": 95, "reasoning": "Standard pod listing in current namespace"}

User: "delete deployment nginx"
Response: {"command": "kubectl delete deployment nginx -n {namespace}", "confidence": 90, "reasoning": "Explicit deployment deletion with resource name provided"}

User: "show logs"
Response: {"command": "kubectl logs", "confidence": 40, "reasoning": "NEEDS_CLARIFICATION: Which pod? Command requires pod name (e.g., 'kubectl logs <pod-name>')"}

User: "scale my api to 5"
Response: {"command": "kubectl scale deployment api --replicas=5 -n {namespace}", "confidence": 75, "reasoning": "Assuming 'api' is deployment name. If it's a different resource type, please specify."}

Now translate the following request:
```

### User Prompt Template

```text
Natural language request: "{user_input}"

Context reminder:
- Current cluster: {cluster_name}
- Current namespace: {namespace}
- Environment type: {environment_type}

Provide your response as JSON with command, confidence, and reasoning fields.
```

### Parameter Details

| Field | Value | Rationale |
|-------|-------|-----------|
| `model` | `gpt-4-turbo-preview` | Best accuracy for technical domain (kubectl syntax) |
| `temperature` | `0.3` | Low creativity - prefer deterministic kubectl syntax |
| `max_tokens` | `500` | Sufficient for command + reasoning (avg 150-200 tokens) |
| `response_format.type` | `json_object` | Enforces structured JSON output for parsing |

---

## Response Specification

### Success Response (HTTP 200)

```json
{
  "id": "chatcmpl-abc123",
  "object": "chat.completion",
  "created": 1234567890,
  "model": "gpt-4-turbo-preview",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "{\"command\": \"kubectl get pods -n default\", \"confidence\": 95, \"reasoning\": \"Standard pod listing in current namespace\"}"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 450,
    "completion_tokens": 50,
    "total_tokens": 500
  }
}
```

### Expected Content Structure

The `content` field contains a JSON string that must parse to:

```json
{
  "command": "kubectl [subcommand] [args]",
  "confidence": 85,
  "reasoning": "Explanation of translation logic or clarification needed"
}
```

### Field Validation

| Field | Type | Constraints | Validation |
|-------|------|-------------|------------|
| `command` | `string` | Required, non-empty | Must start with "kubectl " |
| `confidence` | `integer` | Required, 0-100 | Reject if outside range |
| `reasoning` | `string` | Required, 1-1000 chars | Truncate if longer |

---

## Error Responses

### 401 Unauthorized

```json
{
  "error": {
    "message": "Incorrect API key provided",
    "type": "invalid_request_error",
    "param": null,
    "code": "invalid_api_key"
  }
}
```

**Handling**: Display error to user: "Invalid OpenAI API key. Check your configuration."

### 429 Rate Limit Exceeded

```json
{
  "error": {
    "message": "Rate limit reached for requests",
    "type": "rate_limit_error",
    "param": null,
    "code": "rate_limit_exceeded"
  }
}
```

**Handling**: Display error: "OpenAI API rate limit exceeded. Try again in 60 seconds or enter kubectl command manually."

### 500 Internal Server Error

```json
{
  "error": {
    "message": "The server had an error while processing your request",
    "type": "server_error",
    "param": null,
    "code": null
  }
}
```

**Handling**: Retry once after 2 seconds. If retry fails, offer fallback: "OpenAI service unavailable. Enter kubectl command manually:"

### Timeout (Client-side)

**Condition**: No response after 10 seconds

**Handling**: Cancel request, display: "OpenAI request timed out. Enter kubectl command manually:"

---

## Client Implementation

### Rust Code Outline

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
    response_format: ResponseFormat,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ResponseFormat {
    r#type: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Deserialize, Serialize)]
struct TranslationContent {
    command: String,
    confidence: u8,
    reasoning: String,
}

async fn translate_to_kubectl(
    user_input: &str,
    context: &KubectlContext,
    api_key: &str,
) -> Result<TranslationContent, TranslationError> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    
    let system_prompt = format_system_prompt(context);
    let user_prompt = format_user_prompt(user_input, context);
    
    let request = OpenAIRequest {
        model: "gpt-4-turbo-preview".to_string(),
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
            r#type: "json_object".to_string(),
        },
    };
    
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await?
        .error_for_status()?;
    
    let openai_response: OpenAIResponse = response.json().await?;
    let content_json = &openai_response.choices[0].message.content;
    
    let translation: TranslationContent = serde_json::from_str(content_json)?;
    
    // Validate command starts with "kubectl "
    if !translation.command.starts_with("kubectl ") {
        return Err(TranslationError::InvalidCommand);
    }
    
    // Validate confidence range
    if translation.confidence > 100 {
        return Err(TranslationError::InvalidConfidence);
    }
    
    Ok(translation)
}
```

---

## Retry Strategy

| Error Type | Retry | Delay | Max Retries | Fallback |
|------------|-------|-------|-------------|----------|
| Network timeout | Yes | 2s | 1 | Manual kubectl input |
| 500 server error | Yes | 2s | 1 | Manual kubectl input |
| 401 unauthorized | No | - | - | Error message |
| 429 rate limit | No | - | - | Wait message + fallback |
| 4xx client error | No | - | - | Error message |

---

## Cost Estimation

**Model**: GPT-4 Turbo (as of 2024)
- Input: $0.01 per 1K tokens
- Output: $0.03 per 1K tokens

**Average Usage per Command**:
- Prompt tokens: ~450 (system prompt + user input)
- Completion tokens: ~50 (JSON response)
- Total: ~500 tokens

**Cost per Command**: 
- Input: 0.45K * $0.01 = $0.0045
- Output: 0.05K * $0.03 = $0.0015
- Total: ~$0.006 (~0.6 cents)

**Monthly Estimate (1 user, 50 commands/day)**:
- 50 commands * 30 days = 1,500 commands
- 1,500 * $0.006 = $9/month per active user

---

## Testing

### Mock Response for Tests

```json
{
  "choices": [
    {
      "message": {
        "content": "{\"command\": \"kubectl get pods -n default\", \"confidence\": 95, \"reasoning\": \"Standard pod listing\"}"
      }
    }
  ]
}
```

### Test Cases

1. **Valid translation**: Input "show pods" → Expect command="kubectl get pods"
2. **Low confidence**: Input "show logs" → Expect confidence < 70, reasoning contains "NEEDS_CLARIFICATION"
3. **Destructive command**: Input "delete all pods" → Expect command="kubectl delete pods --all", confidence > 80
4. **Timeout simulation**: Delay 11 seconds → Expect timeout error
5. **Invalid API key**: 401 response → Expect auth error message

---

## Security Considerations

1. **API Key Storage**: Store in `~/.kaido/config.toml` with file permissions 600
2. **Command Sanitization**: Validate that returned command starts with "kubectl " (no shell injection)
3. **No Sensitive Data in Prompts**: Do not send kubeconfig credentials or secrets to OpenAI
4. **Audit Logging**: Log all OpenAI interactions to audit.db (input + output)

---

## Rate Limits

**OpenAI Standard Tier**:
- 10,000 requests per minute (RPM)
- 2,000,000 tokens per minute (TPM)

**Expected Usage**: 
- Beta users: 5 users * 50 commands/day = 250 commands/day = 0.17 RPM (well under limit)

**No rate limiting required in MVP** - natural user pacing (typing) prevents burst requests.


