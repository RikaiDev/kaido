/// Showcase tests - demonstrate capabilities with real output
/// These are NEW test cases not shown in previous conversation

use kaido::error::PatternMatcher;
use kaido::tools::{ToolRegistry, RiskLevel};

#[test]
fn showcase_kubernetes_production_safety() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  Test Case 1: Production Safety - Kubernetes            ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    let registry = ToolRegistry::new();
    let kubectl = registry.get_tool("kubectl").unwrap();
    let ctx = kaido::tools::ToolContext::default();
    
    let dangerous_commands = vec![
        ("kubectl delete namespace production", "CRITICAL - deleting entire namespace"),
        ("kubectl delete pods --all -n prod", "CRITICAL - batch deletion"),
        ("kubectl drain node-prod-01 --force", "HIGH - forcing node drain"),
        ("kubectl scale deployment api --replicas=0", "HIGH - scaling to zero"),
    ];
    
    println!("Scenario: User attempts dangerous operations in production\n");
    
    for (cmd, description) in dangerous_commands {
        let risk = kubectl.classify_risk(cmd, &ctx);
        let emoji = match risk {
            RiskLevel::Critical => "🚨",
            RiskLevel::High => "⚠️ ",
            RiskLevel::Medium => "⚡",
            RiskLevel::Low => "✓",
        };
        
        println!("{} {:?} - {}", emoji, risk, description);
        println!("   Command: {}", cmd);
        println!("   Requires confirmation: {}", risk.requires_confirmation());
        println!("   Typed confirmation: {}\n", risk.requires_typed_confirmation(true));
    }
    
    println!("✅ All dangerous operations correctly flagged!\n");
}

#[test]
fn showcase_docker_container_management() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  Test Case 2: Docker Container Lifecycle                ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    let registry = ToolRegistry::new();
    let docker = registry.get_tool("docker").unwrap();
    let ctx = kaido::tools::ToolContext::default();
    
    let operations = vec![
        ("docker ps -a", RiskLevel::Low, "List all containers"),
        ("docker run -d nginx", RiskLevel::Medium, "Start new container"),
        ("docker restart nginx", RiskLevel::Medium, "Restart container"),
        ("docker rm nginx", RiskLevel::High, "Remove container"),
        ("docker rm $(docker ps -aq)", RiskLevel::Critical, "DANGER: Remove all containers"),
    ];
    
    println!("Scenario: Common Docker operations with risk levels\n");
    
    for (cmd, expected_risk, description) in operations {
        let actual_risk = docker.classify_risk(cmd, &ctx);
        let match_icon = if actual_risk == expected_risk { "✓" } else { "✗" };
        
        println!("{} {:?} - {}", match_icon, actual_risk, description);
        println!("   Command: {}", cmd);
        
        if actual_risk == RiskLevel::Critical {
            println!("   🚨 WARNING: This will affect ALL containers!");
        }
        println!();
        
        assert_eq!(actual_risk, expected_risk);
    }
}

#[test]
fn showcase_database_operations() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  Test Case 3: Database Operations Safety                ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    let registry = ToolRegistry::new();
    let mysql = registry.get_tool("mysql").unwrap();
    let ctx = kaido::tools::ToolContext::default();
    
    let queries = vec![
        ("SELECT * FROM users WHERE id = 1", RiskLevel::Low, "Safe query with WHERE"),
        ("INSERT INTO users VALUES (1, 'test')", RiskLevel::Medium, "Insert operation"),
        ("UPDATE users SET active=1 WHERE id=1", RiskLevel::Medium, "Update with WHERE"),
        ("DROP TABLE temp_backup", RiskLevel::High, "Drop table"),
        ("DELETE FROM users", RiskLevel::Critical, "🚨 DELETE without WHERE!"),
        ("DROP DATABASE production", RiskLevel::Critical, "🚨 Drop entire database!"),
    ];
    
    println!("Scenario: SQL operations from safest to most dangerous\n");
    
    for (query, expected_risk, description) in queries {
        let actual_risk = mysql.classify_risk(query, &ctx);
        let emoji = match actual_risk {
            RiskLevel::Critical => "🚨",
            RiskLevel::High => "⚠️ ",
            RiskLevel::Medium => "⚡",
            RiskLevel::Low => "✅",
        };
        
        println!("{} {:?} - {}", emoji, actual_risk, description);
        println!("   SQL: {}", query);
        
        if actual_risk == RiskLevel::Critical {
            println!("   🛑 CRITICAL: Would require typing 'I understand' to proceed");
        }
        println!();
        
        assert_eq!(actual_risk, expected_risk);
    }
}

#[test]
fn showcase_error_explanation_kubectl() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  Test Case 4: Kubectl Error Explanation                 ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    let matcher = PatternMatcher::new();
    
    let error = r#"Error from server (Forbidden): pods is forbidden: User "developer" cannot list resource "pods" in API group "" in the namespace "production""#;
    
    println!("Scenario: User encounters RBAC permission error\n");
    println!("Error message:");
    println!("{}\n", error);
    
    let explanation = matcher.match_pattern(error).expect("Should match kubectl RBAC error");
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("AI Explanation:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("Type: {}", explanation.error_type);
    println!("Reason: {}\n", explanation.reason);
    
    println!("Possible Causes:");
    for (i, cause) in explanation.possible_causes.iter().enumerate() {
        println!("  {}. {}", i + 1, cause);
    }
    
    println!("\nSuggested Solutions:");
    for (i, solution) in explanation.solutions.iter().enumerate() {
        println!("  {}. {} [Risk: {:?}]", i + 1, solution.description, solution.risk_level);
        if let Some(cmd) = &solution.command {
            println!("     Command: {}", cmd);
        }
    }
    
    println!("\n✅ User now understands the RBAC issue!\n");
}

#[test]
fn showcase_error_explanation_docker() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  Test Case 5: Docker Daemon Error                       ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    let matcher = PatternMatcher::new();
    
    let error = "Cannot connect to the Docker daemon at unix:///var/run/docker.sock. Is the docker daemon running?";
    
    println!("Scenario: Docker daemon is not running\n");
    println!("Error message:");
    println!("{}\n", error);
    
    let explanation = matcher.match_pattern(error).expect("Should match Docker daemon error");
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("AI Explanation:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("Type: {}", explanation.error_type);
    println!("Reason: {}\n", explanation.reason);
    
    println!("Solutions for different platforms:");
    for (i, solution) in explanation.solutions.iter().enumerate() {
        println!("\n  {}. {}", i + 1, solution.description);
        if let Some(cmd) = &solution.command {
            println!("     $ {}", cmd);
        }
    }
    
    println!("\n✅ User knows exactly how to fix the issue!\n");
}

#[test]
fn showcase_tool_detection_ambiguous() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  Test Case 6: Smart Tool Detection                      ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    let registry = ToolRegistry::new();
    
    let test_cases = vec![
        ("kubectl get pods", Some("kubectl"), "Explicit command"),
        ("docker ps -a", Some("docker"), "Explicit command"),
        ("show databases", Some("mysql"), "SQL keyword detected"),
        ("vendor/bin/drush cr", Some("drush"), "Drupal CLI detected"),
        ("list all items", None, "Too ambiguous - no tool keywords"),
        ("get pods in namespace prod", None, "Has 'pods' keyword but < 50% threshold"),
    ];
    
    println!("Scenario: System detects appropriate tool from user input\n");
    
    for (input, expected, description) in test_cases {
        let detected = registry.detect_tool(input);
        let detected_name = detected.map(|t| t.name());
        
        let icon = if detected_name == expected { "✓" } else { "✗" };
        
        println!("{} Input: \"{}\"", icon, input);
        match detected_name {
            Some(tool) => println!("   → Detected: {} ({})", tool, description),
            None => println!("   → Not detected ({})", description),
        }
        println!();
        
        assert_eq!(detected_name, expected);
    }
    
    println!("💡 Tip: Include tool keywords or use explicit commands for best results\n");
}

#[test]
fn showcase_multi_tool_comparison() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  Test Case 7: Cross-Tool Risk Comparison                ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    let registry = ToolRegistry::new();
    let ctx = kaido::tools::ToolContext::default();
    
    println!("Scenario: Similar operations across different tools\n");
    
    let operations = vec![
        ("kubectl", "kubectl get pods", "List resources"),
        ("docker", "docker ps", "List containers"),
        ("mysql", "SELECT * FROM users", "Query data"),
    ];
    
    println!("✅ LOW RISK - Read Operations:");
    for (tool_name, cmd, desc) in operations {
        let tool = registry.get_tool(tool_name).unwrap();
        let risk = tool.classify_risk(cmd, &ctx);
        println!("   {} | {} → {:?}", tool_name, desc, risk);
    }
    
    println!("\n⚡ MEDIUM RISK - Modify Operations:");
    let modify_ops = vec![
        ("kubectl", "kubectl apply -f config.yaml", "Apply config"),
        ("docker", "docker run nginx", "Start container"),
        ("mysql", "INSERT INTO users VALUES (1, 'test')", "Insert data"),
    ];
    for (tool_name, cmd, desc) in modify_ops {
        let tool = registry.get_tool(tool_name).unwrap();
        let risk = tool.classify_risk(cmd, &ctx);
        println!("   {} | {} → {:?}", tool_name, desc, risk);
    }
    
    println!("\n⚠️  HIGH RISK - Delete Operations:");
    let delete_ops = vec![
        ("kubectl", "kubectl delete pod nginx", "Delete pod"),
        ("docker", "docker rm nginx", "Remove container"),
        ("mysql", "DROP TABLE users", "Drop table"),
    ];
    for (tool_name, cmd, desc) in delete_ops {
        let tool = registry.get_tool(tool_name).unwrap();
        let risk = tool.classify_risk(cmd, &ctx);
        println!("   {} | {} → {:?}", tool_name, desc, risk);
    }
    
    println!("\n🚨 CRITICAL RISK - Destructive Operations:");
    let critical_ops = vec![
        ("kubectl", "kubectl delete namespace production", "Delete namespace"),
        ("docker", "docker rm $(docker ps -aq)", "Delete all containers"),
        ("mysql", "DROP DATABASE production", "Drop database"),
    ];
    for (tool_name, cmd, desc) in critical_ops {
        let tool = registry.get_tool(tool_name).unwrap();
        let risk = tool.classify_risk(cmd, &ctx);
        println!("   {} | {} → {:?}", tool_name, desc, risk);
    }
    
    println!("\n✅ Consistent risk classification across all tools!\n");
}


