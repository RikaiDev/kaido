// Mentor Guidance structures
//
// Defines the structured guidance format that both pattern matching
// and LLM fallback can produce.

use serde::{Deserialize, Serialize};

/// A single actionable next step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextStep {
    /// Description of what to do
    pub description: String,
    /// Optional command to run
    pub command: Option<String>,
}

impl NextStep {
    /// Create a new next step with command
    pub fn with_command(description: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            command: Some(command.into()),
        }
    }

    /// Create a new next step without command
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            command: None,
        }
    }
}

/// Source of the guidance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GuidanceSource {
    /// Generated from pattern matching (instant)
    Pattern,
    /// Generated from LLM (slower but comprehensive)
    LLM,
    /// Retrieved from cache
    Cached,
    /// Generic fallback when all else fails
    Fallback,
}

/// Complete mentor guidance for an error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentorGuidance {
    /// The most important part of the error message
    pub key_message: String,

    /// Plain language explanation of what the error means
    pub explanation: String,

    /// Keywords to search for more help
    pub search_keywords: Vec<String>,

    /// Actionable next steps
    pub next_steps: Vec<NextStep>,

    /// Related concepts to learn about
    pub related_concepts: Vec<String>,

    /// Where this guidance came from
    pub source: GuidanceSource,
}

impl MentorGuidance {
    /// Create guidance from pattern matching
    pub fn from_pattern(key_message: impl Into<String>, explanation: impl Into<String>) -> Self {
        Self {
            key_message: key_message.into(),
            explanation: explanation.into(),
            search_keywords: Vec::new(),
            next_steps: Vec::new(),
            related_concepts: Vec::new(),
            source: GuidanceSource::Pattern,
        }
    }

    /// Create fallback guidance when nothing else works
    pub fn fallback(key_message: impl Into<String>) -> Self {
        Self {
            key_message: key_message.into(),
            explanation: "An error occurred. Check the full output for details.".to_string(),
            search_keywords: Vec::new(),
            next_steps: Vec::new(),
            related_concepts: Vec::new(),
            source: GuidanceSource::Fallback,
        }
    }

    /// Add search keywords
    pub fn with_search(mut self, keywords: Vec<String>) -> Self {
        self.search_keywords = keywords;
        self
    }

    /// Add next steps
    pub fn with_steps(mut self, steps: Vec<NextStep>) -> Self {
        self.next_steps = steps;
        self
    }

    /// Add related concepts
    pub fn with_concepts(mut self, concepts: Vec<String>) -> Self {
        self.related_concepts = concepts;
        self
    }

    /// Mark as from LLM
    pub fn from_llm(mut self) -> Self {
        self.source = GuidanceSource::LLM;
        self
    }

    /// Mark as from cache
    pub fn from_cache(mut self) -> Self {
        self.source = GuidanceSource::Cached;
        self
    }
}

impl Default for MentorGuidance {
    fn default() -> Self {
        Self::fallback("Unknown error")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_step_with_command() {
        let step = NextStep::with_command("Fix the typo", "vim file.txt");
        assert_eq!(step.description, "Fix the typo");
        assert_eq!(step.command, Some("vim file.txt".to_string()));
    }

    #[test]
    fn test_next_step_without_command() {
        let step = NextStep::new("Check the documentation");
        assert!(step.command.is_none());
    }

    #[test]
    fn test_guidance_from_pattern() {
        let guidance =
            MentorGuidance::from_pattern("command not found", "The command is not installed");
        assert_eq!(guidance.source, GuidanceSource::Pattern);
    }

    #[test]
    fn test_guidance_builder() {
        let guidance = MentorGuidance::from_pattern("error", "explanation")
            .with_search(vec!["keyword".to_string()])
            .with_steps(vec![NextStep::new("do something")])
            .with_concepts(vec!["concept".to_string()]);

        assert_eq!(guidance.search_keywords.len(), 1);
        assert_eq!(guidance.next_steps.len(), 1);
        assert_eq!(guidance.related_concepts.len(), 1);
    }

    #[test]
    fn test_guidance_serialization() {
        let guidance = MentorGuidance::from_pattern("test", "test explanation");
        let json = serde_json::to_string(&guidance).unwrap();
        let parsed: MentorGuidance = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.key_message, "test");
    }
}
