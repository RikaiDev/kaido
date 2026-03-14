#[cfg(test)]
mod tests {
    use kaido::shell::ai::{AIProcessor, Translation};

    #[test]
    fn test_detect_natural_language() {
        let ai = AIProcessor::new();

        assert!(ai.is_natural_language("show me files"));
        assert!(ai.is_natural_language("start nginx service"));
        assert!(!ai.is_natural_language("ls -la"));
        assert!(!ai.is_natural_language("grep foo bar"));
    }

    #[test]
    fn test_translation_display() {
        let translation = Translation {
            original: "start nginx".to_string(),
            intent: "Start nginx service".to_string(),
            command: "sudo systemctl start nginx".to_string(),
            explanation: "This will start nginx, requires sudo".to_string(),
        };

        let display = translation.to_display_string();
        assert!(display.contains("Intent:"));
        assert!(display.contains("Translate:"));
    }

    #[test]
    fn test_error_explanation() {
        let ai = AIProcessor::new();
        let error_output = "nginx: [emerg] bind() to 0.0.0.0:80 failed";

        let explanation = ai.explain_error(error_output);
        assert!(explanation.contains("Port 80"));
        assert!(explanation.contains("already in use"));
    }
}
