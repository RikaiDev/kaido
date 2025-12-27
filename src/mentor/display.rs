// Mentor display formatting
//
// Renders mentor guidance in a clear, educational format with
// proper terminal styling and adaptive width.

use super::colors::MentorColors;
use super::guidance::MentorGuidance;
use super::types::ErrorInfo;

/// Verbosity level for mentor display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Verbosity {
    /// Full educational explanation with all details
    Verbose,
    /// Standard display with key points
    #[default]
    Normal,
    /// One-liner for experts
    Compact,
}

/// Configuration for mentor display
#[derive(Debug, Clone)]
pub struct DisplayConfig {
    /// Verbosity level
    pub verbosity: Verbosity,
    /// Terminal width (0 = auto-detect)
    pub terminal_width: u16,
    /// Whether colors are enabled
    pub colors_enabled: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            verbosity: Verbosity::Normal,
            terminal_width: 0, // Auto-detect
            colors_enabled: std::env::var("NO_COLOR").is_err(),
        }
    }
}

/// Mentor display renderer
pub struct MentorDisplay {
    config: DisplayConfig,
    colors: MentorColors,
}

impl MentorDisplay {
    /// Create new display with default config
    pub fn new() -> Self {
        Self::with_config(DisplayConfig::default())
    }

    /// Create display with custom config
    pub fn with_config(config: DisplayConfig) -> Self {
        let colors = MentorColors::with_enabled(config.colors_enabled);
        Self { config, colors }
    }

    /// Set verbosity level
    pub fn with_verbosity(mut self, verbosity: Verbosity) -> Self {
        self.config.verbosity = verbosity;
        self
    }

    /// Render error info as formatted string
    pub fn render(&self, error: &ErrorInfo) -> String {
        match self.config.verbosity {
            Verbosity::Verbose => self.render_verbose(error),
            Verbosity::Normal => self.render_normal(error),
            Verbosity::Compact => self.render_compact(error),
        }
    }

    /// Render MentorGuidance as formatted string
    pub fn render_guidance(&self, guidance: &MentorGuidance) -> String {
        match self.config.verbosity {
            Verbosity::Verbose => self.render_guidance_verbose(guidance),
            Verbosity::Normal => self.render_guidance_normal(guidance),
            Verbosity::Compact => self.render_guidance_compact(guidance),
        }
    }

    /// Render compact guidance
    fn render_guidance_compact(&self, guidance: &MentorGuidance) -> String {
        let c = &self.colors;
        let width = self.box_width().min(60);
        let inner_width = width - 4;

        let key_msg = Self::truncate(&guidance.key_message, inner_width - 4);

        let mut output = String::new();
        output.push_str(&format!(
            "{}┌─ MENTOR {}┐{}\n",
            c.border(),
            "─".repeat(width - 12),
            c.reset()
        ));
        output.push_str(&format!(
            "{}│{} {}{} {}│{}\n",
            c.border(),
            c.key_message(),
            key_msg,
            c.reset(),
            " ".repeat(inner_width.saturating_sub(key_msg.len())),
            c.reset()
        ));
        output.push_str(&format!(
            "{}└{}┘{}",
            c.border(),
            "─".repeat(width - 2),
            c.reset()
        ));

        output
    }

    /// Render normal guidance
    fn render_guidance_normal(&self, guidance: &MentorGuidance) -> String {
        let c = &self.colors;
        let width = self.box_width();
        let inner_width = width - 4;

        let mut output = String::new();

        // Top border
        output.push_str(&format!(
            "\n{}┌─ {}MENTOR{} {}┐{}\n",
            c.border(),
            c.title(),
            c.border(),
            "─".repeat(width - 12),
            c.reset()
        ));

        output.push_str(&self.render_empty_line(width));

        // Key message
        let key_display = Self::truncate(&guidance.key_message, inner_width - 10);
        output.push_str(&self.render_line(
            width,
            &format!(
                "  {}Key:{} {}{}{}",
                c.error_type(),
                c.reset(),
                c.key_message(),
                key_display,
                c.reset()
            ),
        ));

        output.push_str(&self.render_empty_line(width));

        // Explanation (wrapped)
        for line in Self::wrap_text(&guidance.explanation, inner_width - 4) {
            output.push_str(&self.render_line(width, &format!("  {}", line)));
        }

        output.push_str(&self.render_empty_line(width));

        // First next step if available
        if let Some(step) = guidance.next_steps.first() {
            let step_text = if let Some(ref cmd) = step.command {
                format!("Try: {}{}{}", c.command(), cmd, c.reset())
            } else {
                format!("Try: {}", step.description)
            };
            output.push_str(&self.render_line(
                width,
                &format!("  {}{}{}", c.search(), step_text, c.reset()),
            ));
            output.push_str(&self.render_empty_line(width));
        }

        // Bottom border
        output.push_str(&format!(
            "{}└{}┘{}\n",
            c.border(),
            "─".repeat(width - 2),
            c.reset()
        ));

        output
    }

    /// Render verbose guidance
    fn render_guidance_verbose(&self, guidance: &MentorGuidance) -> String {
        let c = &self.colors;
        let width = self.box_width();
        let inner_width = width - 4;

        let mut output = String::new();

        // Top border
        output.push_str(&format!(
            "\n{}┌─ {}MENTOR{} {}┐{}\n",
            c.border(),
            c.title(),
            c.border(),
            "─".repeat(width - 12),
            c.reset()
        ));

        output.push_str(&self.render_empty_line(width));

        // Key message with underline
        let key_display = Self::truncate(&guidance.key_message, inner_width - 10);
        output.push_str(&self.render_line(
            width,
            &format!(
                "  {}Key:{} \"{}{}{}\"",
                c.error_type(),
                c.reset(),
                c.key_message(),
                key_display,
                c.reset()
            ),
        ));

        let underline_len = key_display.len().min(inner_width - 12);
        output.push_str(&self.render_line(
            width,
            &format!("       {}{}{}", c.dim(), "~".repeat(underline_len), c.reset()),
        ));

        output.push_str(&self.render_empty_line(width));

        // Explanation
        output.push_str(&self.render_line(
            width,
            &format!("  {}This means:{}", c.dim(), c.reset()),
        ));
        for line in Self::wrap_text(&guidance.explanation, inner_width - 6) {
            output.push_str(&self.render_line(width, &format!("    {}", line)));
        }

        output.push_str(&self.render_empty_line(width));

        // Search keywords
        if !guidance.search_keywords.is_empty() {
            let keywords = guidance.search_keywords.join(", ");
            output.push_str(&self.render_line(
                width,
                &format!("  {}Search:{} {}", c.search(), c.reset(), keywords),
            ));
            output.push_str(&self.render_empty_line(width));
        }

        // Next steps
        if !guidance.next_steps.is_empty() {
            output.push_str(&self.render_line(
                width,
                &format!("  {}Next steps:{}", c.dim(), c.reset()),
            ));
            for (i, step) in guidance.next_steps.iter().take(4).enumerate() {
                let step_text = if let Some(ref cmd) = step.command {
                    format!("{}{}{}", c.command(), cmd, c.reset())
                } else {
                    step.description.clone()
                };
                let display = Self::truncate(&step_text, inner_width - 8);
                output.push_str(&self.render_line(
                    width,
                    &format!("    {}{}. {}{}", c.dim(), i + 1, c.reset(), display),
                ));
            }
            output.push_str(&self.render_empty_line(width));
        }

        // Related concepts
        if !guidance.related_concepts.is_empty() {
            let concepts = guidance.related_concepts.join(", ");
            output.push_str(&self.render_line(
                width,
                &format!(
                    "  {}Learn more:{} {}{}{}",
                    c.concept(),
                    c.reset(),
                    c.concept(),
                    concepts,
                    c.reset()
                ),
            ));
            output.push_str(&self.render_empty_line(width));
        }

        // Bottom border
        output.push_str(&format!(
            "{}└{}┘{}\n",
            c.border(),
            "─".repeat(width - 2),
            c.reset()
        ));

        output
    }

    /// Get the box width based on terminal width
    fn box_width(&self) -> usize {
        let term_width = if self.config.terminal_width > 0 {
            self.config.terminal_width as usize
        } else {
            // Try to detect terminal width
            terminal_size::terminal_size()
                .map(|(w, _)| w.0 as usize)
                .unwrap_or(80)
        };

        if term_width > 100 {
            80 // Cap at 80 for readability
        } else if term_width > 60 {
            term_width - 4 // Leave margin
        } else {
            term_width.max(40) // Minimum width
        }
    }

    /// Render compact one-liner
    fn render_compact(&self, error: &ErrorInfo) -> String {
        let c = &self.colors;
        let width = self.box_width().min(60);
        let inner_width = width - 4; // Account for borders and padding

        // Truncate key message if needed
        let key_msg = Self::truncate(&error.key_message, inner_width - 4);

        let mut output = String::new();

        // Top border
        output.push_str(&format!(
            "{}┌─ MENTOR {}┐{}\n",
            c.border(),
            "─".repeat(width - 12),
            c.reset()
        ));

        // Content line
        output.push_str(&format!(
            "{}│{} {}{} {}│{}\n",
            c.border(),
            c.key_message(),
            key_msg,
            c.reset(),
            " ".repeat(inner_width.saturating_sub(key_msg.len())),
            c.reset()
        ));

        // Bottom border
        output.push_str(&format!(
            "{}└{}┘{}",
            c.border(),
            "─".repeat(width - 2),
            c.reset()
        ));

        output
    }

    /// Render normal display with key points
    fn render_normal(&self, error: &ErrorInfo) -> String {
        let c = &self.colors;
        let width = self.box_width();
        let inner_width = width - 4;

        let mut output = String::new();

        // Top border with title
        output.push_str(&format!(
            "\n{}┌─ {}MENTOR{} {}┐{}\n",
            c.border(),
            c.title(),
            c.border(),
            "─".repeat(width - 12),
            c.reset()
        ));

        // Empty line
        output.push_str(&self.render_empty_line(width));

        // Error type
        output.push_str(&self.render_line(
            width,
            &format!(
                "  {}Type:{} {}",
                c.dim(),
                c.reset(),
                error.error_type.name()
            ),
        ));

        // Key message with highlight
        let key_display = Self::truncate(&error.key_message, inner_width - 10);
        output.push_str(&self.render_line(
            width,
            &format!(
                "  {}Key:{} {}{}{}",
                c.error_type(),
                c.reset(),
                c.key_message(),
                key_display,
                c.reset()
            ),
        ));

        // Source location if available
        if let Some(ref loc) = error.source_location {
            let loc_str = loc.to_string();
            let loc_display = Self::truncate(&loc_str, inner_width - 14);
            output.push_str(&self.render_line(
                width,
                &format!(
                    "  {}Location:{} {}{}{}",
                    c.location(),
                    c.reset(),
                    c.location(),
                    loc_display,
                    c.reset()
                ),
            ));
        }

        // Empty line
        output.push_str(&self.render_empty_line(width));

        // Suggested next step based on error type
        if let Some(suggestion) = self.get_quick_suggestion(error) {
            output.push_str(&self.render_line(
                width,
                &format!("  {}Try:{} {}", c.search(), c.reset(), suggestion),
            ));
            output.push_str(&self.render_empty_line(width));
        }

        // Bottom border
        output.push_str(&format!(
            "{}└{}┘{}\n",
            c.border(),
            "─".repeat(width - 2),
            c.reset()
        ));

        output
    }

    /// Render verbose display with full educational content
    fn render_verbose(&self, error: &ErrorInfo) -> String {
        let c = &self.colors;
        let width = self.box_width();
        let inner_width = width - 4;

        let mut output = String::new();

        // Top border with title
        output.push_str(&format!(
            "\n{}┌─ {}MENTOR{} {}┐{}\n",
            c.border(),
            c.title(),
            c.border(),
            "─".repeat(width - 12),
            c.reset()
        ));

        // Empty line
        output.push_str(&self.render_empty_line(width));

        // Key message with underline emphasis
        let key_display = Self::truncate(&error.key_message, inner_width - 10);
        output.push_str(&self.render_line(
            width,
            &format!(
                "  {}Key:{} \"{}{}{}\"",
                c.error_type(),
                c.reset(),
                c.key_message(),
                key_display,
                c.reset()
            ),
        ));

        // Underline for emphasis
        let underline_len = key_display.len().min(inner_width - 12);
        output.push_str(&self.render_line(
            width,
            &format!("       {}{}{}", c.dim(), "~".repeat(underline_len), c.reset()),
        ));

        // Empty line
        output.push_str(&self.render_empty_line(width));

        // Error explanation
        if let Some(explanation) = self.get_error_explanation(error) {
            output.push_str(&self.render_line(
                width,
                &format!("  {}This means:{}", c.dim(), c.reset()),
            ));
            for line in Self::wrap_text(&explanation, inner_width - 6) {
                output.push_str(&self.render_line(width, &format!("    {}", line)));
            }
            output.push_str(&self.render_empty_line(width));
        }

        // Source location if available
        if let Some(ref loc) = error.source_location {
            let loc_str = loc.to_string();
            output.push_str(&self.render_line(
                width,
                &format!(
                    "  {}Location:{} {}{}{}",
                    c.location(),
                    c.reset(),
                    c.location(),
                    loc_str,
                    c.reset()
                ),
            ));
            output.push_str(&self.render_empty_line(width));
        }

        // Search suggestion
        if let Some(search) = self.get_search_suggestion(error) {
            output.push_str(&self.render_line(
                width,
                &format!(
                    "  {}Search:{} {}",
                    c.search(),
                    c.reset(),
                    search
                ),
            ));
            output.push_str(&self.render_empty_line(width));
        }

        // Next steps
        let steps = self.get_next_steps(error);
        if !steps.is_empty() {
            output.push_str(&self.render_line(
                width,
                &format!("  {}Next steps:{}", c.dim(), c.reset()),
            ));
            for (i, step) in steps.iter().enumerate() {
                let step_display = Self::truncate(step, inner_width - 8);
                output.push_str(&self.render_line(
                    width,
                    &format!(
                        "    {}{}. {}{}{}",
                        c.dim(),
                        i + 1,
                        c.command(),
                        step_display,
                        c.reset()
                    ),
                ));
            }
            output.push_str(&self.render_empty_line(width));
        }

        // Learn more topics
        if let Some(concepts) = self.get_learning_concepts(error) {
            output.push_str(&self.render_line(
                width,
                &format!(
                    "  {}Learn more:{} {}{}{}",
                    c.concept(),
                    c.reset(),
                    c.concept(),
                    concepts,
                    c.reset()
                ),
            ));
            output.push_str(&self.render_empty_line(width));
        }

        // Bottom border
        output.push_str(&format!(
            "{}└{}┘{}\n",
            c.border(),
            "─".repeat(width - 2),
            c.reset()
        ));

        output
    }

    /// Render an empty line within the box
    fn render_empty_line(&self, width: usize) -> String {
        format!(
            "{}│{}│{}\n",
            self.colors.border(),
            " ".repeat(width - 2),
            self.colors.reset()
        )
    }

    /// Render a content line within the box
    fn render_line(&self, width: usize, content: &str) -> String {
        // Calculate visible length (without ANSI codes)
        let visible_len = Self::visible_length(content);
        let padding = (width - 2).saturating_sub(visible_len);

        format!(
            "{}│{}{}{}│{}\n",
            self.colors.border(),
            content,
            " ".repeat(padding),
            self.colors.reset(),
            self.colors.reset()
        )
    }

    /// Calculate visible length of string (excluding ANSI codes)
    fn visible_length(s: &str) -> usize {
        let mut len = 0;
        let mut in_escape = false;

        for c in s.chars() {
            if c == '\x1b' {
                in_escape = true;
            } else if in_escape {
                if c == 'm' {
                    in_escape = false;
                }
            } else {
                len += 1;
            }
        }

        len
    }

    /// Truncate string to max length with ellipsis
    fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else if max_len > 3 {
            format!("{}...", &s[..max_len - 3])
        } else {
            s[..max_len].to_string()
        }
    }

    /// Wrap text to fit within width
    fn wrap_text(text: &str, width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in text.split_whitespace() {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + 1 + word.len() <= width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    /// Get a quick suggestion for the error
    fn get_quick_suggestion(&self, error: &ErrorInfo) -> Option<String> {
        use super::types::ErrorType;

        match error.error_type {
            ErrorType::CommandNotFound => {
                let cmd = error.key_message
                    .split_whitespace()
                    .last()
                    .unwrap_or("command");
                Some(format!("which {} or brew install {}", cmd, cmd))
            }
            ErrorType::PermissionDenied => Some("sudo !!".to_string()),
            ErrorType::FileNotFound => Some("ls -la to check path".to_string()),
            ErrorType::ConnectionRefused => Some("Check if service is running".to_string()),
            ErrorType::PortInUse => Some("lsof -i :<port> to find process".to_string()),
            _ => None,
        }
    }

    /// Get detailed explanation for error type
    fn get_error_explanation(&self, error: &ErrorInfo) -> Option<String> {
        use super::types::ErrorType;

        match error.error_type {
            ErrorType::CommandNotFound => Some(
                "The shell cannot find this command. It's either not installed, \
                 or not in your PATH environment variable."
                    .to_string(),
            ),
            ErrorType::PermissionDenied => Some(
                "You don't have permission to perform this action. \
                 This usually means you need elevated privileges (sudo) \
                 or the file/directory permissions need to be changed."
                    .to_string(),
            ),
            ErrorType::FileNotFound => Some(
                "The specified file or directory doesn't exist. \
                 Check the path for typos or verify the file was created."
                    .to_string(),
            ),
            ErrorType::SyntaxError => Some(
                "There's a syntax error in the command or configuration file. \
                 Check for typos, missing quotes, or incorrect formatting."
                    .to_string(),
            ),
            ErrorType::ConnectionRefused => Some(
                "The connection was refused by the target. \
                 The service might not be running, or a firewall is blocking it."
                    .to_string(),
            ),
            ErrorType::ConfigurationError => Some(
                "There's an error in a configuration file. \
                 Check the file for typos or invalid directives."
                    .to_string(),
            ),
            ErrorType::PortInUse => Some(
                "Another process is already using this port. \
                 You'll need to stop that process or use a different port."
                    .to_string(),
            ),
            ErrorType::DependencyError => Some(
                "A required module or dependency is missing. \
                 You may need to install it or check your import paths."
                    .to_string(),
            ),
            _ => None,
        }
    }

    /// Get search suggestion for the error
    fn get_search_suggestion(&self, error: &ErrorInfo) -> Option<String> {
        use super::types::ErrorType;

        match error.error_type {
            ErrorType::CommandNotFound => {
                let cmd = error.key_message
                    .split_whitespace()
                    .last()
                    .unwrap_or("command");
                Some(format!("install {} macos/linux", cmd))
            }
            ErrorType::ConfigurationError => Some(format!(
                "{} configuration syntax",
                error.command.split_whitespace().next().unwrap_or("config")
            )),
            ErrorType::DockerError => Some("docker troubleshooting".to_string()),
            ErrorType::KubernetesError => Some("kubernetes debugging".to_string()),
            ErrorType::GitError => Some("git common errors".to_string()),
            _ => None,
        }
    }

    /// Get suggested next steps
    fn get_next_steps(&self, error: &ErrorInfo) -> Vec<String> {
        use super::types::ErrorType;

        match error.error_type {
            ErrorType::CommandNotFound => {
                let cmd = error.key_message
                    .split_whitespace()
                    .last()
                    .unwrap_or("command");
                vec![
                    format!("which {}", cmd),
                    format!("brew install {} (macOS)", cmd),
                    format!("apt install {} (Ubuntu)", cmd),
                ]
            }
            ErrorType::PermissionDenied => {
                vec![
                    "sudo !! (run last command as root)".to_string(),
                    "ls -la <file> (check permissions)".to_string(),
                    "chmod +x <file> (if executable)".to_string(),
                ]
            }
            ErrorType::FileNotFound => {
                vec![
                    "ls -la (list current directory)".to_string(),
                    "pwd (print working directory)".to_string(),
                    "find . -name '<filename>'".to_string(),
                ]
            }
            ErrorType::PortInUse => {
                vec![
                    "lsof -i :<port> (find process)".to_string(),
                    "kill <pid> (stop process)".to_string(),
                    "Use a different port".to_string(),
                ]
            }
            ErrorType::ConnectionRefused => {
                vec![
                    "Check if service is running".to_string(),
                    "Verify the host and port".to_string(),
                    "Check firewall settings".to_string(),
                ]
            }
            _ => {
                if let Some(ref loc) = error.source_location {
                    if let Some(line) = loc.line {
                        vec![format!(
                            "vim {} +{}",
                            loc.file.display(),
                            line
                        )]
                    } else {
                        vec![format!("vim {}", loc.file.display())]
                    }
                } else {
                    vec![]
                }
            }
        }
    }

    /// Get learning concepts related to the error
    fn get_learning_concepts(&self, error: &ErrorInfo) -> Option<String> {
        use super::types::ErrorType;

        match error.error_type {
            ErrorType::CommandNotFound => Some("PATH environment, package managers".to_string()),
            ErrorType::PermissionDenied => Some("Unix permissions, sudo, file ownership".to_string()),
            ErrorType::FileNotFound => Some("file paths, working directory, ls command".to_string()),
            ErrorType::ConnectionRefused => Some("networking, ports, services".to_string()),
            ErrorType::ConfigurationError => Some("configuration files, syntax checking".to_string()),
            ErrorType::DockerError => Some("Docker containers, images, volumes".to_string()),
            ErrorType::KubernetesError => Some("Kubernetes pods, deployments, services".to_string()),
            ErrorType::GitError => Some("Git workflow, branches, commits".to_string()),
            _ => None,
        }
    }
}

impl Default for MentorDisplay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mentor::types::{ErrorType, SourceLocation};

    fn create_test_error() -> ErrorInfo {
        ErrorInfo::new(
            ErrorType::CommandNotFound,
            127,
            "command not found: kubectl",
            "kubectl get pods",
        )
    }

    fn create_error_with_location() -> ErrorInfo {
        ErrorInfo::new(
            ErrorType::ConfigurationError,
            1,
            "unknown directive 'proxy_passs'",
            "nginx -t",
        )
        .with_location(SourceLocation::new("/etc/nginx/nginx.conf").with_line(42))
    }

    #[test]
    fn test_compact_render() {
        let display = MentorDisplay::new().with_verbosity(Verbosity::Compact);
        let error = create_test_error();
        let output = display.render(&error);

        assert!(output.contains("MENTOR"));
        assert!(output.contains("command not found"));
        // Should be a short output
        assert!(output.lines().count() <= 4);
    }

    #[test]
    fn test_normal_render() {
        let display = MentorDisplay::new().with_verbosity(Verbosity::Normal);
        let error = create_test_error();
        let output = display.render(&error);

        assert!(output.contains("MENTOR"));
        assert!(output.contains("Command Not Found"));
        assert!(output.contains("Key:"));
    }

    #[test]
    fn test_verbose_render() {
        let display = MentorDisplay::new().with_verbosity(Verbosity::Verbose);
        let error = create_test_error();
        let output = display.render(&error);

        assert!(output.contains("MENTOR"));
        assert!(output.contains("This means:"));
        assert!(output.contains("Next steps:"));
        assert!(output.contains("Learn more:"));
    }

    #[test]
    fn test_render_with_location() {
        let display = MentorDisplay::new().with_verbosity(Verbosity::Normal);
        let error = create_error_with_location();
        let output = display.render(&error);

        assert!(output.contains("Location:"));
        assert!(output.contains("nginx.conf"));
        assert!(output.contains("42"));
    }

    #[test]
    fn test_visible_length() {
        assert_eq!(MentorDisplay::visible_length("hello"), 5);
        assert_eq!(MentorDisplay::visible_length("\x1b[31mhello\x1b[0m"), 5);
        assert_eq!(MentorDisplay::visible_length("\x1b[1;33mtest\x1b[0m"), 4);
    }

    #[test]
    fn test_truncate() {
        assert_eq!(MentorDisplay::truncate("hello", 10), "hello");
        assert_eq!(MentorDisplay::truncate("hello world", 8), "hello...");
        assert_eq!(MentorDisplay::truncate("hi", 2), "hi");
    }

    #[test]
    fn test_wrap_text() {
        let lines = MentorDisplay::wrap_text("hello world this is a test", 12);
        assert_eq!(lines.len(), 3);
        assert!(lines[0].len() <= 12);
    }

    #[test]
    fn test_no_color() {
        let config = DisplayConfig {
            colors_enabled: false,
            ..Default::default()
        };
        let display = MentorDisplay::with_config(config);
        let error = create_test_error();
        let output = display.render(&error);

        // Should not contain ANSI escape codes
        assert!(!output.contains("\x1b["));
    }

    #[test]
    fn test_box_width_capped() {
        let config = DisplayConfig {
            terminal_width: 200,
            ..Default::default()
        };
        let display = MentorDisplay::with_config(config);
        assert_eq!(display.box_width(), 80); // Capped at 80
    }

    #[test]
    fn test_box_width_narrow() {
        let config = DisplayConfig {
            terminal_width: 70,
            ..Default::default()
        };
        let display = MentorDisplay::with_config(config);
        assert_eq!(display.box_width(), 66); // 70 - 4 margin
    }
}
