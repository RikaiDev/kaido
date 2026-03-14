#[cfg(test)]
mod tests {
    use kaido::shell::ai::AIProcessor;

    #[test]
    fn test_detect_natural_language() {
        let ai = AIProcessor::new();

        assert!(ai.is_natural_language("show me files"));
        assert!(ai.is_natural_language("start nginx service"));
        assert!(!ai.is_natural_language("ls -la"));
        assert!(!ai.is_natural_language("grep foo bar"));
    }
}
