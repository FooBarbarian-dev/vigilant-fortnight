//! Basic usage example demonstrating pattern switching
//!
//! This example shows how to create agents and switch between different
//! orchestration patterns with minimal code changes.
//!
//! Note: This example requires API credentials and won't run as-is.
//! It's meant to demonstrate the API usage pattern.

use anyhow::Result;
use rig_patterns::{Agent, Aggregation, Orchestrator, Pattern};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for debugging
    tracing_subscriber::fmt::init();

    println!("=== rig-patterns Basic Usage Example ===\n");

    // NOTE: In a real application, you would initialize your LLM clients here
    // For example:
    // let openai_client = rig::providers::openai::Client::new("your-api-key");
    // let model = openai_client.completion_model("gpt-4");

    println!("This is a demonstration of the rig-patterns API.");
    println!("To run this example with real LLMs, you need to:");
    println!("1. Set up your LLM provider credentials");
    println!("2. Initialize completion models");
    println!("3. Create agents with those models\n");

    // Example API usage (commented out since we don't have real models):
    /*
    // Create agents with different roles
    let agents = vec![
        Agent::new(
            "summarizer",
            model.clone(),
            "You are a helpful assistant that summarizes text concisely."
        ),
        Agent::new(
            "critic",
            model.clone(),
            "You are a critic who evaluates and provides constructive feedback."
        ),
        Agent::new(
            "refiner",
            model.clone(),
            "You take feedback and create improved versions."
        ),
    ];

    let input = "Explain the concept of async programming in Rust.";

    // Pattern 1: Sequential
    println!("--- Pattern 1: Sequential ---");
    let result = Orchestrator::new(agents.clone())
        .pattern(Pattern::Sequential)
        .build()?
        .execute(input)
        .await?;

    println!("Output: {}\n", result.output);
    println!("Trace:");
    for entry in result.pattern_metadata.trace {
        println!("  {}", entry);
    }

    // Pattern 2: Concurrent with Vote aggregation
    println!("\n--- Pattern 2: Concurrent (Vote) ---");
    let result = Orchestrator::new(agents.clone())
        .pattern(Pattern::Concurrent {
            aggregation: Aggregation::Vote,
        })
        .build()?
        .execute(input)
        .await?;

    println!("Output: {}\n", result.output);

    // Pattern 3: Group Chat
    println!("\n--- Pattern 3: Group Chat ---");
    let result = Orchestrator::new(agents.clone())
        .pattern(Pattern::GroupChat { max_rounds: 3 })
        .build()?
        .execute(input)
        .await?;

    println!("Output: {}\n", result.output);

    // Pattern 4: Handoff
    println!("\n--- Pattern 4: Handoff ---");
    let result = Orchestrator::new(agents.clone())
        .pattern(Pattern::Handoff { max_hops: 5 })
        .build()?
        .execute(input)
        .await?;

    println!("Output: {}\n", result.output);

    // Pattern 5: Magentic (requires at least 2 agents)
    println!("\n--- Pattern 5: Magentic ---");
    let result = Orchestrator::new(agents.clone())
        .pattern(Pattern::Magentic { max_iterations: 3 })
        .build()?
        .execute(input)
        .await?;

    println!("Output: {}\n", result.output);
    */

    println!("\n=== Key Takeaway ===");
    println!("Notice how the same agents work with ALL patterns!");
    println!("Just change the `.pattern()` call to switch orchestration strategies.");

    Ok(())
}
