// Prompt builder for Kaido shell
//
// Builds a prompt that shows:
// - kaido branding
// - current directory (shortened)
// - git branch (if in a git repo)

use std::env;
use std::path::PathBuf;

/// ANSI color codes for prompt
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";

    pub const CYAN: &str = "\x1b[36m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
}

/// Prompt builder for the Kaido shell
pub struct PromptBuilder {
    /// Whether to show colors
    use_colors: bool,
    /// Whether to show git branch
    show_git_branch: bool,
    /// Custom prompt prefix (default: "kaido")
    prefix: String,
}

impl PromptBuilder {
    /// Create a new prompt builder with defaults
    pub fn new() -> Self {
        Self {
            use_colors: true,
            show_git_branch: true,
            prefix: "kaido".to_string(),
        }
    }

    /// Disable colors
    pub fn no_colors(mut self) -> Self {
        self.use_colors = false;
        self
    }

    /// Disable git branch display
    pub fn no_git_branch(mut self) -> Self {
        self.show_git_branch = false;
        self
    }

    /// Set custom prefix
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Build the prompt string
    pub fn build(&self) -> String {
        let cwd = self.get_shortened_cwd();
        let git_branch = if self.show_git_branch {
            self.get_git_branch()
        } else {
            None
        };

        if self.use_colors {
            self.build_colored_prompt(&cwd, git_branch.as_deref())
        } else {
            self.build_plain_prompt(&cwd, git_branch.as_deref())
        }
    }

    /// Build colored prompt
    fn build_colored_prompt(&self, cwd: &str, git_branch: Option<&str>) -> String {
        let mut prompt = String::new();

        // Prefix (cyan, bold)
        prompt.push_str(colors::BOLD);
        prompt.push_str(colors::CYAN);
        prompt.push_str(&self.prefix);
        prompt.push_str(colors::RESET);

        // Space
        prompt.push(' ');

        // Current directory (blue)
        prompt.push_str(colors::BLUE);
        prompt.push_str(cwd);
        prompt.push_str(colors::RESET);

        // Git branch (green, in parentheses)
        if let Some(branch) = git_branch {
            prompt.push(' ');
            prompt.push_str(colors::DIM);
            prompt.push('(');
            prompt.push_str(colors::GREEN);
            prompt.push_str(branch);
            prompt.push_str(colors::RESET);
            prompt.push_str(colors::DIM);
            prompt.push(')');
            prompt.push_str(colors::RESET);
        }

        // Prompt character
        prompt.push(' ');
        prompt.push_str(colors::YELLOW);
        prompt.push_str("$ ");
        prompt.push_str(colors::RESET);

        prompt
    }

    /// Build plain prompt (no colors)
    fn build_plain_prompt(&self, cwd: &str, git_branch: Option<&str>) -> String {
        let mut prompt = String::new();

        prompt.push_str(&self.prefix);
        prompt.push(' ');
        prompt.push_str(cwd);

        if let Some(branch) = git_branch {
            prompt.push_str(" (");
            prompt.push_str(branch);
            prompt.push(')');
        }

        prompt.push_str(" $ ");

        prompt
    }

    /// Get current working directory, shortened
    fn get_shortened_cwd(&self) -> String {
        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // Try to replace home directory with ~
        if let Some(home) = dirs::home_dir() {
            if let Ok(relative) = cwd.strip_prefix(&home) {
                return format!("~/{}", relative.display());
            }
        }

        cwd.display().to_string()
    }

    /// Get current git branch if in a git repository
    fn get_git_branch(&self) -> Option<String> {
        // Try to read .git/HEAD
        let cwd = env::current_dir().ok()?;

        // Walk up to find .git directory
        let mut current = cwd.as_path();
        loop {
            let git_head = current.join(".git/HEAD");
            if git_head.exists() {
                return self.parse_git_head(&git_head);
            }

            // Move to parent
            current = current.parent()?;
        }
    }

    /// Parse .git/HEAD to get branch name
    fn parse_git_head(&self, head_path: &std::path::Path) -> Option<String> {
        let content = std::fs::read_to_string(head_path).ok()?;
        let content = content.trim();

        // Format: "ref: refs/heads/branch-name"
        if let Some(branch) = content.strip_prefix("ref: refs/heads/") {
            Some(branch.to_string())
        } else if content.len() == 40 {
            // Detached HEAD (SHA)
            Some(format!("{}...", &content[..7]))
        } else {
            None
        }
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
    fn test_prompt_builder_default() {
        let builder = PromptBuilder::new();
        let prompt = builder.build();

        // Should contain kaido prefix
        assert!(prompt.contains("kaido"));
        // Should end with $
        assert!(prompt.contains("$"));
    }

    #[test]
    fn test_prompt_builder_no_colors() {
        let builder = PromptBuilder::new().no_colors();
        let prompt = builder.build();

        // Should not contain ANSI codes
        assert!(!prompt.contains("\x1b["));
        assert!(prompt.contains("kaido"));
    }

    #[test]
    fn test_prompt_builder_custom_prefix() {
        let builder = PromptBuilder::new().no_colors().with_prefix("myshell");
        let prompt = builder.build();

        assert!(prompt.starts_with("myshell "));
    }

    #[test]
    fn test_shortened_cwd() {
        let builder = PromptBuilder::new();
        let cwd = builder.get_shortened_cwd();

        // Should return something (not empty)
        assert!(!cwd.is_empty());
    }
}
