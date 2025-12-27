use std::process::Command;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_api_key_configuration() {
    // Test 1: Configuration with invalid API key should fail validation
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("test_config.toml");
    
    // Create a configuration with invalid API key
    let config_content = r#"
[model]
name = "gpt-3.5-turbo"
path = "models/placeholder.gguf"
model_type = "Cloud"
max_tokens = 2048
temperature = 0.7
context_size = 4096

[cloud_api]
api_url = "https://api.openai.com/v1/chat/completions"
api_key = "sk-invalid-key"
model_name = "gpt-3.5-turbo"
timeout_seconds = 30

[safety]
require_confirmation_for = ["rm -rf", "sudo"]
auto_confirm_safe_commands = true
auto_confirm_dangerous = false
log_all_commands = true
max_plan_steps = 10
confirmation_timeout_seconds = 30

[shell]
default_prompt = "kaido> "
history_size = 1000
auto_complete = true
show_execution_time = true
timeout_seconds = 300

[logging]
level = "info"
file = "~/.local/share/kaido/logs/kaido.log"
max_file_size = "10MB"
max_files = 5

[ai]
explanation_style = "beginner"
safety_level = "medium"
auto_execute_plans = false
max_retries = 3
response_timeout = 30

[privacy]
offline_mode = false
log_ai_interactions = false
anonymize_commands = false
"#;
    
    fs::write(&config_path, config_content).expect("Failed to write config file");
    
    // Test configuration validation
    let output = Command::new("cargo")
        .args(&["run", "--", "--validate-config", "--config", config_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute kaido --validate-config");
    
    // Should succeed in loading and validating the configuration structure
    assert!(output.status.success(), "Configuration validation should succeed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Configuration is valid"), "Should indicate valid configuration");
    assert!(stdout.contains("Cloud API"), "Should show cloud API information");
}

#[test]
fn test_openai_configuration_template() {
    // Test 2: OpenAI configuration template should be valid
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("openai_config.toml");
    
    // Create OpenAI configuration template
    let config_content = r#"
[model]
name = "gpt-3.5-turbo"
path = "models/placeholder.gguf"
model_type = "Cloud"
max_tokens = 2048
temperature = 0.7
context_size = 4096

[cloud_api]
api_url = "https://api.openai.com/v1/chat/completions"
api_key = "sk-your-openai-api-key-here"
model_name = "gpt-3.5-turbo"
timeout_seconds = 30

[safety]
require_confirmation_for = ["rm -rf", "sudo"]
auto_confirm_safe_commands = true
auto_confirm_dangerous = false
log_all_commands = true
max_plan_steps = 10
confirmation_timeout_seconds = 30

[shell]
default_prompt = "kaido> "
history_size = 1000
auto_complete = true
show_execution_time = true
timeout_seconds = 300

[logging]
level = "info"
file = "~/.local/share/kaido/logs/kaido.log"
max_file_size = "10MB"
max_files = 5

[ai]
explanation_style = "beginner"
safety_level = "medium"
auto_execute_plans = false
max_retries = 3
response_timeout = 30

[privacy]
offline_mode = false
log_ai_interactions = false
anonymize_commands = false
"#;
    
    fs::write(&config_path, config_content).expect("Failed to write config file");
    
    // Test configuration validation
    let output = Command::new("cargo")
        .args(&["run", "--", "--validate-config", "--config", config_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute kaido --validate-config");
    
    assert!(output.status.success(), "OpenAI configuration should be valid");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Configuration is valid"), "Should indicate valid configuration");
    assert!(stdout.contains("gpt-3.5-turbo"), "Should show GPT model name");
}

#[test]
fn test_cloud_api_configuration_validation() {
    // Test 3: Cloud API configuration validation
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("cloud_config.toml");
    
    // Create configuration with missing cloud_api section but Cloud model type
    let config_content = r#"
[model]
name = "gpt-3.5-turbo"
path = "models/placeholder.gguf"
model_type = "Cloud"
max_tokens = 2048
temperature = 0.7
context_size = 4096

[safety]
require_confirmation_for = ["rm -rf", "sudo"]
auto_confirm_safe_commands = true
auto_confirm_dangerous = false
log_all_commands = true
max_plan_steps = 10
confirmation_timeout_seconds = 30

[shell]
default_prompt = "kaido> "
history_size = 1000
auto_complete = true
show_execution_time = true
timeout_seconds = 300

[logging]
level = "info"
file = "~/.local/share/kaido/logs/kaido.log"
max_file_size = "10MB"
max_files = 5

[ai]
explanation_style = "beginner"
safety_level = "medium"
auto_execute_plans = false
max_retries = 3
response_timeout = 30

[privacy]
offline_mode = false
log_ai_interactions = false
anonymize_commands = false
"#;
    
    fs::write(&config_path, config_content).expect("Failed to write config file");
    
    // Test configuration validation
    let output = Command::new("cargo")
        .args(&["run", "--", "--validate-config", "--config", config_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute kaido --validate-config");
    
    // Should succeed - cloud_api is optional
    assert!(output.status.success(), "Configuration without cloud_api should be valid");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Configuration is valid"), "Should indicate valid configuration");
}

#[test]
fn test_configuration_file_format_validation() {
    // Test 4: Configuration file format validation
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("invalid_config.toml");
    
    // Create invalid TOML configuration
    let config_content = r#"
[model]
name = "gpt-3.5-turbo"
path = "models/placeholder.gguf"
model_type = "Cloud"
max_tokens = 2048
temperature = 0.7
context_size = 4096

# Missing closing bracket
[safety
require_confirmation_for = ["rm -rf", "sudo"]
auto_confirm_safe_commands = true
"#;
    
    fs::write(&config_path, config_content).expect("Failed to write config file");
    
    // Test configuration validation
    let output = Command::new("cargo")
        .args(&["run", "--", "--validate-config", "--config", config_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute kaido --validate-config");
    
    // Should fail due to invalid TOML
    assert!(!output.status.success(), "Invalid TOML should fail validation");
}

#[test]
fn test_api_key_format_validation() {
    // Test 5: API key format validation
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("api_key_config.toml");
    
    // Create configuration with empty API key
    let config_content = r#"
[model]
name = "gpt-3.5-turbo"
path = "models/placeholder.gguf"
model_type = "Cloud"
max_tokens = 2048
temperature = 0.7
context_size = 4096

[cloud_api]
api_url = "https://api.openai.com/v1/chat/completions"
api_key = ""
model_name = "gpt-3.5-turbo"
timeout_seconds = 30

[safety]
require_confirmation_for = ["rm -rf", "sudo"]
auto_confirm_safe_commands = true
auto_confirm_dangerous = false
log_all_commands = true
max_plan_steps = 10
confirmation_timeout_seconds = 30

[shell]
default_prompt = "kaido> "
history_size = 1000
auto_complete = true
show_execution_time = true
timeout_seconds = 300

[logging]
level = "info"
file = "~/.local/share/kaido/logs/kaido.log"
max_file_size = "10MB"
max_files = 5

[ai]
explanation_style = "beginner"
safety_level = "medium"
auto_execute_plans = false
max_retries = 3
response_timeout = 30

[privacy]
offline_mode = false
log_ai_interactions = false
anonymize_commands = false
"#;
    
    fs::write(&config_path, config_content).expect("Failed to write config file");
    
    // Test configuration validation
    let output = Command::new("cargo")
        .args(&["run", "--", "--validate-config", "--config", config_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute kaido --validate-config");
    
    // Should fail due to empty API key
    assert!(!output.status.success(), "Empty API key should fail validation");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Cloud API key cannot be empty"), "Should show API key error");
}
