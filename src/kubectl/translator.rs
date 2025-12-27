use serde::{Deserialize, Serialize};

/// Result of AI translation from natural language to kubectl command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    /// Generated kubectl command
    pub kubectl_command: String,
    
    /// AI confidence score (0-100)
    pub confidence_score: u8,
    
    /// AI reasoning/explanation
    pub reasoning: String,
}

impl TranslationResult {
    /// Create new translation result
    pub fn new(kubectl_command: String, confidence_score: u8, reasoning: String) -> Self {
        Self {
            kubectl_command,
            confidence_score,
            reasoning,
        }
    }
    
    /// Check if confidence is low (below threshold)
    pub fn is_low_confidence(&self, threshold: u8) -> bool {
        self.confidence_score < threshold
    }
    
    /// Check if command is valid (starts with "kubectl ")
    #[cfg(test)]
    pub fn is_valid_command(&self) -> bool {
        self.kubectl_command.trim().starts_with("kubectl ")
    }
    
    /// Check if reasoning contains clarification request
    #[cfg(test)]
    pub fn needs_clarification(&self) -> bool {
        self.reasoning.contains("NEEDS_CLARIFICATION")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_result() {
        let result = TranslationResult::new(
            "kubectl get pods -n default".to_string(),
            95,
            "Standard pod listing in current namespace".to_string(),
        );
        
        assert!(result.is_valid_command());
        assert!(!result.is_low_confidence(70));
        assert!(!result.needs_clarification());
    }

    #[test]
    fn test_low_confidence() {
        let result = TranslationResult::new(
            "kubectl logs".to_string(),
            40,
            "NEEDS_CLARIFICATION: Which pod?".to_string(),
        );
        
        assert!(result.is_low_confidence(70));
        assert!(result.needs_clarification());
    }

    #[test]
    fn test_invalid_command() {
        let result = TranslationResult::new(
            "docker ps".to_string(),
            50,
            "Wrong tool".to_string(),
        );
        
        assert!(!result.is_valid_command());
    }
}

