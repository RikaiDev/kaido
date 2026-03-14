use crate::config::CopilotConfig;
use crate::tools::{LLMBackend, LLMResponse};
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct CopilotBackend {
    client: Client,
    config: CopilotConfig,
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
        Self {
            client: Client::new(),
            config: CopilotConfig::default(),
        }
    }
    
    pub fn with_config(config: CopilotConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }
    
    pub fn is_available(&self) -> bool {
        !self.get_token().is_empty()
    }
    
    fn get_token(&self) -> String {
        if !self.config.token.is_empty() {
            self.config.token.clone()
        } else {
            CopilotConfig::load_token().unwrap_or_default()
        }
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
        let token = self.get_token();
        
        if token.is_empty() {
            return Err(anyhow::anyhow!(
                "Copilot not configured.\n\n\
                Run: opencode providers login copilot\n\
                Then use Copilot in Kaido!"
            ));
        }
        
        let request = CopilotRequest {
            model: self.config.model.clone(),
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
        
        let url = format!("{}/v1/chat/completions", self.config.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {token}"))
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
