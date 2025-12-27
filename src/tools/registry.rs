use super::{Tool, KubectlTool, DockerTool, SQLTool, SQLDialect, DrushTool, NginxTool, Apache2Tool, NetworkTool};

/// Tool registry for managing and detecting tools
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new registry with all built-in tools
    pub fn new() -> Self {
        let mut registry = Self { tools: vec![] };
        
        // Register built-in tools
        registry.register(Box::new(KubectlTool::new()));
        registry.register(Box::new(DockerTool::new()));
        registry.register(Box::new(SQLTool::new(SQLDialect::MySQL)));
        registry.register(Box::new(DrushTool::new()));
        
        // Register new ops tools
        registry.register(Box::new(NginxTool::new()));
        registry.register(Box::new(Apache2Tool::new()));
        registry.register(Box::new(NetworkTool::new()));
        
        registry
    }
    
    /// Register a custom tool
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }
    
    /// Automatically detect which tool to use based on input
    /// Returns the tool with highest confidence score (>= 0.5)
    pub fn detect_tool(&self, input: &str) -> Option<&dyn Tool> {
        let mut best_match: Option<(&dyn Tool, f32)> = None;
        
        for tool in &self.tools {
            let score = tool.detect_intent(input);
            if score >= 0.5 {  // At least 50% confidence
                if let Some((_, best_score)) = best_match {
                    if score > best_score {
                        best_match = Some((tool.as_ref(), score));
                    }
                } else {
                    best_match = Some((tool.as_ref(), score));
                }
            }
        }
        
        best_match.map(|(tool, _)| tool)
    }
    
    /// Get tool by name
    pub fn get_tool(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.iter()
            .find(|t| t.name() == name)
            .map(|t| t.as_ref())
    }
    
    /// List all registered tools
    pub fn list_tools(&self) -> Vec<&str> {
        self.tools.iter().map(|t| t.name()).collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ToolRegistry::new();
        let tools = registry.list_tools();
        
        assert!(tools.contains(&"kubectl"));
        assert!(tools.contains(&"docker"));
        assert!(tools.contains(&"mysql"));
        assert!(tools.contains(&"drush"));
    }

    #[test]
    fn test_get_tool_by_name() {
        let registry = ToolRegistry::new();
        
        let kubectl = registry.get_tool("kubectl");
        assert!(kubectl.is_some());
        assert_eq!(kubectl.unwrap().name(), "kubectl");
        
        let nonexistent = registry.get_tool("nonexistent");
        assert!(nonexistent.is_none());
    }
}

