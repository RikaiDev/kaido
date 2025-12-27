/// Real-world error explanation tests
/// Based on actual errors users encounter

use kaido::error::PatternMatcher;

#[test]
fn test_real_drush_sqlq_error() {
    // Your actual reported error
    let matcher = PatternMatcher::new();
    let error = r#"
In SqlCommands.php line 189:

  Query failed. Rerun with --debug to see any error message. --------------

  database.mysql

  --------------

  ERROR 1064 (42000) at line 1: You have an error in your SQL syntax; check the manual that corresponds to your MariaDB server version for the right syntax to use near 'database.mysql' at line 1
"#;
    
    let explanation = matcher.match_pattern(error);
    assert!(explanation.is_some(), "Should detect drush sqlq error");
    
    let exp = explanation.unwrap();
    assert_eq!(exp.error_type, "Drush SQL File Execution Error");
    assert!(!exp.solutions.is_empty(), "Should provide solutions");
    assert!(
        exp.solutions[0].command.as_ref().unwrap().contains("sql:cli"),
        "Should recommend sql:cli"
    );
    
    println!("\n=== Real Drush Error Explanation ===");
    println!("Type: {}", exp.error_type);
    println!("Reason: {}", exp.reason);
    println!("\nSolutions:");
    for (i, sol) in exp.solutions.iter().enumerate() {
        println!("{}. {} [{}]", i + 1, sol.description, sol.risk_level);
        if let Some(cmd) = &sol.command {
            println!("   Command: {}", cmd);
        }
    }
}

#[test]
fn test_real_mysql_syntax_error() {
    let matcher = PatternMatcher::new();
    let error = "ERROR 1064 (42000): You have an error in your SQL syntax; check the manual that corresponds to your MySQL server version for the right syntax to use near 'SLECT * FROM users' at line 1";
    
    let explanation = matcher.match_pattern(error);
    assert!(explanation.is_some(), "Should detect MySQL syntax error");
    
    let exp = explanation.unwrap();
    println!("\n=== MySQL Syntax Error Explanation ===");
    println!("Type: {}", exp.error_type);
    println!("Reason: {}", exp.reason);
}

#[test]
fn test_real_docker_daemon_error() {
    let matcher = PatternMatcher::new();
    let error = r#"Cannot connect to the Docker daemon at unix:///var/run/docker.sock. Is the docker daemon running?"#;
    
    let explanation = matcher.match_pattern(error);
    assert!(explanation.is_some(), "Should detect Docker daemon error");
    
    let exp = explanation.unwrap();
    assert_eq!(exp.error_type, "Docker Daemon Not Running");
    assert!(exp.solutions.len() >= 2, "Should provide multiple solutions");
    
    println!("\n=== Docker Daemon Error Explanation ===");
    println!("Type: {}", exp.error_type);
    println!("Solutions: {} provided", exp.solutions.len());
}

#[test]
fn test_real_kubectl_permission_error() {
    let matcher = PatternMatcher::new();
    let error = r#"Error from server (Forbidden): pods is forbidden: User "developer" cannot list resource "pods" in API group "" in the namespace "production""#;
    
    let explanation = matcher.match_pattern(error);
    assert!(explanation.is_some(), "Should detect kubectl RBAC error");
    
    let exp = explanation.unwrap();
    println!("\n=== Kubectl Permission Error Explanation ===");
    println!("Type: {}", exp.error_type);
    println!("Reason: {}", exp.reason);
}

#[test]
fn test_real_mysql_access_denied() {
    let matcher = PatternMatcher::new();
    let error = "ERROR 1045 (28000): Access denied for user 'root'@'localhost' (using password: YES)";
    
    let explanation = matcher.match_pattern(error);
    assert!(explanation.is_some(), "Should detect MySQL auth error");
    
    let exp = explanation.unwrap();
    assert_eq!(exp.error_type, "MySQL Authentication Failed");
    
    println!("\n=== MySQL Auth Error Explanation ===");
    println!("Type: {}", exp.error_type);
}

#[test]
fn test_real_kubectl_context_not_set() {
    let matcher = PatternMatcher::new();
    let error = "error: current-context is not set";
    
    let explanation = matcher.match_pattern(error);
    assert!(explanation.is_some(), "Should detect kubectl context error");
    
    let exp = explanation.unwrap();
    assert!(
        exp.solutions.iter().any(|s| s.command.as_ref().map_or(false, |c| c.contains("get-contexts"))),
        "Should suggest viewing contexts"
    );
}

#[test]
fn test_real_docker_image_not_found() {
    let matcher = PatternMatcher::new();
    let error = "Unable to find image 'nginx:latestt' locally";
    
    let explanation = matcher.match_pattern(error);
    assert!(explanation.is_some(), "Should detect Docker image error");
    
    let exp = explanation.unwrap();
    println!("\n=== Docker Image Not Found Explanation ===");
    println!("Type: {}", exp.error_type);
}


