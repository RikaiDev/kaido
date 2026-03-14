use kaido::shell::ai::AIProcessor;
use kaido::shell::executor::CommandExecutor;
use kaido::shell::learning::LearningTracker;
use kaido::shell::parser::CommandParser;
use kaido::shell::skills::SkillsRegistry;

#[test]
fn test_parser_executor_integration() {
    let parser = CommandParser::new();
    let executor = CommandExecutor::new();

    let parsed = parser.parse("echo hello").unwrap();
    assert_eq!(parsed.command, "echo");
    assert_eq!(parsed.args, vec!["hello"]);

    let output = executor.execute("echo", &["hello"]).unwrap();
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello");
}

#[test]
fn test_ai_intent_detection() {
    let ai = AIProcessor::new();

    assert!(ai.is_natural_language("show me all files"));
    assert!(ai.is_natural_language("start the web server"));

    assert!(!ai.is_natural_language("ls -la"));
    assert!(!ai.is_natural_language("docker ps"));
}

#[test]
fn test_learning_tracker() {
    let mut tracker = LearningTracker::new();

    tracker.record_command("ls");
    tracker.record_command("cd /tmp");
    tracker.record_command("cp file1 file2");

    let progress = tracker.get_progress();
    assert!(progress.contains("File Operations"));
}

#[test]
fn test_skills_registry() {
    let registry = SkillsRegistry::load();

    if let Some(skill) = registry.match_error("502 Bad Gateway") {
        assert!(skill.teaches.iter().any(|t| t.contains("502")));
    }
}

#[test]
fn test_parser_pipe_handling() {
    let parser = CommandParser::new();

    let parsed = parser.parse("echo hello | grep h").unwrap();
    assert_eq!(parsed.commands.len(), 2);
}

#[test]
fn test_learning_tracker_categories() {
    let mut tracker = LearningTracker::new();

    tracker.record_command("docker ps");
    tracker.record_command("nginx start");
    tracker.record_command("ps aux");

    let progress = tracker.get_progress();
    assert!(progress.contains("Docker"));
    assert!(progress.contains("Nginx"));
    assert!(progress.contains("Process Management"));
}
