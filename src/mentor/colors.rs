// Color definitions for mentor display
//
// Provides consistent terminal coloring for the mentor system.
// Respects NO_COLOR environment variable for accessibility.

/// ANSI escape codes for terminal colors
pub struct MentorColors {
    /// Whether colors are enabled
    enabled: bool,
}

impl MentorColors {
    /// Create new color provider, respecting NO_COLOR env var
    pub fn new() -> Self {
        let enabled = std::env::var("NO_COLOR").is_err();
        Self { enabled }
    }

    /// Create with colors explicitly enabled or disabled
    pub fn with_enabled(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Check if colors are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    // Border and structure colors

    /// Dim cyan for box borders
    pub fn border(&self) -> &'static str {
        if self.enabled { "\x1b[36m" } else { "" }
    }

    /// Bold cyan for title
    pub fn title(&self) -> &'static str {
        if self.enabled { "\x1b[1;36m" } else { "" }
    }

    // Content colors

    /// Bold yellow for key message (the main error)
    pub fn key_message(&self) -> &'static str {
        if self.enabled { "\x1b[1;33m" } else { "" }
    }

    /// White for explanation text
    pub fn explanation(&self) -> &'static str {
        if self.enabled { "\x1b[0m" } else { "" }
    }

    /// Dim blue for source location
    pub fn location(&self) -> &'static str {
        if self.enabled { "\x1b[34m" } else { "" }
    }

    /// Green for search suggestions
    pub fn search(&self) -> &'static str {
        if self.enabled { "\x1b[32m" } else { "" }
    }

    /// Bold white for commands
    pub fn command(&self) -> &'static str {
        if self.enabled { "\x1b[1;37m" } else { "" }
    }

    /// Magenta for concepts/learning topics
    pub fn concept(&self) -> &'static str {
        if self.enabled { "\x1b[35m" } else { "" }
    }

    /// Dim for secondary/muted text
    pub fn dim(&self) -> &'static str {
        if self.enabled { "\x1b[2m" } else { "" }
    }

    /// Red for error type label
    pub fn error_type(&self) -> &'static str {
        if self.enabled { "\x1b[1;31m" } else { "" }
    }

    /// Reset all formatting
    pub fn reset(&self) -> &'static str {
        if self.enabled { "\x1b[0m" } else { "" }
    }

    /// Underline for emphasis
    pub fn underline(&self) -> &'static str {
        if self.enabled { "\x1b[4m" } else { "" }
    }
}

impl Default for MentorColors {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colors_enabled() {
        let colors = MentorColors::with_enabled(true);
        assert!(colors.is_enabled());
        assert!(!colors.border().is_empty());
        assert!(!colors.reset().is_empty());
    }

    #[test]
    fn test_colors_disabled() {
        let colors = MentorColors::with_enabled(false);
        assert!(!colors.is_enabled());
        assert!(colors.border().is_empty());
        assert!(colors.reset().is_empty());
    }

    #[test]
    fn test_all_colors_have_reset() {
        let colors = MentorColors::with_enabled(true);
        // All color codes should be non-empty when enabled
        assert!(!colors.border().is_empty());
        assert!(!colors.title().is_empty());
        assert!(!colors.key_message().is_empty());
        assert!(!colors.location().is_empty());
        assert!(!colors.search().is_empty());
        assert!(!colors.command().is_empty());
        assert!(!colors.concept().is_empty());
        assert!(!colors.dim().is_empty());
        assert!(!colors.error_type().is_empty());
    }
}
