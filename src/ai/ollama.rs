// Ollama AI Backend - Local LLM inference via Ollama REST API
use crate::config::OllamaConfig;
use crate::tools::{LLMBackend, LLMResponse};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Ollama API request structure
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

/// Ollama API response structure
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

/// Ollama API error response
#[derive(Debug, Deserialize)]
struct OllamaError {
    error: String,
}

/// Ollama backend for local LLM inference
pub struct OllamaBackend {
    config: OllamaConfig,
    client: reqwest::Client,
}

impl OllamaBackend {
    /// Create new Ollama backend with default config
    pub fn new() -> Self {
        Self::with_config(OllamaConfig::default())
    }

    /// Create Ollama backend with custom config
    pub fn with_config(config: OllamaConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to build reqwest client");

        Self { config, client }
    }

    /// Check if Ollama is running and accessible
    pub async fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", self.config.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.config.base_url);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Ollama not available at {}",
                self.config.base_url
            ));
        }

        #[derive(Deserialize)]
        struct TagsResponse {
            models: Vec<ModelInfo>,
        }

        #[derive(Deserialize)]
        struct ModelInfo {
            name: String,
        }

        let tags: TagsResponse = response.json().await?;
        Ok(tags.models.into_iter().map(|m| m.name).collect())
    }

    /// Get configured model name
    pub fn model_name(&self) -> &str {
        &self.config.model
    }

    /// Get Ollama server status including version and available models
    pub async fn get_status(&self) -> Result<OllamaStatus> {
        let available = self.is_available().await;
        if !available {
            return Ok(OllamaStatus {
                available: false,
                version: None,
                models: vec![],
                recommended_model: None,
            });
        }

        let models = self.list_models().await.unwrap_or_default();
        let recommended = Self::recommend_model(&models);

        Ok(OllamaStatus {
            available: true,
            version: None, // Could add version endpoint if needed
            models,
            recommended_model: recommended,
        })
    }

    /// Recommend a model based on available models
    /// Prioritizes: codestral > qwen2.5 > llama3.2 > mistral > others
    pub fn recommend_model(available_models: &[String]) -> Option<String> {
        // Priority order for model recommendations
        let priority = [
            ("codestral", "Best for code understanding"),
            ("qwen2.5:14b", "Excellent balance of speed and accuracy"),
            ("qwen2.5:7b", "Good balance for most systems"),
            ("qwen2.5", "Balanced performance"),
            ("llama3.2:3b", "Fast, good for basic tasks"),
            ("llama3.2", "Fast, good for basic tasks"),
            ("mistral", "Good general purpose"),
        ];

        for (model_prefix, _) in priority {
            for available in available_models {
                if available.starts_with(model_prefix) {
                    return Some(available.clone());
                }
            }
        }

        // Return first available model if none match priority
        available_models.first().cloned()
    }

    /// Get model recommendations based on system capabilities
    pub fn get_model_recommendations() -> Vec<ModelRecommendation> {
        vec![
            ModelRecommendation {
                model: "llama3.2:3b".to_string(),
                size_gb: 2.0,
                description: "Fast, basic diagnosis. Good for older hardware.".to_string(),
                min_ram_gb: 4,
            },
            ModelRecommendation {
                model: "qwen2.5:7b".to_string(),
                size_gb: 4.5,
                description: "Balanced performance. Recommended for most users.".to_string(),
                min_ram_gb: 8,
            },
            ModelRecommendation {
                model: "qwen2.5:14b".to_string(),
                size_gb: 9.0,
                description: "Higher accuracy. Good for complex errors.".to_string(),
                min_ram_gb: 16,
            },
            ModelRecommendation {
                model: "codestral:22b".to_string(),
                size_gb: 12.0,
                description: "Best accuracy for code and DevOps. Requires good GPU.".to_string(),
                min_ram_gb: 24,
            },
        ]
    }
}

/// Ollama server status
#[derive(Debug, Clone)]
pub struct OllamaStatus {
    /// Whether Ollama is running and accessible
    pub available: bool,
    /// Ollama version (if available)
    pub version: Option<String>,
    /// List of installed models
    pub models: Vec<String>,
    /// Recommended model from available ones
    pub recommended_model: Option<String>,
}

/// Model recommendation with system requirements
#[derive(Debug, Clone)]
pub struct ModelRecommendation {
    /// Model name
    pub model: String,
    /// Approximate size in GB
    pub size_gb: f32,
    /// Description of use case
    pub description: String,
    /// Minimum recommended RAM in GB
    pub min_ram_gb: u32,
}

#[async_trait]
impl LLMBackend for OllamaBackend {
    async fn infer(&self, prompt: &str) -> Result<LLMResponse> {
        let url = format!("{}/api/generate", self.config.base_url);

        log::info!("[AI] Calling Ollama API (model: {})...", self.config.model);

        let request = OllamaRequest {
            model: self.config.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    anyhow::anyhow!(
                        "Cannot connect to Ollama at {}. Is Ollama running?\n\
                        Start with: ollama serve",
                        self.config.base_url
                    )
                } else if e.is_timeout() {
                    anyhow::anyhow!(
                        "Ollama request timed out after {}s. Try a smaller model or increase timeout.",
                        self.config.timeout_seconds
                    )
                } else {
                    anyhow::anyhow!("Ollama request failed: {e}")
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            // Try to parse error message
            if let Ok(err) = serde_json::from_str::<OllamaError>(&error_text) {
                if err.error.contains("not found") {
                    return Err(anyhow::anyhow!(
                        "Model '{}' not found. Install with: ollama pull {}",
                        self.config.model,
                        self.config.model
                    ));
                }
                return Err(anyhow::anyhow!("Ollama error: {}", err.error));
            }

            return Err(anyhow::anyhow!("Ollama API error ({status}): {error_text}"));
        }

        let ollama_response: OllamaResponse = response.json().await?;

        log::info!("[OK] Ollama response successful");

        // Extract command from response
        let text = ollama_response.response.trim();
        let command = extract_command(text).unwrap_or_default();

        Ok(LLMResponse {
            command,
            confidence: 80,
            reasoning: text.to_string(),
        })
    }
}

/// Extract command from AI response (looks for code blocks)
fn extract_command(text: &str) -> Option<String> {
    // Look for code block
    if let Some(start) = text.find("```") {
        if let Some(end) = text[start + 3..].find("```") {
            let code = &text[start + 3..start + 3 + end];
            // Skip language marker
            let code = code
                .lines()
                .skip_while(|line| {
                    let trimmed = line.trim();
                    trimmed.is_empty() || trimmed == "bash" || trimmed == "sh" || trimmed == "shell"
                })
                .collect::<Vec<_>>()
                .join("\n");
            return Some(code.trim().to_string());
        }
    }
    None
}

impl Default for OllamaBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_command() {
        let text = "Here's the command:\n```bash\nls -la\n```";
        assert_eq!(extract_command(text), Some("ls -la".to_string()));

        let text = "```\nkubectl get pods\n```";
        assert_eq!(extract_command(text), Some("kubectl get pods".to_string()));

        let text = "No code block here";
        assert_eq!(extract_command(text), None);
    }

    #[test]
    fn test_default_config() {
        let backend = OllamaBackend::new();
        assert_eq!(backend.config.base_url, "http://localhost:11434");
        assert_eq!(backend.config.model, "llama3.2");
    }

    #[test]
    fn test_recommend_model_priority() {
        // Should prefer codestral over others
        let models = vec![
            "llama3.2:3b".to_string(),
            "codestral:22b".to_string(),
            "qwen2.5:7b".to_string(),
        ];
        assert_eq!(
            OllamaBackend::recommend_model(&models),
            Some("codestral:22b".to_string())
        );

        // Should prefer qwen2.5:14b over smaller versions
        let models = vec![
            "llama3.2:3b".to_string(),
            "qwen2.5:14b".to_string(),
            "qwen2.5:7b".to_string(),
        ];
        assert_eq!(
            OllamaBackend::recommend_model(&models),
            Some("qwen2.5:14b".to_string())
        );

        // Should return first if no priority match
        let models = vec!["phi:latest".to_string(), "custom:v1".to_string()];
        assert_eq!(
            OllamaBackend::recommend_model(&models),
            Some("phi:latest".to_string())
        );

        // Empty list returns None
        let models: Vec<String> = vec![];
        assert_eq!(OllamaBackend::recommend_model(&models), None);
    }

    #[test]
    fn test_model_recommendations() {
        let recommendations = OllamaBackend::get_model_recommendations();
        assert!(!recommendations.is_empty());

        // Check first recommendation (smallest model)
        assert_eq!(recommendations[0].model, "llama3.2:3b");
        assert!(recommendations[0].min_ram_gb <= 4);

        // Check recommendations are in order of size
        for i in 1..recommendations.len() {
            assert!(recommendations[i].size_gb >= recommendations[i - 1].size_gb);
        }
    }
}
