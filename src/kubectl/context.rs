use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Kubernetes environment type detected from context name
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvironmentType {
    Development,
    Staging,
    Production,
    Unknown,
}

impl EnvironmentType {
    /// Detect environment type from context name using regex patterns
    pub fn from_context_name(name: &str) -> Self {
        let name_lower = name.to_lowercase();
        
        if name_lower.contains("prod") || name_lower.contains("production") {
            EnvironmentType::Production
        } else if name_lower.contains("stag") || name_lower.contains("staging") {
            EnvironmentType::Staging
        } else if name_lower.contains("dev") || name_lower.contains("development") {
            EnvironmentType::Development
        } else {
            EnvironmentType::Unknown
        }
    }
    
    /// Convert to string for display
    pub fn as_str(&self) -> &'static str {
        match self {
            EnvironmentType::Development => "development",
            EnvironmentType::Staging => "staging",
            EnvironmentType::Production => "production",
            EnvironmentType::Unknown => "unknown",
        }
    }
}

/// Kubectl context parsed from kubeconfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubectlContext {
    pub name: String,
    pub cluster: String,
    pub namespace: Option<String>,
    pub user: String,
    pub environment_type: EnvironmentType,
}

impl KubectlContext {
    /// Create new kubectl context
    pub fn new(
        name: String,
        cluster: String,
        namespace: Option<String>,
        user: String,
    ) -> Self {
        let environment_type = EnvironmentType::from_context_name(&name);
        
        Self {
            name,
            cluster,
            namespace,
            user,
            environment_type,
        }
    }
    
    /// Get effective namespace (default to "default" if not specified)
    pub fn effective_namespace(&self) -> &str {
        self.namespace.as_deref().unwrap_or("default")
    }
    
    /// Parse kubeconfig from file path
    pub fn from_kubeconfig_file(path: &PathBuf) -> anyhow::Result<Self> {
        use serde_yaml::Value;
        use std::fs;
        
        // Read kubeconfig file
        let contents = fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read kubeconfig at {}: {}", path.display(), e))?;
        
        // Parse YAML
        let config: Value = serde_yaml::from_str(&contents)
            .map_err(|e| anyhow::anyhow!("Failed to parse kubeconfig YAML: {e}"))?;
        
        // Extract current-context
        let current_context_name = config["current-context"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No current-context set in kubeconfig"))?
            .to_string();
        
        // Find the matching context
        let contexts = config["contexts"]
            .as_sequence()
            .ok_or_else(|| anyhow::anyhow!("No contexts found in kubeconfig"))?;
        
        let context_entry = contexts
            .iter()
            .find(|ctx| ctx["name"].as_str() == Some(&current_context_name))
            .ok_or_else(|| anyhow::anyhow!("Current context '{current_context_name}' not found in contexts list"))?;
        
        // Extract context details
        let context = &context_entry["context"];
        let cluster = context["cluster"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No cluster specified in context"))?
            .to_string();
        
        let user = context["user"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No user specified in context"))?
            .to_string();
        
        let namespace = context["namespace"]
            .as_str()
            .map(|s| s.to_string());
        
        Ok(Self::new(current_context_name, cluster, namespace, user))
    }
    
    /// Get current kubectl context from default kubeconfig location
    pub fn current() -> anyhow::Result<Self> {
        // Try $KUBECONFIG env var first
        if let Ok(kubeconfig_path) = std::env::var("KUBECONFIG") {
            let path = PathBuf::from(kubeconfig_path);
            return Self::from_kubeconfig_file(&path);
        }
        
        // Fall back to ~/.kube/config
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
        
        let kubeconfig_path = home.join(".kube").join("config");
        
        if !kubeconfig_path.exists() {
            return Err(anyhow::anyhow!(
                "kubectl context not configured. No kubeconfig found at {}. Run 'kubectl config get-contexts'",
                kubeconfig_path.display()
            ));
        }
        
        Self::from_kubeconfig_file(&kubeconfig_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_detection() {
        assert_eq!(
            EnvironmentType::from_context_name("prod-cluster"),
            EnvironmentType::Production
        );
        assert_eq!(
            EnvironmentType::from_context_name("production-us-west"),
            EnvironmentType::Production
        );
        assert_eq!(
            EnvironmentType::from_context_name("staging-env"),
            EnvironmentType::Staging
        );
        assert_eq!(
            EnvironmentType::from_context_name("dev-cluster"),
            EnvironmentType::Development
        );
        assert_eq!(
            EnvironmentType::from_context_name("my-cluster"),
            EnvironmentType::Unknown
        );
    }

    #[test]
    fn test_effective_namespace() {
        let ctx = KubectlContext::new(
            "prod-cluster".to_string(),
            "production".to_string(),
            Some("my-namespace".to_string()),
            "admin".to_string(),
        );
        assert_eq!(ctx.effective_namespace(), "my-namespace");

        let ctx_no_ns = KubectlContext::new(
            "prod-cluster".to_string(),
            "production".to_string(),
            None,
            "admin".to_string(),
        );
        assert_eq!(ctx_no_ns.effective_namespace(), "default");
    }
}

