// Shell builtins for Kaido shell
//
// Implements shell builtin commands that must be handled by Kaido
// itself rather than delegated to child processes.

use std::collections::HashMap;
use std::path::PathBuf;

/// Environment variable storage for the shell
#[derive(Debug, Clone, Default)]
pub struct ShellEnvironment {
    /// Custom environment variables (set via export)
    variables: HashMap<String, String>,
    /// Command aliases (name -> expansion)
    aliases: HashMap<String, String>,
    /// Previous working directory (for cd -)
    previous_dir: Option<PathBuf>,
}

impl ShellEnvironment {
    /// Create a new shell environment
    pub fn new() -> Self {
        Self::default()
    }

    // === Environment Variables ===

    /// Set an environment variable
    pub fn set_var(&mut self, name: &str, value: &str) {
        self.variables.insert(name.to_string(), value.to_string());
        // Also set in the actual process environment
        std::env::set_var(name, value);
    }

    /// Get an environment variable (checks shell vars first, then system)
    pub fn get_var(&self, name: &str) -> Option<String> {
        self.variables
            .get(name)
            .cloned()
            .or_else(|| std::env::var(name).ok())
    }

    /// Remove an environment variable
    pub fn unset_var(&mut self, name: &str) {
        self.variables.remove(name);
        std::env::remove_var(name);
    }

    /// List all custom environment variables
    pub fn list_vars(&self) -> impl Iterator<Item = (&String, &String)> {
        self.variables.iter()
    }

    // === Aliases ===

    /// Set an alias
    pub fn set_alias(&mut self, name: &str, expansion: &str) {
        self.aliases.insert(name.to_string(), expansion.to_string());
    }

    /// Get an alias expansion
    pub fn get_alias(&self, name: &str) -> Option<&String> {
        self.aliases.get(name)
    }

    /// Remove an alias
    pub fn unset_alias(&mut self, name: &str) -> bool {
        self.aliases.remove(name).is_some()
    }

    /// List all aliases
    pub fn list_aliases(&self) -> impl Iterator<Item = (&String, &String)> {
        self.aliases.iter()
    }

    // === Directory Tracking ===

    /// Get the previous directory
    pub fn previous_dir(&self) -> Option<&PathBuf> {
        self.previous_dir.as_ref()
    }

    /// Set the previous directory (call before changing to new directory)
    pub fn set_previous_dir(&mut self, dir: PathBuf) {
        self.previous_dir = Some(dir);
    }

    /// Expand aliases in a command line
    /// Returns the expanded command or None if no alias matched
    pub fn expand_aliases(&self, line: &str) -> Option<String> {
        let mut parts = line.split_whitespace();
        let first = parts.next()?;

        if let Some(expansion) = self.aliases.get(first) {
            let rest: Vec<&str> = parts.collect();
            if rest.is_empty() {
                Some(expansion.clone())
            } else {
                Some(format!("{} {}", expansion, rest.join(" ")))
            }
        } else {
            None
        }
    }
}

/// Builtin command types
#[derive(Debug, Clone)]
pub enum Builtin {
    /// Change directory: cd [dir]
    Cd(String),
    /// Export environment variable: export VAR=value
    Export(String, String),
    /// List exports: export (no args)
    ExportList,
    /// Remove environment variable: unset VAR
    Unset(String),
    /// Set alias: alias name=value
    Alias(String, String),
    /// List aliases: alias (no args)
    AliasList,
    /// Remove alias: unalias name
    Unalias(String),
    /// Source a file: source file
    Source(PathBuf),
    /// Exit shell: exit [code]
    Exit(i32),
    /// Display help
    Help,
    /// Display history
    History,
    /// Clear screen
    Clear,
}

/// Parse a command line into a builtin if it matches
pub fn parse_builtin(line: &str) -> Option<Builtin> {
    let line = line.trim();

    // Exit
    if line == "exit" || line == "quit" {
        return Some(Builtin::Exit(0));
    }
    if let Some(code) = line.strip_prefix("exit ") {
        let code = code.trim().parse().unwrap_or(0);
        return Some(Builtin::Exit(code));
    }

    // Help
    if line == "help" {
        return Some(Builtin::Help);
    }

    // History
    if line == "history" {
        return Some(Builtin::History);
    }

    // Clear
    if line == "clear" {
        return Some(Builtin::Clear);
    }

    // Cd
    if line == "cd" {
        return Some(Builtin::Cd("~".to_string()));
    }
    if let Some(path) = line.strip_prefix("cd ") {
        return Some(Builtin::Cd(path.trim().to_string()));
    }

    // Export
    if line == "export" {
        return Some(Builtin::ExportList);
    }
    if let Some(rest) = line.strip_prefix("export ") {
        let rest = rest.trim();
        if let Some((name, value)) = rest.split_once('=') {
            let name = name.trim();
            // Remove surrounding quotes from value
            let value = value.trim().trim_matches('"').trim_matches('\'');
            return Some(Builtin::Export(name.to_string(), value.to_string()));
        }
    }

    // Unset
    if let Some(name) = line.strip_prefix("unset ") {
        return Some(Builtin::Unset(name.trim().to_string()));
    }

    // Alias
    if line == "alias" {
        return Some(Builtin::AliasList);
    }
    if let Some(rest) = line.strip_prefix("alias ") {
        let rest = rest.trim();
        if let Some((name, value)) = rest.split_once('=') {
            let name = name.trim();
            // Remove surrounding quotes from value
            let value = value.trim().trim_matches('"').trim_matches('\'');
            return Some(Builtin::Alias(name.to_string(), value.to_string()));
        }
    }

    // Unalias
    if let Some(name) = line.strip_prefix("unalias ") {
        return Some(Builtin::Unalias(name.trim().to_string()));
    }

    // Source
    if let Some(path) = line.strip_prefix("source ") {
        return Some(Builtin::Source(PathBuf::from(path.trim())));
    }
    if let Some(path) = line.strip_prefix(". ") {
        return Some(Builtin::Source(PathBuf::from(path.trim())));
    }

    None
}

/// Result of executing a builtin
#[derive(Debug)]
pub enum BuiltinResult {
    /// Success with optional message
    Ok(Option<String>),
    /// Error with message
    Error(String),
    /// Exit the shell with code
    Exit(i32),
    /// Source commands to execute
    Source(Vec<String>),
}

/// Execute a builtin command
pub fn execute_builtin(
    builtin: &Builtin,
    env: &mut ShellEnvironment,
) -> BuiltinResult {
    match builtin {
        Builtin::Cd(path) => execute_cd(path, env),
        Builtin::Export(name, value) => {
            env.set_var(name, value);
            BuiltinResult::Ok(None)
        }
        Builtin::ExportList => {
            let vars: Vec<String> = env
                .list_vars()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            if vars.is_empty() {
                BuiltinResult::Ok(None)
            } else {
                BuiltinResult::Ok(Some(vars.join("\n")))
            }
        }
        Builtin::Unset(name) => {
            env.unset_var(name);
            BuiltinResult::Ok(None)
        }
        Builtin::Alias(name, value) => {
            env.set_alias(name, value);
            BuiltinResult::Ok(None)
        }
        Builtin::AliasList => {
            let aliases: Vec<String> = env
                .list_aliases()
                .map(|(k, v)| format!("alias {}='{}'", k, v))
                .collect();
            if aliases.is_empty() {
                BuiltinResult::Ok(None)
            } else {
                BuiltinResult::Ok(Some(aliases.join("\n")))
            }
        }
        Builtin::Unalias(name) => {
            if env.unset_alias(name) {
                BuiltinResult::Ok(None)
            } else {
                BuiltinResult::Error(format!("unalias: {}: not found", name))
            }
        }
        Builtin::Source(path) => execute_source(path),
        Builtin::Exit(code) => BuiltinResult::Exit(*code),
        Builtin::Help | Builtin::History | Builtin::Clear => {
            // These are handled by the shell directly
            BuiltinResult::Ok(None)
        }
    }
}

/// Execute cd command
fn execute_cd(path: &str, env: &mut ShellEnvironment) -> BuiltinResult {
    let path = path.trim();

    // Save current directory before changing
    if let Ok(current) = std::env::current_dir() {
        env.set_previous_dir(current);
    }

    // Handle cd - (previous directory)
    if path == "-" {
        return match env.previous_dir() {
            Some(prev) => {
                let prev = prev.clone();
                match std::env::set_current_dir(&prev) {
                    Ok(()) => {
                        // Print the new directory (like bash does)
                        BuiltinResult::Ok(Some(prev.display().to_string()))
                    }
                    Err(e) => BuiltinResult::Error(format!("cd: {}: {}", prev.display(), e)),
                }
            }
            None => BuiltinResult::Error("cd: OLDPWD not set".to_string()),
        };
    }

    // Expand ~ to home directory
    let expanded = if path == "~" || path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            if path == "~" {
                home
            } else {
                home.join(&path[2..])
            }
        } else {
            PathBuf::from(path)
        }
    } else {
        PathBuf::from(path)
    };

    match std::env::set_current_dir(&expanded) {
        Ok(()) => BuiltinResult::Ok(None),
        Err(e) => BuiltinResult::Error(format!("cd: {}: {}", path, e)),
    }
}

/// Execute source command
fn execute_source(path: &PathBuf) -> BuiltinResult {
    // Expand ~ if present
    let expanded = if path.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            let path_str = path.to_string_lossy();
            if path_str == "~" {
                home
            } else {
                home.join(&path_str[2..])
            }
        } else {
            path.clone()
        }
    } else {
        path.clone()
    };

    match std::fs::read_to_string(&expanded) {
        Ok(content) => {
            let commands: Vec<String> = content
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .map(|l| l.to_string())
                .collect();
            BuiltinResult::Source(commands)
        }
        Err(e) => BuiltinResult::Error(format!("source: {}: {}", path.display(), e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_environment_new() {
        let env = ShellEnvironment::new();
        assert!(env.variables.is_empty());
        assert!(env.aliases.is_empty());
        assert!(env.previous_dir.is_none());
    }

    #[test]
    fn test_set_and_get_var() {
        let mut env = ShellEnvironment::new();
        env.set_var("TEST_VAR", "test_value");
        assert_eq!(env.get_var("TEST_VAR"), Some("test_value".to_string()));
    }

    #[test]
    fn test_unset_var() {
        let mut env = ShellEnvironment::new();
        env.set_var("TEST_VAR", "test_value");
        env.unset_var("TEST_VAR");
        // Note: get_var checks system env too, so we check our internal storage
        assert!(env.variables.get("TEST_VAR").is_none());
    }

    #[test]
    fn test_set_and_get_alias() {
        let mut env = ShellEnvironment::new();
        env.set_alias("ll", "ls -la");
        assert_eq!(env.get_alias("ll"), Some(&"ls -la".to_string()));
    }

    #[test]
    fn test_unset_alias() {
        let mut env = ShellEnvironment::new();
        env.set_alias("ll", "ls -la");
        assert!(env.unset_alias("ll"));
        assert!(env.get_alias("ll").is_none());
    }

    #[test]
    fn test_unset_alias_not_found() {
        let mut env = ShellEnvironment::new();
        assert!(!env.unset_alias("nonexistent"));
    }

    #[test]
    fn test_expand_aliases() {
        let mut env = ShellEnvironment::new();
        env.set_alias("k", "kubectl");

        assert_eq!(
            env.expand_aliases("k get pods"),
            Some("kubectl get pods".to_string())
        );
        assert_eq!(env.expand_aliases("k"), Some("kubectl".to_string()));
        assert_eq!(env.expand_aliases("kubectl get pods"), None);
    }

    #[test]
    fn test_previous_dir() {
        let mut env = ShellEnvironment::new();
        assert!(env.previous_dir().is_none());

        env.set_previous_dir(PathBuf::from("/tmp"));
        assert_eq!(env.previous_dir(), Some(&PathBuf::from("/tmp")));
    }

    #[test]
    fn test_parse_builtin_exit() {
        assert!(matches!(parse_builtin("exit"), Some(Builtin::Exit(0))));
        assert!(matches!(parse_builtin("quit"), Some(Builtin::Exit(0))));
        assert!(matches!(parse_builtin("exit 1"), Some(Builtin::Exit(1))));
    }

    #[test]
    fn test_parse_builtin_cd() {
        assert!(matches!(parse_builtin("cd"), Some(Builtin::Cd(s)) if s == "~"));
        assert!(matches!(parse_builtin("cd /tmp"), Some(Builtin::Cd(s)) if s == "/tmp"));
        assert!(matches!(parse_builtin("cd -"), Some(Builtin::Cd(s)) if s == "-"));
    }

    #[test]
    fn test_parse_builtin_export() {
        assert!(matches!(parse_builtin("export"), Some(Builtin::ExportList)));
        match parse_builtin("export FOO=bar") {
            Some(Builtin::Export(name, value)) => {
                assert_eq!(name, "FOO");
                assert_eq!(value, "bar");
            }
            _ => panic!("Expected Export"),
        }
        // Test with quotes
        match parse_builtin("export FOO=\"bar baz\"") {
            Some(Builtin::Export(name, value)) => {
                assert_eq!(name, "FOO");
                assert_eq!(value, "bar baz");
            }
            _ => panic!("Expected Export with quotes"),
        }
    }

    #[test]
    fn test_parse_builtin_alias() {
        assert!(matches!(parse_builtin("alias"), Some(Builtin::AliasList)));
        match parse_builtin("alias k=kubectl") {
            Some(Builtin::Alias(name, value)) => {
                assert_eq!(name, "k");
                assert_eq!(value, "kubectl");
            }
            _ => panic!("Expected Alias"),
        }
    }

    #[test]
    fn test_parse_builtin_source() {
        match parse_builtin("source ~/.bashrc") {
            Some(Builtin::Source(path)) => {
                assert_eq!(path, PathBuf::from("~/.bashrc"));
            }
            _ => panic!("Expected Source"),
        }
        // Test dot notation
        match parse_builtin(". ~/.bashrc") {
            Some(Builtin::Source(path)) => {
                assert_eq!(path, PathBuf::from("~/.bashrc"));
            }
            _ => panic!("Expected Source with dot"),
        }
    }

    #[test]
    fn test_parse_builtin_unset() {
        match parse_builtin("unset FOO") {
            Some(Builtin::Unset(name)) => {
                assert_eq!(name, "FOO");
            }
            _ => panic!("Expected Unset"),
        }
    }

    #[test]
    fn test_parse_builtin_unalias() {
        match parse_builtin("unalias k") {
            Some(Builtin::Unalias(name)) => {
                assert_eq!(name, "k");
            }
            _ => panic!("Expected Unalias"),
        }
    }

    #[test]
    fn test_parse_builtin_not_builtin() {
        assert!(parse_builtin("ls -la").is_none());
        assert!(parse_builtin("echo hello").is_none());
        assert!(parse_builtin("kubectl get pods").is_none());
    }

    #[test]
    fn test_execute_export() {
        let mut env = ShellEnvironment::new();
        let result = execute_builtin(&Builtin::Export("TEST".to_string(), "value".to_string()), &mut env);
        assert!(matches!(result, BuiltinResult::Ok(None)));
        assert_eq!(env.get_var("TEST"), Some("value".to_string()));
    }

    #[test]
    fn test_execute_alias() {
        let mut env = ShellEnvironment::new();
        let result = execute_builtin(&Builtin::Alias("k".to_string(), "kubectl".to_string()), &mut env);
        assert!(matches!(result, BuiltinResult::Ok(None)));
        assert_eq!(env.get_alias("k"), Some(&"kubectl".to_string()));
    }

    #[test]
    fn test_execute_exit() {
        let mut env = ShellEnvironment::new();
        let result = execute_builtin(&Builtin::Exit(42), &mut env);
        assert!(matches!(result, BuiltinResult::Exit(42)));
    }
}
