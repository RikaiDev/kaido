use std::process::Command;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_installation_workflow() {
    // Test 1: kaido --init-config creates configuration file
    let output = Command::new("cargo")
        .args(&["run", "--", "--init-config"])
        .output()
        .expect("Failed to execute kaido --init-config");

    assert!(output.status.success(), "kaido --init-config should succeed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Configuration file created"), "Should indicate config creation");
    assert!(stdout.contains("config.toml"), "Should mention config.toml file");

    // Test 2: kaido --validate-config validates the created configuration
    let output = Command::new("cargo")
        .args(&["run", "--", "--validate-config"])
        .output()
        .expect("Failed to execute kaido --validate-config");

    assert!(output.status.success(), "kaido --validate-config should succeed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Configuration is valid"), "Should indicate valid configuration");
    assert!(stdout.contains("Model:"), "Should show model information");
    assert!(stdout.contains("Safety level:"), "Should show safety level");
}

#[test]
fn test_configuration_file_creation() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("test_config.toml");

    // Test configuration creation with custom path
    let output = Command::new("cargo")
        .args(&["run", "--", "--init-config"])
        .env("XDG_CONFIG_HOME", temp_dir.path())
        .output()
        .expect("Failed to execute kaido --init-config");

    assert!(output.status.success(), "Configuration creation should succeed");

    // Verify configuration file exists
    let config_file = temp_dir.path().join("kaido").join("config.toml");
    assert!(config_file.exists(), "Configuration file should be created");
}

#[test]
fn test_configuration_validation_with_custom_path() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("test_config.toml");

    // First create a configuration file
    let output = Command::new("cargo")
        .args(&["run", "--", "--init-config"])
        .env("XDG_CONFIG_HOME", temp_dir.path())
        .output()
        .expect("Failed to execute kaido --init-config");

    assert!(output.status.success(), "Configuration creation should succeed");

    // Then validate it
    let output = Command::new("cargo")
        .args(&["run", "--", "--validate-config"])
        .env("XDG_CONFIG_HOME", temp_dir.path())
        .output()
        .expect("Failed to execute kaido --validate-config");

    assert!(output.status.success(), "Configuration validation should succeed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Configuration is valid"), "Should indicate valid configuration");
}

#[test]
fn test_help_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to execute kaido --help");

    assert!(output.status.success(), "Help command should succeed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("AI-powered shell"), "Should show description");
    assert!(stdout.contains("--init-config"), "Should show init-config option");
    assert!(stdout.contains("--validate-config"), "Should show validate-config option");
}

#[test]
fn test_version_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--version"])
        .output()
        .expect("Failed to execute kaido --version");

    assert!(output.status.success(), "Version command should succeed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("kaido"), "Should show program name");
}

#[test]
fn test_configuration_file_format() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create configuration file
    let output = Command::new("cargo")
        .args(&["run", "--", "--init-config"])
        .env("XDG_CONFIG_HOME", temp_dir.path())
        .output()
        .expect("Failed to execute kaido --init-config");

    assert!(output.status.success(), "Configuration creation should succeed");

    // Read and validate the configuration file format
    let config_file = temp_dir.path().join("kaido").join("config.toml");
    let config_content = std::fs::read_to_string(&config_file)
        .expect("Failed to read configuration file");

    // Check for required sections
    assert!(config_content.contains("[model]"), "Should contain model section");
    assert!(config_content.contains("[safety]"), "Should contain safety section");
    assert!(config_content.contains("[shell]"), "Should contain shell section");
    assert!(config_content.contains("[logging]"), "Should contain logging section");
    assert!(config_content.contains("[ai]"), "Should contain ai section");
    assert!(config_content.contains("[privacy]"), "Should contain privacy section");

    // Check for required model fields
    assert!(config_content.contains("name ="), "Should contain model name");
    assert!(config_content.contains("model_type ="), "Should contain model type");
    assert!(config_content.contains("max_tokens ="), "Should contain max tokens");
    assert!(config_content.contains("temperature ="), "Should contain temperature");
}
