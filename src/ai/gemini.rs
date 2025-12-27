// Gemini AI Backend
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::tools::{LLMBackend, LLMResponse};

const GEMINI_API_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash-exp:generateContent";

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Debug, Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContentResponse,
}

#[derive(Debug, Deserialize)]
struct GeminiContentResponse {
    parts: Vec<GeminiPartResponse>,
}

#[derive(Debug, Deserialize)]
struct GeminiPartResponse {
    text: String,
}

pub struct GeminiBackend {
    api_key: String,
    client: reqwest::Client,
}

impl GeminiBackend {
    /// Create new Gemini backend
    /// 
    /// API key is loaded from:
    /// 1. Environment variable: GEMINI_API_KEY
    /// 2. Config file: ~/.config/kaido/config.toml
    /// 
    /// Get your API key from: https://aistudio.google.com/app/apikey
    pub fn new() -> Self {
        let api_key = Self::load_api_key();
        
        Self {
            api_key,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to build reqwest client"),
        }
    }
    
    /// Load API key from environment or config
    fn load_api_key() -> String {
        // 1. Try environment variable first
        if let Ok(key) = std::env::var("GEMINI_API_KEY") {
            if !key.is_empty() {
                log::info!("[OK] Gemini API key loaded from environment variable");
                return key;
            }
        }
        
        // 2. Try config file
        if let Ok(config) = crate::config::Config::load() {
            if let Some(key) = &config.gemini_api_key {
                if !key.is_empty() {
                    log::info!("[OK] Gemini API key loaded from config file");
                    return key.clone();
                }
            }
        }
        
        // 3. No API key found - return empty string and fail on first use
        log::warn!("[!] Gemini API key not found. Please set GEMINI_API_KEY environment variable or configure in ~/.config/kaido/config.toml");
        String::new()
    }
    
    /// Create Gemini backend with explicit API key
    pub fn with_api_key(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to build reqwest client"),
        }
    }
}

#[async_trait]
impl LLMBackend for GeminiBackend {
    async fn infer(&self, prompt: &str) -> Result<LLMResponse> {
        // Check if API key is configured
        if self.api_key.is_empty() {
            return Err(anyhow::anyhow!(
                "Gemini API key not configured.\n\
                Please set your API key using one of:\n\
                1. Environment variable: export GEMINI_API_KEY=your_key_here\n\
                2. Config file: ~/.config/kaido/config.toml\n\
                \n\
                Get your API key from: https://aistudio.google.com/app/apikey"
            ));
        }
        
        log::info!("[AI] Calling Gemini API...");
        
        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: prompt.to_string(),
                }],
            }],
        };
        
        let url = format!("{}?key={}", GEMINI_API_URL, self.api_key);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Gemini API error ({}): {}",
                status,
                error_text
            ));
        }
        
        let gemini_response: GeminiResponse = response.json().await?;
        
        let text = gemini_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .ok_or_else(|| anyhow::anyhow!("Gemini returned no content"))?;
        
        log::info!("[OK] Gemini response successful");
        
        // 返回 LLMResponse
        Ok(LLMResponse {
            command: extract_command(&text).unwrap_or_default(),
            confidence: 85,
            reasoning: text,
        })
    }
}

/// 從 AI 回應中提取命令
fn extract_command(text: &str) -> Option<String> {
    // 尋找 code block 中的命令
    if let Some(start) = text.find("```") {
        if let Some(end) = text[start + 3..].find("```") {
            let code = &text[start + 3..start + 3 + end];
            // 跳過語言標記
            let code = code.lines()
                .skip_while(|line| line.trim().is_empty() || 
                    line.trim() == "bash" || 
                    line.trim() == "sh")
                .collect::<Vec<_>>()
                .join("\n");
            return Some(code.trim().to_string());
        }
    }
    None
}

impl Default for GeminiBackend {
    fn default() -> Self {
        Self::new()
    }
}

