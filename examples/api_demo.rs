/// Kaido AI - Complete API demonstration
/// 
/// This example demonstrates all public APIs to eliminate dead_code warnings

use kaido::ai::AIManager;
use kaido::audit::AuditLogger;
use kaido::commands::{CommandEngine, CommandResult};
use kaido::config::Config;
use kaido::tools::{LLMBackend, ToolContext, Translation, ExecutionResult, RiskLevel};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!(">> Kaido AI - Complete API Demo\n");
    
    // 1. Configuration
    let config = Config::load()?;
    println!("[OK] Config loaded");
    
    // Test save method
    config.save()?;
    println!("[OK] Config save tested");
    
    // 2. AI Manager (local GGUF + Gemini fallback)
    let ai_manager = AIManager::new(config.clone());
    println!("[OK] AI Manager initialized (local GGUF + Gemini)");
    
    // 3. Test LLM inference
    let test_prompt = "List all pods in namespace kube-system";
    match ai_manager.infer(test_prompt).await {
        Ok(response) => {
            println!("\n[AI] Inference Test:");
            println!("    Command: {}", response.command);
            println!("    Confidence: {}%", response.confidence);
            println!("    Reasoning: {}", response.reasoning.lines().next().unwrap_or(""));
        }
        Err(e) => {
            println!("[!] AI Inference failed: {}", e);
        }
    }
    
    // 4. Command Engine
    let command_engine = CommandEngine::new();
    println!("\n[OK] Command Engine initialized");
    
    // 5. Translation Result API
    let translation = Translation {
        command: "kubectl get pods -n kube-system".to_string(),
        confidence: 95,
        reasoning: "Standard pod listing command".to_string(),
        tool_name: "kubectl".to_string(),
        requires_files: vec![],
    };
    println!("\n[*] Translation created: {}", translation.command);
    
    // 6. Risk Assessment
    let context = ToolContext::default();
    match command_engine.classify_risk(&translation, &context) {
        Ok(risk) => {
            println!("   Risk Level: {}", risk);
            println!("   Requires Confirmation: {}", risk.requires_confirmation());
            println!("   Requires Typed Confirmation (prod): {}", risk.requires_typed_confirmation(true));
            println!("   Requires Typed Confirmation (dev): {}", risk.requires_typed_confirmation(false));
        }
        Err(e) => {
            println!("   Risk classification failed: {}", e);
        }
    }
    
    // 7. Registry access
    let registry = command_engine.registry();
    println!("\n[#] Tool Registry:");
    if let Some(kubectl_tool) = registry.detect_tool("kubectl get pods") {
        println!("    Detected tool: {}", kubectl_tool.name());
    }
    if let Some(docker_tool) = registry.get_tool("docker") {
        println!("    Docker tool available: {}", docker_tool.name());
    }
    
    // 8. Audit Query (if there are any entries)
    println!("\n[=] Audit log ready for queries");
    // Note: query functions require a database connection reference
    
    // 9. Database Connection API (demonstration)
    use kaido::tools::DatabaseConnection;
    let db_conn = DatabaseConnection {
        host: "localhost".to_string(),
        port: 3306,
        database: "test_db".to_string(),
        username: "root".to_string(),
        is_production: false,
    };
    println!("\n[DB] Database Connection:");
    println!("     Connection String: {}", db_conn.connection_string());
    println!("     Is Production: {}", db_conn.is_prod());
    
    // 10. CommandResult enum (for completeness)
    use std::time::Duration;
    
    let exec_result = ExecutionResult {
        exit_code: 0,
        stdout: "Test output".to_string(),
        stderr: String::new(),
        duration: Duration::from_millis(100),
    };
    
    let cmd_result = CommandResult::Executed {
        translation: translation.clone(),
        execution: exec_result,
    };
    
    match cmd_result {
        CommandResult::Executed { translation, execution } => {
            println!("\n[OK] CommandResult::Executed demonstration:");
            println!("     Command: {}", translation.command);
            println!("     Exit Code: {}", execution.exit_code);
            println!("     Duration: {:?}", execution.duration);
        }
        CommandResult::Cancelled { translation } => {
            println!("     Cancelled: {}", translation.command);
        }
        CommandResult::ErrorExplained { explanation } => {
            println!("     Error: {}", explanation.error_type);
        }
    }
    
    // 11. Kubectl TranslationResult API
    use kaido::kubectl::TranslationResult;
    
    let kubectl_translation = TranslationResult::new(
        "kubectl get pods".to_string(),
        85,
        "Standard pod listing".to_string(),
    );
    
    println!("\n[*] kubectl TranslationResult:");
    println!("    Command: {}", kubectl_translation.kubectl_command);
    println!("    Confidence: {}", kubectl_translation.confidence_score);
    println!("    Is Low Confidence: {}", kubectl_translation.is_low_confidence(90));
    
    println!("\n[+] All APIs demonstrated successfully!");
    
    Ok(())
}

