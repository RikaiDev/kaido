pub struct ContextCollector;

impl ContextCollector {
    pub fn new() -> Self {
        Self
    }

    pub fn collect_for_file(&self, _file_path: &str) -> String {
        String::new()
    }

    pub fn collect_for_command(&self, _cmd: &str, _output: &str) -> String {
        String::new()
    }
}

impl Default for ContextCollector {
    fn default() -> Self {
        Self::new()
    }
}
