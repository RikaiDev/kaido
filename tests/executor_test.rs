#[cfg(test)]
mod tests {
    use kaido::shell::CommandExecutor;

    #[test]
    fn test_execute_simple_command() {
        let executor = CommandExecutor::new();
        let result = executor.execute("echo", &["hello"]);
        assert!(result.is_ok());
        assert_eq!(String::from_utf8_lossy(&result.unwrap().stdout), "hello\n");
    }
}
