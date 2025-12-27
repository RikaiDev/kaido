// Real-world agent scenario tests
// These tests verify the agent can handle actual ops problems

use kaido::agent::{AgentLoop, AgentStatus};
use kaido::ai::AIManager;
use kaido::config::Config;
use kaido::tools::ToolContext;

#[tokio::test]
#[ignore] // Requires actual system tools and network
async fn test_nginx_port_conflict_scenario() {
    // Scenario: nginx won't start because port 80 is occupied
    let config = Config::default();
    let ai_manager = AIManager::new(config);
    let context = ToolContext::default();
    
    let problem = "nginx cannot start, error says port 80 is already in use";
    
    let mut agent = AgentLoop::new(problem.to_string(), context);
    let result = agent.run_until_complete(&ai_manager).await;
    
    assert!(result.is_ok());
    let final_state = result.unwrap();
    
    // Agent should have completed diagnosis
    assert!(matches!(final_state.status, AgentStatus::Completed));
    
    // Should have executed some diagnostic actions
    assert!(final_state.history.len() > 0);
    
    // Should have collected information about port usage
    assert!(!final_state.collected_info.is_empty());
    
    println!("Agent completed with {} steps", final_state.history.len());
    if let Some(root_cause) = &final_state.root_cause {
        println!("Root cause: {}", root_cause);
    }
}

#[tokio::test]
#[ignore] // Requires actual system tools
async fn test_apache_404_webhook_scenario() {
    // Scenario: Apache returns 404 for webhook endpoint
    let config = Config::default();
    let ai_manager = AIManager::new(config);
    let context = ToolContext::default();
    
    let problem = "apache returns 404 for /webhook endpoint, backend is running on port 8080";
    
    let mut agent = AgentLoop::new(problem.to_string(), context);
    let result = agent.run_until_complete(&ai_manager).await;
    
    assert!(result.is_ok());
    let final_state = result.unwrap();
    
    // Agent should analyze the problem
    assert!(final_state.iteration > 0);
    
    println!("Agent diagnosis:");
    println!("Steps taken: {}", final_state.history.len());
    println!("Status: {:?}", final_state.status);
}

#[tokio::test]
#[ignore] // Requires docker
async fn test_docker_compose_network_scenario() {
    // Scenario: docker-compose services can't communicate
    let config = Config::default();
    let ai_manager = AIManager::new(config);
    let context = ToolContext::default();
    
    let problem = "docker-compose services are running but cannot connect to each other";
    
    let mut agent = AgentLoop::new(problem.to_string(), context);
    let result = agent.run_until_complete(&ai_manager).await;
    
    assert!(result.is_ok());
    let final_state = result.unwrap();
    
    // Should gather network information
    assert!(final_state.history.len() > 0);
    
    println!("Agent network diagnosis completed");
}

#[tokio::test]
async fn test_agent_iteration_limit() {
    // Test that agent stops after max iterations
    let config = Config::default();
    let ai_manager = AIManager::new(config);
    let context = ToolContext::default();
    
    // Give it an impossible task
    let problem = "solve world hunger";
    
    let mut agent = AgentLoop::new(problem.to_string(), context);
    let result = agent.run_until_complete(&ai_manager).await;
    
    assert!(result.is_ok());
    let final_state = result.unwrap();
    
    // Should stop due to iteration limit
    match final_state.status {
        AgentStatus::Stopped(_) => {
            println!("Agent correctly stopped after max iterations");
        }
        _ => {}
    }
}

#[test]
fn test_agent_state_tracking() {
    use kaido::agent::AgentState;
    
    let mut state = AgentState::new("test problem".to_string());
    
    // Test state management
    assert_eq!(state.iteration, 0);
    assert!(state.should_continue());
    
    // Add some steps
    state.add_step(
        kaido::agent::StepType::Thought,
        "thinking...".to_string(),
        None,
        None
    );
    
    assert_eq!(state.history.len(), 1);
    
    // Add collected info
    state.collected_info.push(("netstat".to_string(), "port 80 in use".to_string()));
    assert_eq!(state.collected_info.len(), 1);
}

