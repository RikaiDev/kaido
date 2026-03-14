use crate::tools::{LLMBackend, LLMResponse};
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct CopilotBackend {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

#[derive(Serialize)]
struct CopilotRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct CopilotResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

impl CopilotBackend {
    pub fn new() -> Self {
        let api_key = std::env::var("GITHUB_COPILOT_TOKEN")
            .or_else(|_| std::env::var("COPILOT_TOKEN"))
            .unwrap_or_default();
        
        Self {
            client: Client::new(),
            api_key,
            model: "gpt-4o".to_string(),
            base_url: "https://api.github.com".to_string(),
        }
    }
    
    pub fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
}

impl Default for CopilotBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LLMBackend for CopilotBackend {
    async fn infer(&self, prompt: &str) -> Result<LLMResponse> {
        if !self.is_available() {
            anyhow::bail!("Copilot token not set. Set GITHUB_COPILOT_TOKEN environment variable.");
        }
        
        let request = CopilotRequest {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a DevOps assistant. Translate natural language to shell commands. Respond with just the command, no explanation.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            temperature: 0.3,
            max_tokens: 256,
        };
        
        let url = format!("{}/v1/chat/completions", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&request)
            .send()
            .await
            .context("Failed to call Copilot API")?;
        
        let result: CopilotResponse = response
            .json()
            .await
            .context("Failed to parse Copilot response")?;
        
        let content = result.choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();
        
        Ok(LLMResponse {
            command: content,
            confidence: 85,
            reasoning: "Copilot inference".to_string(),
        })
    }
}
