use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    pub providers: std::collections::HashMap<String, ProviderAuth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAuth {
    #[serde(rename = "type")]
    pub auth_type: String,
    pub refresh: Option<String>,
    pub access: Option<String>,
    pub key: Option<String>,
    pub expires: Option<i64>,
}

impl AuthConfig {
    pub fn load() -> Self {
        let path = Self::path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
        Self::default()
    }

    pub fn path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("opencode")
            .join("auth.json")
    }

    pub fn get_copilot_token(&self) -> Option<String> {
        self.providers
            .get("github-copilot")
            .and_then(|p| p.access.clone().or(p.refresh.clone()))
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
