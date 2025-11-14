// Integration tests for pattern execution
use rig_patterns::{Agent, Orchestrator, Pattern, ResolutionStrategy, Aggregation};

/// Helper to create a simple test agent
fn create_test_agent(id: &str, response: &str) -> Result<Agent, Box<dyn std::error::Error>> {
    // For testing, we'd normally use mock agents, but since we don't have that infrastructure,
    // we'll rely on the unit tests in each pattern module
    // This file demonstrates the test structure
    Ok(Agent::from_openai(id, "test-key", "gpt-4", response)?)
}

#[tokio::test]
#[ignore] // Ignore by default since it requires API keys
async fn test_sequential_pattern_completes() {
    // This test would verify Sequential pattern completes
    // Requires API keys to run
    let agents = vec![
        create_test_agent("agent1", "I am agent 1").unwrap(),
        create_test_agent("agent2", "I am agent 2").unwrap(),
    ];

    let orchestrator = Orchestrator::new(agents)
        .pattern(Pattern::Sequential)
        .build()
        .unwrap();

    let result = orchestrator.execute("test input").await;
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore] // Ignore by default since it requires API keys
async fn test_concurrent_pattern_completes() {
    let agents = vec![
        create_test_agent("agent1", "I am agent 1").unwrap(),
        create_test_agent("agent2", "I am agent 2").unwrap(),
    ];

    let orchestrator = Orchestrator::new(agents)
        .pattern(Pattern::Concurrent {
            aggregation: Aggregation::Combine,
        })
        .build()
        .unwrap();

    let result = orchestrator.execute("test input").await;
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_group_chat_pattern_completes() {
    let agents = vec![
        create_test_agent("agent1", "I am agent 1").unwrap(),
        create_test_agent("agent2", "I am agent 2").unwrap(),
    ];

    let orchestrator = Orchestrator::new(agents)
        .pattern(Pattern::GroupChat {
            max_rounds: 2,
            resolution: ResolutionStrategy::AllRounds,
        })
        .build()
        .unwrap();

    let result = orchestrator.execute("test input").await;
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_handoff_pattern_completes() {
    let agents = vec![
        create_test_agent("agent1", "I am agent 1").unwrap(),
        create_test_agent("agent2", "I am agent 2").unwrap(),
    ];

    let orchestrator = Orchestrator::new(agents)
        .pattern(Pattern::Handoff { max_hops: 3 })
        .build()
        .unwrap();

    let result = orchestrator.execute("test input").await;
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_magentic_pattern_completes() {
    let agents = vec![
        create_test_agent("manager", "I am the manager").unwrap(),
        create_test_agent("worker", "I am a worker").unwrap(),
    ];

    let orchestrator = Orchestrator::new(agents)
        .pattern(Pattern::Magentic { max_iterations: 3 })
        .build()
        .unwrap();

    let result = orchestrator.execute("test input").await;
    assert!(result.is_ok());
}

#[test]
fn test_pattern_enum_variants() {
    // Test that all pattern variants can be constructed
    let _sequential = Pattern::Sequential;
    let _concurrent = Pattern::Concurrent {
        aggregation: Aggregation::Combine,
    };
    let _group_chat = Pattern::GroupChat {
        max_rounds: 3,
        resolution: ResolutionStrategy::Consensus,
    };
    let _handoff = Pattern::Handoff { max_hops: 5 };
    let _magentic = Pattern::Magentic { max_iterations: 5 };
}

#[test]
fn test_resolution_strategy_variants() {
    // Test all resolution strategies
    let _consensus = ResolutionStrategy::Consensus;
    let _first = ResolutionStrategy::FirstToComplete;
    let _all = ResolutionStrategy::AllRounds;
    let _majority = ResolutionStrategy::Majority;

    // Test default
    assert_eq!(ResolutionStrategy::default(), ResolutionStrategy::Consensus);
}

#[test]
fn test_aggregation_variants() {
    let _consensus = Aggregation::Consensus;
    let _vote = Aggregation::Vote;
    let _combine = Aggregation::Combine;
}
