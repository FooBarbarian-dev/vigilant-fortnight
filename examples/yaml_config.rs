//! YAML configuration example
//!
//! This example demonstrates loading orchestrator configuration from YAML files,
//! making it easy to switch patterns without changing code.
//!
//! Note: This example requires API credentials and the 'yaml' feature enabled.
//! Run with: cargo run --example yaml_config --features yaml

use anyhow::Result;
use rig_patterns::config::OrchestratorConfig;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("=== rig-patterns YAML Config Example ===\n");

    // Example 1: Load and validate a config file
    println!("--- Example 1: Load Config ---");

    let yaml = r#"
agents:
  - id: summarizer
    provider: openai
    model: gpt-4
    system_prompt: "You are a helpful assistant that summarizes text concisely."
  - id: critic
    provider: openai
    model: gpt-4
    system_prompt: "You are a critic who evaluates and provides feedback."

orchestration:
  type: sequential
"#;

    let config = OrchestratorConfig::from_yaml(yaml)?;
    println!("Loaded config with {} agents", config.agents.len());
    println!("Pattern: {:?}", config.pattern());

    // Validate the configuration
    config.validate()?;
    println!("✓ Configuration is valid\n");

    // Example 2: Different patterns in YAML
    println!("--- Example 2: Concurrent Pattern ---");

    let concurrent_yaml = r#"
agents:
  - id: agent1
    provider: openai
    model: gpt-4
    system_prompt: "Agent 1"
  - id: agent2
    provider: openai
    model: gpt-4
    system_prompt: "Agent 2"

orchestration:
  type: concurrent
  aggregation: vote
"#;

    let config = OrchestratorConfig::from_yaml(concurrent_yaml)?;
    println!("Pattern: {:?}\n", config.pattern());

    // Example 3: Group Chat pattern
    println!("--- Example 3: Group Chat Pattern ---");

    let group_chat_yaml = r#"
agents:
  - id: moderator
    provider: anthropic
    model: claude-3-opus
    system_prompt: "You moderate discussions"
  - id: analyst
    provider: openai
    model: gpt-4
    system_prompt: "You analyze data"
  - id: writer
    provider: openai
    model: gpt-4
    system_prompt: "You write reports"

orchestration:
  type: group_chat
  max_rounds: 5
"#;

    let config = OrchestratorConfig::from_yaml(group_chat_yaml)?;
    println!("Pattern: {:?}", config.pattern());
    println!("Agents: {}", config.agents.len());
    config.validate()?;
    println!("✓ Configuration is valid\n");

    // Example 4: Magentic pattern
    println!("--- Example 4: Magentic Pattern ---");

    let magentic_yaml = r#"
agents:
  - id: manager
    provider: openai
    model: gpt-4
    system_prompt: "You are a project manager who breaks down tasks"
  - id: researcher
    provider: openai
    model: gpt-4
    system_prompt: "You research topics"
  - id: writer
    provider: openai
    model: gpt-4
    system_prompt: "You write content"

orchestration:
  type: magentic
  max_iterations: 10
"#;

    let config = OrchestratorConfig::from_yaml(magentic_yaml)?;
    println!("Pattern: {:?}", config.pattern());
    config.validate()?;
    println!("✓ Configuration is valid\n");

    // Example 5: Save config to file
    println!("--- Example 5: Save Config ---");

    let save_path = "/tmp/rig-patterns-config.yaml";
    config.save(save_path)?;
    println!("Saved config to: {}", save_path);

    // Load it back
    let loaded = OrchestratorConfig::from_file(save_path)?;
    println!("Loaded back: {} agents", loaded.agents.len());

    // Example 6: Build orchestrator from config
    println!("\n--- Example 6: Build Orchestrator from Config ---");
    println!("To build an orchestrator from config, you would:");
    println!("1. Load the config file");
    println!("2. Initialize LLM clients based on provider/model");
    println!("3. Create Agent instances");
    println!("4. Build Orchestrator with the configured pattern");

    /*
    // Example code (requires real API clients):
    let config = OrchestratorConfig::from_file("config.yaml")?;

    let mut agents = Vec::new();
    for agent_config in config.agents {
        // Initialize client based on provider
        let client = match agent_config.provider.as_str() {
            "openai" => create_openai_client(),
            "anthropic" => create_anthropic_client(),
            _ => anyhow::bail!("Unknown provider: {}", agent_config.provider),
        };

        let model = client.completion_model(&agent_config.model);
        let agent = Agent::new(&agent_config.id, model, &agent_config.system_prompt);
        agents.push(agent);
    }

    let orchestrator = Orchestrator::new(agents)
        .pattern(config.pattern().clone())
        .build()?;

    let result = orchestrator.execute("Your input here").await?;
    */

    println!("\n=== Key Takeaway ===");
    println!("YAML configs let you switch patterns by editing a file,");
    println!("without touching your code. Perfect for experimentation!");

    Ok(())
}
