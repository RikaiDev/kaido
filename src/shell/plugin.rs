pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn on_command(&self, _cmd: &str) -> HookResult {
        HookResult::None
    }
}

pub enum HookResult {
    Modified(String),
    Suggestion(String),
    None,
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

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn on_command(&self, cmd: &str) -> Option<HookResult> {
        for plugin in &self.plugins {
            let result = plugin.on_command(cmd);
            if !matches!(result, HookResult::None) {
                return Some(result);
            }
        }
        None
    }
}
