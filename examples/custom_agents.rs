//! Custom agents example
//!
//! This example demonstrates creating agents with different LLM providers
//! (OpenAI, Anthropic, Cohere) and using them together in orchestration patterns.
//!
//! Note: This example requires API credentials for the providers you want to use.

use anyhow::Result;
use rig_patterns::{Agent, Orchestrator, Pattern};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("=== rig-patterns Custom Agents Example ===\n");
    println!("This example shows how to mix agents from different providers.\n");

    // The key insight: rig-patterns works with ANY rig-core CompletionModel
    // You can mix and match providers in the same orchestration!

    println!("Example 1: Multi-Provider Setup");
    println!("────────────────────────────────");
    println!();

    /*
    // Initialize different provider clients
    let openai_client = rig::providers::openai::Client::new(
        std::env::var("OPENAI_API_KEY")
            .expect("OPENAI_API_KEY environment variable not set")
    );

    let anthropic_client = rig::providers::anthropic::Client::new(
        std::env::var("ANTHROPIC_API_KEY")
            .expect("ANTHROPIC_API_KEY environment variable not set")
    );

    // Create agents using different providers
    let agents = vec![
        Agent::new(
            "gpt4-analyst",
            openai_client.completion_model("gpt-4"),
            "You are a data analyst who excels at finding patterns in data."
        ),
        Agent::new(
            "claude-writer",
            anthropic_client.completion_model("claude-3-opus-20240229"),
            "You are a technical writer who excels at clear documentation."
        ),
        Agent::new(
            "gpt4-reviewer",
            openai_client.completion_model("gpt-4"),
            "You are a code reviewer who provides constructive feedback."
        ),
    ];

    println!("Created {} agents from different providers:", agents.len());
    for agent in &agents {
        println!("  - {} ({})", agent.id(), agent.system_prompt());
    }
    println!();

    // Use them in a Sequential pattern
    let input = "Analyze this dataset and create documentation for the findings.";

    let result = Orchestrator::new(agents)
        .pattern(Pattern::Sequential)
        .build()?
        .execute(input)
        .await?;

    println!("Result: {}", result.output);
    */

    println!("Example setup shown above demonstrates:");
    println!("✓ Multiple provider clients (OpenAI, Anthropic)");
    println!("✓ Agents with different models from different providers");
    println!("✓ All working together in the same orchestration\n");

    println!("Example 2: Provider-Specific Strengths");
    println!("──────────────────────────────────────");
    println!();
    println!("You might choose different providers for different roles:");
    println!();
    println!("GPT-4 (OpenAI):");
    println!("  - Strong at: Code generation, analysis, structured output");
    println!("  - Good for: Technical tasks, data processing");
    println!();
    println!("Claude (Anthropic):");
    println!("  - Strong at: Long context, nuanced writing, reasoning");
    println!("  - Good for: Documentation, complex analysis, creative tasks");
    println!();
    println!("Example orchestration leveraging strengths:");
    println!("  1. GPT-4: Analyze code structure");
    println!("  2. Claude: Write comprehensive documentation");
    println!("  3. GPT-4: Review for technical accuracy\n");

    println!("Example 3: Same Provider, Different Models");
    println!("──────────────────────────────────────────");
    println!();

    /*
    // You can also use different models from the same provider
    let agents = vec![
        Agent::new(
            "gpt4-expert",
            openai_client.completion_model("gpt-4"),
            "You are an expert analyst."
        ),
        Agent::new(
            "gpt35-assistant",
            openai_client.completion_model("gpt-3.5-turbo"),
            "You are a helpful assistant."
        ),
    ];

    // Use in a Concurrent pattern to compare model outputs
    let result = Orchestrator::new(agents)
        .pattern(Pattern::Concurrent {
            aggregation: Aggregation::Combine
        })
        .build()?
        .execute("Explain quantum computing.")
        .await?;
    */

    println!("Use case: Compare outputs from different model versions");
    println!("Pattern: Concurrent with Combine aggregation");
    println!("Benefit: See how different models approach the same task\n");

    println!("Example 4: Environment-Based Configuration");
    println!("──────────────────────────────────────────");
    println!();

    /*
    // Helper function to create agents based on environment
    fn create_agent_pool() -> Result<Vec<Agent>> {
        let mut agents = Vec::new();

        // Add OpenAI agents if API key is available
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            let client = rig::providers::openai::Client::new(api_key);
            agents.push(Agent::new(
                "openai-agent",
                client.completion_model("gpt-4"),
                "OpenAI-powered agent"
            ));
        }

        // Add Anthropic agents if API key is available
        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            let client = rig::providers::anthropic::Client::new(api_key);
            agents.push(Agent::new(
                "anthropic-agent",
                client.completion_model("claude-3-opus-20240229"),
                "Anthropic-powered agent"
            ));
        }

        if agents.is_empty() {
            anyhow::bail!("No API keys found. Set OPENAI_API_KEY or ANTHROPIC_API_KEY");
        }

        Ok(agents)
    }

    let agents = create_agent_pool()?;
    println!("Created {} agents based on available credentials", agents.len());
    */

    println!("Pattern: Dynamically create agents based on available API keys");
    println!("Benefit: Gracefully handle different deployment environments\n");

    println!("Example 5: Specialized Agent Teams");
    println!("──────────────────────────────────");
    println!();

    /*
    // Create specialized teams for different tasks
    fn create_research_team(client: &rig::providers::openai::Client) -> Vec<Agent> {
        vec![
            Agent::new(
                "researcher",
                client.completion_model("gpt-4"),
                "You research topics thoroughly."
            ),
            Agent::new(
                "fact-checker",
                client.completion_model("gpt-4"),
                "You verify facts and sources."
            ),
            Agent::new(
                "synthesizer",
                client.completion_model("gpt-4"),
                "You synthesize research into clear summaries."
            ),
        ]
    }

    fn create_writing_team(
        openai: &rig::providers::openai::Client,
        anthropic: &rig::providers::anthropic::Client
    ) -> Vec<Agent> {
        vec![
            Agent::new(
                "outliner",
                openai.completion_model("gpt-4"),
                "You create document outlines."
            ),
            Agent::new(
                "writer",
                anthropic.completion_model("claude-3-opus-20240229"),
                "You write compelling content."
            ),
            Agent::new(
                "editor",
                openai.completion_model("gpt-4"),
                "You edit for clarity and correctness."
            ),
        ]
    }

    // Use different teams for different tasks
    let research_result = Orchestrator::new(create_research_team(&openai_client))
        .pattern(Pattern::Sequential)
        .build()?
        .execute("Research quantum computing applications")
        .await?;

    let writing_result = Orchestrator::new(create_writing_team(&openai_client, &anthropic_client))
        .pattern(Pattern::Sequential)
        .build()?
        .execute(&research_result.output)
        .await?;
    */

    println!("Pattern: Create specialized agent teams for different phases");
    println!("Example: Research team → Writing team → Review team");
    println!("Benefit: Optimize model choice for each task type\n");

    println!("═══════════════════════════════════════════════════");
    println!("Key Takeaways");
    println!("═══════════════════════════════════════════════════");
    println!("✓ Mix providers freely - rig-patterns is provider-agnostic");
    println!("✓ Choose models based on their strengths for each role");
    println!("✓ Same agent interface works across all providers");
    println!("✓ Switch providers without changing orchestration code");

    Ok(())
}
