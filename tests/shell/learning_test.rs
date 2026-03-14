#[cfg(test)]
mod tests {
    use kaido::shell::learning::LearningTracker;

    #[test]
    fn test_skill_tracking() {
        let mut tracker = LearningTracker::new();

        // Record file operations
        tracker.record_command("ls -la");
        tracker.record_command("cd /tmp");
        tracker.record_command("cp file1 file2");

        // Check progress
        let progress = tracker.get_progress();
        assert!(progress.contains("File Operations"));
    }
}
