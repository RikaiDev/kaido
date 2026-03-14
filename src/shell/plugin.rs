use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone)]
pub enum ShellEvent {
    CommandExecuted {
        cmd: String,
        exit_code: i32,
        output: String,
    },
    ErrorOccurred {
        cmd: String,
        error: String,
        exit_code: Option<i32>,
    },
    ConfigEdited {
        file: String,
        error: Option<String>,
    },
    ToolOutput {
        tool: String,
        output: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticContext {
    pub category: String,
    pub commands: Vec<DiagnosticCommand>,
    pub explanation: String,
    pub learn: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticCommand {
    pub cmd: String,
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    pub handled: bool,
    pub context: Option<DiagnosticContext>,
    pub message: Option<String>,
}

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;

    fn event(&self, _event: &ShellEvent) -> PluginResponse {
        PluginResponse {
            handled: false,
            context: None,
            message: None,
        }
    }
}

#[derive(Deserialize)]
pub struct PluginConfig {
    pub plugins: Vec<PluginEntry>,
}

#[derive(Deserialize)]
pub struct PluginEntry {
    pub name: String,
    pub enabled: Option<bool>,
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn load_from_config() -> Result<Self, anyhow::Error> {
        let mut manager = Self::new();

        let config_path = dirs::home_dir()
            .map(|h| h.join(".kaido").join("plugins.toml"))
            .unwrap_or_default();

        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = toml::from_str::<PluginConfig>(&content) {
                for entry in config.plugins {
                    if entry.enabled.unwrap_or(true) {
                        match entry.name.as_str() {
                            "nginx" => {
                                manager.register(Box::new(
                                    crate::shell::plugins::ops::NginxPlugin::new(),
                                ));
                            }
                            "docker" => {
                                manager.register(Box::new(
                                    crate::shell::plugins::ops::DockerPlugin::new(),
                                ));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(manager)
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn emit(&self, event: &ShellEvent) -> Vec<PluginResponse> {
        let mut responses = Vec::new();
        for plugin in &self.plugins {
            let response = plugin.event(event);
            if response.handled || response.context.is_some() || response.message.is_some() {
                responses.push(response);
            }
        }
        responses
    }

    pub fn collect_diagnostics(&self, event: &ShellEvent) -> Vec<DiagnosticContext> {
        self.emit(event)
            .into_iter()
            .filter_map(|r| r.context)
            .collect()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
