pub mod explainer;
pub mod gemini;
pub mod ollama;

pub use explainer::CommandExplainer;
pub use gemini::GeminiBackend;
pub use ollama::{ModelRecommendation, OllamaBackend, OllamaStatus};

use crate::config::{AIProvider, Config};
use crate::kubectl::{KubectlContext, TranslationResult};
use crate::tools::{LLMBackend, LLMResponse};
use anyhow::Result;
use async_trait::async_trait;

/// AI Manager - Handles inference with multiple backends
/// Supports: Gemini API (cloud) and Ollama (local)
pub struct AIManager {
    gemini: GeminiBackend,
    ollama: OllamaBackend,
    provider: AIProvider,
}

impl AIManager {
    /// Create a new AI manager with config
    pub fn new(config: Config) -> Self {
        Self {
            gemini: GeminiBackend::new(),
            ollama: OllamaBackend::with_config(config.ollama.clone()),
            provider: config.provider.clone(),
        }
    }

    /// Translate natural language to kubectl command
    pub async fn translate_kubectl(
        &self,
        input: &str,
        context: &KubectlContext,
    ) -> crate::utils::KaidoResult<TranslationResult> {
        log::info!("Attempting kubectl translation");

        // Build kubectl-specific prompt
        let namespace = context.namespace.as_deref().unwrap_or("default");
        let prompt = format!(
            "Translate this natural language request into a kubectl command.\n\
            Current Kubernetes context:\n\
            - Cluster: {}\n\
            - Namespace: {}\n\
            - Environment: {}\n\n\
            User request: {}\n\n\
            Respond ONLY with a JSON object in this exact format:\n\
            {{\n  \"command\": \"kubectl ...\",\n  \"confidence\": 85,\n  \"reasoning\": \"explanation\"\n}}",
            context.cluster,
            namespace,
            context.environment_type.as_str(),
            input
        );

        // Use configured provider
        let response_text = self
            .infer(&prompt)
            .await
            .map_err(|e| crate::utils::KaidoError::ModelError {
                message: e.to_string(),
                model_name: "ai".to_string(),
            })?
            .reasoning;

        // Parse JSON response
        #[derive(serde::Deserialize)]
        struct KubectlResponse {
            command: String,
            confidence: u8,
            reasoning: String,
        }

        match serde_json::from_str::<KubectlResponse>(&response_text) {
            Ok(parsed) => {
                log::info!("Kubectl translation successful: {}", parsed.command);
                Ok(TranslationResult {
                    kubectl_command: parsed.command,
                    confidence_score: parsed.confidence,
                    reasoning: parsed.reasoning,
                })
            }
            Err(e) => {
                log::warn!("Failed to parse AI output as JSON: {e}");
                Err(crate::utils::KaidoError::ModelError {
                    message: format!("AI returned invalid JSON: {e}"),
                    model_name: "ai".to_string(),
                })
            }
        }
    }

    /// Infer using the configured provider strategy
    async fn infer_with_provider(&self, prompt: &str) -> Result<LLMResponse> {
        match &self.provider {
            AIProvider::Gemini => {
                log::info!("Using Gemini API (configured)");
                self.gemini.infer(prompt).await
            }
            AIProvider::Ollama => {
                log::info!("Using Ollama (configured)");
                self.ollama.infer(prompt).await
            }
            AIProvider::Auto => {
                // Auto: Try Gemini first, then Ollama
                log::info!("Auto mode: trying Gemini API first");
                match self.gemini.infer(prompt).await {
                    Ok(response) => {
                        log::info!("[OK] Gemini API successful");
                        Ok(response)
                    }
                    Err(gemini_err) => {
                        log::warn!("Gemini failed: {gemini_err}, trying Ollama");

                        match self.ollama.infer(prompt).await {
                            Ok(response) => {
                                log::info!("[OK] Ollama successful");
                                Ok(response)
                            }
                            Err(ollama_err) => {
                                log::error!("All AI backends failed");
                                Err(anyhow::anyhow!(
                                    "All AI backends failed:\n\
                                    - Gemini: {gemini_err}\n\
                                    - Ollama: {ollama_err}\n\n\
                                    Please ensure either:\n\
                                    1. GEMINI_API_KEY is set, or\n\
                                    2. Ollama is running (ollama serve)"
                                ))
                            }
                        }
                    }
                }
            }
        }
    }
}

// Implement LLMBackend trait for AIManager
#[async_trait]
impl LLMBackend for AIManager {
    async fn infer(&self, prompt: &str) -> Result<LLMResponse> {
        self.infer_with_provider(prompt).await
    }
}
