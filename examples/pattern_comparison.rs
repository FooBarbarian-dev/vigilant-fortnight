//! Pattern comparison example
//!
//! This example runs the same input through all 5 orchestration patterns
//! and compares their results, helping you understand when to use each pattern.
//!
//! Note: This example requires API credentials and won't run as-is.

use anyhow::Result;
use rig_patterns::{Agent, Aggregation, Orchestrator, Pattern};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("=== rig-patterns Pattern Comparison ===\n");
    println!("This example compares all 5 orchestration patterns on the same task.\n");

    // The task we'll use for comparison
    let input = "Analyze the pros and cons of microservices architecture and provide a recommendation.";

    println!("Task: {}\n", input);
    println!("This demonstration shows how different patterns would be used.");
    println!("In a real implementation, you'd see different behaviors:\n");

    // Pattern descriptions and use cases
    let patterns = vec![
        (
            "Sequential",
            Pattern::Sequential,
            "Use when: Each agent builds on the previous agent's work (e.g., draft → review → refine)",
            "Behavior: Agent A outputs to Agent B outputs to Agent C",
        ),
        (
            "Concurrent (Vote)",
            Pattern::Concurrent { aggregation: Aggregation::Vote },
            "Use when: You want multiple perspectives and majority consensus",
            "Behavior: All agents get the same input, results are voted on",
        ),
        (
            "Concurrent (Consensus)",
            Pattern::Concurrent { aggregation: Aggregation::Consensus },
            "Use when: You want an LLM to reconcile different expert opinions",
            "Behavior: All agents respond, then consensus is built via LLM",
        ),
        (
            "Concurrent (Combine)",
            Pattern::Concurrent { aggregation: Aggregation::Combine },
            "Use when: You want to see all perspectives without choosing",
            "Behavior: All agent outputs are concatenated together",
        ),
        (
            "GroupChat",
            Pattern::GroupChat { max_rounds: 5 },
            "Use when: Agents should debate and refine ideas through conversation",
            "Behavior: Agents take turns responding to each other's messages",
        ),
        (
            "Handoff",
            Pattern::Handoff { max_hops: 5 },
            "Use when: Agents should route tasks to the most qualified specialist",
            "Behavior: Each agent can handle or pass to another agent",
        ),
        (
            "Magentic",
            Pattern::Magentic { max_iterations: 10 },
            "Use when: Complex task needs to be broken down and coordinated",
            "Behavior: Manager creates task list, workers execute, manager synthesizes",
        ),
    ];

    for (name, pattern, use_case, behavior) in patterns {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Pattern: {}", name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Use Case: {}", use_case);
        println!("Behavior: {}", behavior);
        println!();

        /*
        // In a real implementation with API credentials:
        let agents = create_agents(); // Your agent creation logic
        let start = Instant::now();

        let result = Orchestrator::new(agents)
            .pattern(pattern)
            .build()?
            .execute(input)
            .await?;

        let duration = start.elapsed();

        println!("Duration: {:?}", duration);
        println!("\nOutput:\n{}", result.output);
        println!("\nMetadata:");
        for (key, value) in result.pattern_metadata.details {
            println!("  {}: {}", key, value);
        }
        println!("\nExecution trace:");
        for entry in result.pattern_metadata.trace.iter().take(5) {
            println!("  {}", entry);
        }
        if result.pattern_metadata.trace.len() > 5 {
            println!("  ... ({} more entries)", result.pattern_metadata.trace.len() - 5);
        }
        */

        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Pattern Selection Guide");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("Sequential: Best for linear workflows (draft → edit → finalize)");
    println!("Concurrent: Best for gathering multiple perspectives");
    println!("GroupChat: Best for collaborative refinement through dialogue");
    println!("Handoff: Best for routing to specialists based on task type");
    println!("Magentic: Best for complex, multi-step projects");
    println!();
    println!("Pro tip: Start with Sequential, then experiment with others!");

    Ok(())
}

/*
// Example agent creation helper (requires real models)
fn create_agents() -> Vec<Agent> {
    let client = rig::providers::openai::Client::new("api-key");
    let model = client.completion_model("gpt-4");

    vec![
        Agent::new(
            "architect",
            model.clone(),
            "You are a software architect specializing in system design."
        ),
        Agent::new(
            "devops",
            model.clone(),
            "You are a DevOps engineer focused on operational concerns."
        ),
        Agent::new(
            "security",
            model.clone(),
            "You are a security expert focused on system security."
        ),
    ]
}
*/
