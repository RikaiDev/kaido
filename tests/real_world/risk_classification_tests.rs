/// Real-world risk classification tests
/// Ensure dangerous commands are properly classified

use kaido::tools::{ToolRegistry, ToolContext, RiskLevel};

#[test]
fn test_kubectl_critical_risks() {
    let registry = ToolRegistry::new();
    let context = ToolContext::default();
    let kubectl = registry.get_tool("kubectl").unwrap();
    
    let critical_commands = vec![
        ("kubectl delete namespace production", RiskLevel::Critical),
        ("kubectl delete pods --all", RiskLevel::Critical),
        ("kubectl delete deployment --all -n default", RiskLevel::Critical),
    ];
    
    println!("\n=== Kubectl CRITICAL Risk Commands ===");
    for (cmd, expected_risk) in critical_commands {
        let risk = kubectl.classify_risk(cmd, &context);
        println!("{:?}: {}", risk, cmd);
        assert_eq!(risk, expected_risk, "Wrong risk for: {}", cmd);
    }
}

#[test]
fn test_kubectl_high_risks() {
    let registry = ToolRegistry::new();
    let context = ToolContext::default();
    let kubectl = registry.get_tool("kubectl").unwrap();
    
    let high_risk_commands = vec![
        ("kubectl delete deployment nginx", RiskLevel::High),
        ("kubectl delete pod test-pod", RiskLevel::High),
        ("kubectl drain node-01", RiskLevel::High),
        ("kubectl scale deployment nginx --replicas=0", RiskLevel::High),
    ];
    
    println!("\n=== Kubectl HIGH Risk Commands ===");
    for (cmd, expected_risk) in high_risk_commands {
        let risk = kubectl.classify_risk(cmd, &context);
        println!("{:?}: {}", risk, cmd);
        assert_eq!(risk, expected_risk, "Wrong risk for: {}", cmd);
    }
}

#[test]
fn test_docker_critical_risks() {
    let registry = ToolRegistry::new();
    let context = ToolContext::default();
    let docker = registry.get_tool("docker").unwrap();
    
    let critical_commands = vec![
        ("docker rm $(docker ps -aq)", RiskLevel::Critical),
        ("docker rmi `docker images -q`", RiskLevel::Critical),
    ];
    
    println!("\n=== Docker CRITICAL Risk Commands ===");
    for (cmd, expected_risk) in critical_commands {
        let risk = docker.classify_risk(cmd, &context);
        println!("{:?}: {}", risk, cmd);
        assert_eq!(risk, expected_risk, "Wrong risk for: {}", cmd);
    }
}

#[test]
fn test_sql_critical_risks() {
    let registry = ToolRegistry::new();
    let context = ToolContext::default();
    let mysql = registry.get_tool("mysql").unwrap();
    
    let critical_commands = vec![
        ("DROP DATABASE production", RiskLevel::Critical),
        ("DELETE FROM users", RiskLevel::Critical), // No WHERE clause
        ("TRUNCATE users", RiskLevel::Critical),
    ];
    
    println!("\n=== SQL CRITICAL Risk Commands ===");
    for (cmd, expected_risk) in critical_commands {
        let risk = mysql.classify_risk(cmd, &context);
        println!("{:?}: {}", risk, cmd);
        assert_eq!(risk, expected_risk, "Wrong risk for: {}", cmd);
    }
}

#[test]
fn test_safe_commands() {
    let registry = ToolRegistry::new();
    let context = ToolContext::default();
    
    let safe_commands = vec![
        ("kubectl", "kubectl get pods"),
        ("docker", "docker ps -a"),
        ("mysql", "SELECT * FROM users WHERE id = 1"),
        ("mysql", "SHOW DATABASES"),
    ];
    
    println!("\n=== Safe (LOW Risk) Commands ===");
    for (tool_name, cmd) in safe_commands {
        let tool = registry.get_tool(tool_name).unwrap();
        let risk = tool.classify_risk(cmd, &context);
        println!("{:?}: {}", risk, cmd);
        assert_eq!(risk, RiskLevel::Low, "Wrong risk for safe command: {}", cmd);
    }
}

#[test]
fn test_medium_risk_commands() {
    let registry = ToolRegistry::new();
    let context = ToolContext::default();
    
    let medium_commands = vec![
        ("kubectl", "kubectl apply -f deployment.yaml"),
        ("kubectl", "kubectl scale deployment nginx --replicas=3"),
        ("docker", "docker run nginx"),
        ("docker", "docker restart nginx"),
        ("mysql", "INSERT INTO users VALUES (1, 'test')"),
        ("mysql", "UPDATE users SET name='test' WHERE id=1"),
    ];
    
    println!("\n=== MEDIUM Risk Commands ===");
    for (tool_name, cmd) in medium_commands {
        let tool = registry.get_tool(tool_name).unwrap();
        let risk = tool.classify_risk(cmd, &context);
        println!("{:?}: {}", risk, cmd);
        assert_eq!(risk, RiskLevel::Medium, "Wrong risk for: {}", cmd);
    }
}

#[test]
fn test_risk_confirmation_requirements() {
    println!("\n=== Risk Confirmation Requirements ===");
    
    let test_cases = vec![
        (RiskLevel::Low, false, "No confirmation needed"),
        (RiskLevel::Medium, true, "Yes/No confirmation"),
        (RiskLevel::High, true, "Typed confirmation in production"),
        (RiskLevel::Critical, true, "Always typed confirmation"),
    ];
    
    for (risk, should_confirm, description) in test_cases {
        let requires_conf = risk.requires_confirmation();
        println!("{:?}: {} - {}", risk, requires_conf, description);
        assert_eq!(requires_conf, should_confirm);
    }
    
    // Test production-specific requirements
    assert!(RiskLevel::High.requires_typed_confirmation(true), "HIGH in production needs typed");
    assert!(!RiskLevel::High.requires_typed_confirmation(false), "HIGH in dev doesn't need typed");
    assert!(RiskLevel::Critical.requires_typed_confirmation(true), "CRITICAL always needs typed");
    assert!(RiskLevel::Critical.requires_typed_confirmation(false), "CRITICAL always needs typed");
}


