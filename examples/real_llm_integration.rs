//! Real LLM Provider Integration Example
//!
//! This example demonstrates how to use rig-patterns with actual LLM providers
//! (OpenAI, Anthropic, Cohere) using your API keys.
//!
//! ## Setup
//!
//! 1. Set your API keys as environment variables:
//!    ```bash
//!    export OPENAI_API_KEY="sk-..."
//!    export ANTHROPIC_API_KEY="sk-ant-..."
//!    export COHERE_API_KEY="..."
//!    ```
//!
//! 2. Run the example:
//!    ```bash
//!    cargo run --example real_llm_integration
//!    ```
//!
//! ## Supported Providers & Models
//!
//! ### OpenAI
//! - gpt-4 (most capable)
//! - gpt-4-turbo-preview (faster, cheaper)
//! - gpt-3.5-turbo (fastest, cheapest)
//! - Get your API key: https://platform.openai.com/api-keys
//!
//! ### Anthropic (Claude)
//! - claude-3-opus-20240229 (most capable)
//! - claude-3-sonnet-20240229 (balanced)
//! - claude-3-haiku-20240307 (fastest, cheapest)
//! - Get your API key: https://console.anthropic.com/
//!
//! ### Cohere
//! - command (most capable)
//! - command-light (faster, cheaper)
//! - command-nightly (experimental features)
//! - Get your API key: https://dashboard.cohere.com/api-keys

use anyhow::Result;
use rig_patterns::{Agent, Aggregation, Orchestrator, Pattern};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for debugging
    tracing_subscriber::fmt::init();

    println!("=== rig-patterns Real LLM Integration ===\n");

    // Example 1: Single provider (OpenAI)
    println!("--- Example 1: OpenAI Only ---");
    example_openai_only().await?;

    // Example 2: Multi-provider setup
    println!("\n--- Example 2: Multi-Provider ---");
    example_multi_provider().await?;

    // Example 3: Using from_env helper
    println!("\n--- Example 3: Environment Variables ---");
    example_from_env().await?;

    // Example 4: Pattern switching with real LLMs
    println!("\n--- Example 4: Pattern Switching ---");
    example_pattern_switching().await?;

    Ok(())
}

/// Example 1: Use OpenAI models only
async fn example_openai_only() -> Result<()> {
    // Check if API key is set
    let api_key = match std::env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("⚠️  OPENAI_API_KEY not set. Skipping this example.");
            println!("   Set it with: export OPENAI_API_KEY=\"sk-...\"");
            return Ok(());
        }
    };

    println!("Creating OpenAI agents...");

    // Create agents using different OpenAI models
    let agents = vec![
        Agent::from_openai(
            "gpt4-writer",
            &api_key,
            "gpt-4",
            "You write clear, concise content.",
        )?,
        Agent::from_openai(
            "gpt35-reviewer",
            &api_key,
            "gpt-3.5-turbo",
            "You review content for clarity and correctness.",
        )?,
    ];

    // Execute with Sequential pattern
    let orchestrator = Orchestrator::new(agents).pattern(Pattern::Sequential).build()?;

    println!("Executing Sequential pattern...");
    let result = orchestrator
        .execute("Write a one-sentence explanation of Rust's ownership system.")
        .await?;

    println!("\n✅ Result:");
    println!("{}", result.output);
    println!("\n📊 Metadata: {} steps completed", result.pattern_metadata.trace.len());

    Ok(())
}

/// Example 2: Mix providers (OpenAI + Anthropic + Cohere)
async fn example_multi_provider() -> Result<()> {
    println!("Creating agents from multiple providers...");

    let mut agents = Vec::new();

    // Add OpenAI agent if available
    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        agents.push(Agent::from_openai(
            "openai-analyst",
            &api_key,
            "gpt-4",
            "You analyze code for potential issues.",
        )?);
        println!("✓ Added OpenAI agent");
    }

    // Add Anthropic agent if available
    if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
        agents.push(Agent::from_anthropic(
            "claude-writer",
            &api_key,
            "claude-3-sonnet-20240229",
            "You write comprehensive documentation.",
        )?);
        println!("✓ Added Anthropic agent");
    }

    // Add Cohere agent if available
    if let Ok(api_key) = std::env::var("COHERE_API_KEY") {
        agents.push(Agent::from_cohere(
            "cohere-summarizer",
            &api_key,
            "command",
            "You create concise summaries.",
        )?);
        println!("✓ Added Cohere agent");
    }

    if agents.is_empty() {
        println!("⚠️  No API keys found. Set at least one:");
        println!("   export OPENAI_API_KEY=\"sk-...\"");
        println!("   export ANTHROPIC_API_KEY=\"sk-ant-...\"");
        println!("   export COHERE_API_KEY=\"...\"");
        return Ok(());
    }

    println!("\nUsing {} agents from different providers", agents.len());

    // Use Concurrent pattern to get multiple perspectives
    let orchestrator = Orchestrator::new(agents)
        .pattern(Pattern::Concurrent {
            aggregation: Aggregation::Combine,
        })
        .build()?;

    println!("Executing Concurrent pattern...");
    let result = orchestrator
        .execute("Explain the benefits of async programming in 2 sentences.")
        .await?;

    println!("\n✅ Result (combined from all providers):");
    println!("{}", result.output);

    Ok(())
}

/// Example 3: Using the from_env helper
async fn example_from_env() -> Result<()> {
    println!("Using Agent::from_env helper...");

    // Try to create agents from environment variables
    let mut agents = Vec::new();

    // Try each provider
    for (provider, model) in [
        ("openai", "gpt-4"),
        ("anthropic", "claude-3-haiku-20240307"),
        ("cohere", "command-light"),
    ] {
        match Agent::from_env(
            &format!("{}-agent", provider),
            provider,
            model,
            &format!("You are a helpful {} agent.", provider),
        ) {
            Ok(agent) => {
                agents.push(agent);
                println!("✓ Created {} agent using {}", provider, model);
            }
            Err(e) => {
                println!("ℹ️  Skipping {} ({})", provider, e);
            }
        }
    }

    if agents.is_empty() {
        println!("\n⚠️  No agents could be created. Set API keys:");
        println!("   export OPENAI_API_KEY=\"sk-...\"");
        println!("   export ANTHROPIC_API_KEY=\"sk-ant-...\"");
        println!("   export COHERE_API_KEY=\"...\"");
        return Ok(());
    }

    println!("\nCreated {} agents total", agents.len());

    // Quick test with first agent
    let test_result = agents[0].prompt("Say hello!").await?;
    println!("\n✅ Test response from {}:", agents[0].id());
    println!("{}", test_result);

    Ok(())
}

/// Example 4: Pattern switching with real LLMs
async fn example_pattern_switching() -> Result<()> {
    // Check for at least one API key
    let api_key = std::env::var("OPENAI_API_KEY")
        .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
        .or_else(|_| std::env::var("COHERE_API_KEY"));

    let api_key = match api_key {
        Ok(key) => key,
        Err(_) => {
            println!("⚠️  No API keys found. Set at least one API key to run this example.");
            return Ok(());
        }
    };

    println!("Demonstrating pattern switching...");

    // Determine provider from which key is available
    let (provider, model) = if std::env::var("OPENAI_API_KEY").is_ok() {
        ("openai", "gpt-3.5-turbo")
    } else if std::env::var("ANTHROPIC_API_KEY").is_ok() {
        ("anthropic", "claude-3-haiku-20240307")
    } else {
        ("cohere", "command-light")
    };

    // Create agents using from_env
    let agents = vec![
        Agent::from_env("summarizer", provider, model, "You summarize text concisely.")?,
        Agent::from_env("critic", provider, model, "You provide constructive feedback.")?,
    ];

    let input = "Explain why testing is important in software development.";
    println!("\nInput: {}", input);

    // Pattern 1: Sequential
    println!("\n1️⃣  Sequential Pattern");
    let result = Orchestrator::new(agents.clone())
        .pattern(Pattern::Sequential)
        .build()?
        .execute(input)
        .await?;
    println!("Output: {}", result.output.chars().take(200).collect::<String>());

    // Pattern 2: Concurrent
    println!("\n2️⃣  Concurrent Pattern (Vote)");
    let result = Orchestrator::new(agents.clone())
        .pattern(Pattern::Concurrent {
            aggregation: Aggregation::Vote,
        })
        .build()?
        .execute(input)
        .await?;
    println!("Output: {}", result.output.chars().take(200).collect::<String>());

    println!("\n✅ Successfully switched patterns with real LLMs!");
    println!("💡 Try changing the pattern enum to experiment!");

    Ok(())
}

/// Helper: Print available providers
#[allow(dead_code)]
fn print_available_providers() {
    println!("=== Available LLM Providers ===\n");

    println!("OpenAI:");
    println!("  Models: gpt-4, gpt-4-turbo-preview, gpt-3.5-turbo");
    println!("  API Key: OPENAI_API_KEY");
    println!("  Get key: https://platform.openai.com/api-keys\n");

    println!("Anthropic (Claude):");
    println!("  Models: claude-3-opus-20240229, claude-3-sonnet-20240229, claude-3-haiku-20240307");
    println!("  API Key: ANTHROPIC_API_KEY");
    println!("  Get key: https://console.anthropic.com/\n");

    println!("Cohere:");
    println!("  Models: command, command-light, command-nightly");
    println!("  API Key: COHERE_API_KEY");
    println!("  Get key: https://dashboard.cohere.com/api-keys\n");
}
