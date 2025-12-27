/// Build AI prompt for kubectl translation
/// 
/// This is a simplified version for kubectl-only MVP.
/// Full prompt engineering will be implemented in Phase 3.
pub struct PromptBuilder {
    system_prompt: String,
    user_input: String,
}

impl PromptBuilder {
    /// Create new prompt builder
    pub fn new() -> Self {
        Self {
            system_prompt: String::new(),
            user_input: String::new(),
        }
    }

    /// Set system prompt
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }

    /// Set user input
    pub fn with_user_input(mut self, input: impl Into<String>) -> Self {
        self.user_input = input.into();
        self
    }

    /// Build final prompt
    pub fn build(self) -> (String, String) {
        (self.system_prompt, self.user_input)
    }
}

impl Default for PromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_builder() {
        let (system, user) = PromptBuilder::new()
            .with_system_prompt("You are a kubectl expert")
            .with_user_input("show pods")
            .build();

        assert_eq!(system, "You are a kubectl expert");
        assert_eq!(user, "show pods");
    }
}
