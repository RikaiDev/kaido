/// Your actual reported scenario: drush sqlq database.mysql
/// Complete end-to-end test

use kaido::tools::{DrushTool, Tool};

#[test]
fn test_drush_sqlq_file_error_detection() {
    let drush = DrushTool::new();
    
    // Your exact error message
    let error = r#"
vendor/bin/drush sqlq database.mysql

In SqlCommands.php line 189:

  Query failed. Rerun with --debug to see any error message. --------------

  database.mysql

  --------------

  ERROR 1064 (42000) at line 1: You have an error in your SQL syntax; check the manual that corresponds to your MariaDB server version for the right syntax to use near 'database.mysql' at line 1
"#;
    
    // Test error explanation
    let explanation = drush.explain_error(error);
    assert!(explanation.is_some(), "Drush should provide error explanation");
    
    let exp = explanation.unwrap();
    
    println!("\n=== Your Drush Scenario Test ===");
    println!("Error Type: {}", exp.error_type);
    println!("Reason: {}", exp.reason);
    println!("\nPossible Causes:");
    for (i, cause) in exp.possible_causes.iter().enumerate() {
        println!("  {}. {}", i + 1, cause);
    }
    println!("\nSolutions:");
    for (i, sol) in exp.solutions.iter().enumerate() {
        println!("  {}. {} [Risk: {}]", i + 1, sol.description, sol.risk_level);
        if let Some(cmd) = &sol.command {
            println!("     Command: {}", cmd);
        }
    }
    
    // Verify solution quality
    assert_eq!(exp.error_type, "Drush SQL File Execution Error");
    assert_eq!(exp.solutions.len(), 3, "Should provide 3 solutions");
    assert_eq!(exp.recommended_solution, 0, "Should recommend first solution");
    
    // Verify first solution is sql:cli (the correct one)
    let first_solution = &exp.solutions[0];
    assert!(
        first_solution.command.as_ref().unwrap().contains("sql:cli"),
        "First solution should use sql:cli"
    );
    assert!(
        first_solution.description.contains("推薦"),
        "First solution should be marked as recommended"
    );
}

#[test]
fn test_drush_correct_commands() {
    let drush = DrushTool::new();
    
    // These should NOT trigger error explanation
    let correct_commands = vec![
        "drush cr",
        "drush sql:cli",
        "drush sqlq 'SHOW TABLES'",
    ];
    
    println!("\n=== Drush Correct Commands ===");
    for cmd in correct_commands {
        let explanation = drush.explain_error(cmd);
        assert!(
            explanation.is_none(),
            "Should not explain error for correct command: {}",
            cmd
        );
        println!("✓ '{}' - recognized as correct", cmd);
    }
}

#[test]
fn test_drush_tool_detection() {
    let drush = DrushTool::new();
    
    let test_cases = vec![
        ("drush cr", 1.0), // Explicit drush → 100%
        ("vendor/bin/drush sqlq", 1.0), // Explicit path → 100%
        ("drupal cache clear", 0.6), // Contains "drupal" → 60%
        ("clear cache", 0.0), // No drush keyword → 0%
    ];
    
    println!("\n=== Drush Detection Confidence ===");
    for (input, expected_confidence) in test_cases {
        let confidence = drush.detect_intent(input);
        println!("'{}' → {:.0}% confidence", input, confidence * 100.0);
        assert_eq!(
            confidence, expected_confidence,
            "Wrong confidence for: {}",
            input
        );
    }
}

#[test]
fn test_complete_drush_workflow() {
    println!("\n=== Complete Drush Workflow Test ===");
    println!("Scenario: User tries 'vendor/bin/drush sqlq database.mysql'");
    
    // 1. Tool detection
    let drush = DrushTool::new();
    let user_input = "vendor/bin/drush sqlq database.mysql";
    let detection = drush.detect_intent(user_input);
    println!("1. Detection: {:.0}% confidence → drush", detection * 100.0);
    assert_eq!(detection, 1.0);
    
    // 2. Error occurs
    let error = "ERROR 1064 (42000) at line 1: drush sqlq database.mysql";
    println!("2. Error occurs: {}", error);
    
    // 3. Error explanation
    let explanation = drush.explain_error(error);
    assert!(explanation.is_some());
    let exp = explanation.unwrap();
    println!("3. Error explained: {}", exp.error_type);
    
    // 4. Solution provided
    let solution = &exp.solutions[exp.recommended_solution];
    println!("4. Recommended solution:");
    println!("   {}", solution.description);
    println!("   Command: {}", solution.command.as_ref().unwrap());
    
    // 5. Verify solution is correct
    assert!(
        solution.command.as_ref().unwrap().contains("sql:cli < database.mysql"),
        "Solution should use correct syntax"
    );
    
    println!("\n✅ Complete workflow validated!");
}


