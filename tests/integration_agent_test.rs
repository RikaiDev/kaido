// Integration tests for agent functionality
use kaido::agent::{AgentLoop, AgentState, StepType};
use kaido::tools::ToolContext;

#[test]
fn test_agent_loop_creation() {
    let problem = "test problem".to_string();
    let context = ToolContext::default();

    let agent = AgentLoop::new(problem.clone(), context);
    let state = agent.state();

    assert_eq!(state.task, problem);
    assert_eq!(state.iteration, 0);
    assert!(state.history.is_empty());
}

#[test]
fn test_agent_state_summary() {
    let state = AgentState::new("Test task".to_string());
    let summary = state.summary();

    assert!(summary.contains("Test task"));
    assert!(summary.contains("Steps: 0"));
}

#[test]
fn test_agent_step_filtering() {
    let mut state = AgentState::new("test".to_string());

    state.add_step(StepType::Thought, "think 1".to_string(), None, None);
    state.add_step(
        StepType::Action,
        "action 1".to_string(),
        Some("tool".to_string()),
        None,
    );
    state.add_step(StepType::Observation, "obs 1".to_string(), None, Some(true));
    state.add_step(StepType::Thought, "think 2".to_string(), None, None);

    let thoughts = state.get_recent_steps(StepType::Thought, 10);
    assert_eq!(thoughts.len(), 2);

    let actions = state.get_recent_steps(StepType::Action, 10);
    assert_eq!(actions.len(), 1);
}

#[test]
fn test_agent_should_continue() {
    let mut state = AgentState::new("test".to_string());

    // Should continue initially
    assert!(state.should_continue());

    // Should stop after max iterations
    state.iteration = 20;
    assert!(!state.should_continue());
}
