# rig-patterns

**Dead-simple orchestration pattern abstractions over the rig-core LLM framework**

[![Crates.io](https://img.shields.io/crates/v/rig-patterns.svg)](https://crates.io/crates/rig-patterns)
[![Documentation](https://docs.rs/rig-patterns/badge.svg)](https://docs.rs/rig-patterns)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## Overview

`rig-patterns` makes it trivial to switch between different LLM orchestration patterns. Change how your agents collaborate by modifying **a single line of code** instead of rewriting your application logic.

### The Problem

Building multi-agent LLM systems requires choosing an orchestration pattern:
- Should agents run sequentially, each building on the previous output?
- Should they run concurrently and vote on the best answer?
- Should they debate in a group chat until reaching consensus?
- Should agents hand off tasks to specialists?
- Should a manager coordinate workers on a complex task?

Switching between these patterns typically means significant code rewrites.

### The Solution

```rust
// Create your agents once
let agents = vec![
    Agent::new("summarizer", model, "You summarize text"),
    Agent::new("critic", model, "You critique summaries"),
];

// Switch patterns by changing ONE line:
let orchestrator = Orchestrator::new(agents)
    .pattern(Pattern::Sequential)  // ← Just change this!
    // .pattern(Pattern::Concurrent { aggregation: Aggregation::Vote })
    // .pattern(Pattern::GroupChat { max_rounds: 5 })
    .build()?;

let result = orchestrator.execute("Your input here").await?;
```

## Getting Started

This repository contains both the core `rig-patterns` library and an interactive web UI for demonstrations.

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- API keys for LLM providers (optional for UI demo, required for real usage)

### Building Everything

```bash
# Clone the repository
git clone https://github.com/yourusername/vigilant-fortnight.git
cd vigilant-fortnight

# Build the library
cargo build --release

# Build the UI application
cd rig-patterns-ui
cargo build --release
cd ..
```

### Running the Examples

The library includes several examples demonstrating different patterns:

```bash
# Run basic usage example
cargo run --example basic_usage

# Run pattern comparison (shows all 5 patterns)
cargo run --example pattern_comparison

# Run custom agents example
cargo run --example custom_agents

# Run real LLM integration (requires API keys)
cargo run --example real_llm_integration
```

### Running the Interactive UI

The UI provides a visual demonstration of all orchestration patterns with real-time execution visualization:

```bash
# From the root directory
cd rig-patterns-ui
cargo run --release

# Or run directly from root
cargo run --release --bin rig-patterns-ui
```

Then open your browser to `http://localhost:3000`

**Features:**
- Configure multiple agents with different roles
- Switch between all 5 orchestration patterns
- Real-time visualization of agent execution
- Compare pattern performance side-by-side
- Load preset configurations

### Setting Up API Keys (Optional)

For real LLM integration, create a `.env` file in the project root:

```bash
cp .env.example .env
# Edit .env and add your API keys
```

Example `.env`:
```bash
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
COHERE_API_KEY=...
```

The examples and UI will use mock data if API keys are not configured.

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
rig-patterns = "0.1"
rig-core = "0.4"
```

### Basic Example

```rust
use rig_patterns::{Agent, Orchestrator, Pattern};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize your LLM client (example with OpenAI)
    let client = rig::providers::openai::Client::new("your-api-key");
    let model = client.completion_model("gpt-4");

    // Create agents with different roles
    let agents = vec![
        Agent::new(
            "writer",
            model.clone(),
            "You write clear, concise content."
        ),
        Agent::new(
            "editor",
            model.clone(),
            "You edit content for clarity and correctness."
        ),
    ];

    // Build and execute with Sequential pattern
    let result = Orchestrator::new(agents)
        .pattern(Pattern::Sequential)
        .build()?
        .execute("Write a haiku about Rust programming")
        .await?;

    println!("Output: {}", result.output);
    Ok(())
}
```

## Connecting to LLM Providers

`rig-patterns` supports all major LLM providers through convenient helper methods. You can mix and match providers in the same orchestration!

### Supported Providers

| Provider | Models | API Key Env Var | Get API Key |
|----------|--------|-----------------|-------------|
| **OpenAI** | gpt-4, gpt-4-turbo-preview, gpt-3.5-turbo | `OPENAI_API_KEY` | [platform.openai.com/api-keys](https://platform.openai.com/api-keys) |
| **Anthropic** | claude-3-opus, claude-3-sonnet, claude-3-haiku | `ANTHROPIC_API_KEY` | [console.anthropic.com](https://console.anthropic.com/) |
| **Cohere** | command, command-light, command-nightly | `COHERE_API_KEY` | [dashboard.cohere.com/api-keys](https://dashboard.cohere.com/api-keys) |

### Setup API Keys

1. **Copy the example environment file:**
   ```bash
   cp .env.example .env
   ```

2. **Add your API keys to `.env`:**
   ```bash
   OPENAI_API_KEY=sk-...
   ANTHROPIC_API_KEY=sk-ant-...
   COHERE_API_KEY=...
   ```

3. **Load environment variables:**
   ```rust
   // In your code, or use a crate like `dotenv`
   dotenv::dotenv().ok();
   ```

### Creating Agents from Providers

#### Method 1: Direct API Key

```rust
use rig_patterns::Agent;

// OpenAI
let agent = Agent::from_openai(
    "writer",
    "sk-...",  // Your API key
    "gpt-4",
    "You are a helpful writer"
)?;

// Anthropic
let agent = Agent::from_anthropic(
    "analyst",
    "sk-ant-...",
    "claude-3-opus-20240229",
    "You are a thorough analyst"
)?;

// Cohere
let agent = Agent::from_cohere(
    "summarizer",
    "...",
    "command",
    "You create concise summaries"
)?;
```

#### Method 2: From Environment Variables (Recommended)

```rust
use rig_patterns::Agent;

// Reads from OPENAI_API_KEY environment variable
let agent = Agent::from_env(
    "writer",
    "openai",
    "gpt-4",
    "You are a helpful writer"
)?;

// Reads from ANTHROPIC_API_KEY
let agent = Agent::from_env(
    "analyst",
    "anthropic",
    "claude-3-sonnet-20240229",
    "You analyze thoroughly"
)?;
```

### Multi-Provider Example

Mix providers in the same orchestration:

```rust
use rig_patterns::{Agent, Orchestrator, Pattern, Aggregation};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create agents from different providers
    let agents = vec![
        Agent::from_env("gpt4-analyst", "openai", "gpt-4", "You analyze code")?,
        Agent::from_env("claude-writer", "anthropic", "claude-3-sonnet-20240229", "You write docs")?,
        Agent::from_env("cohere-reviewer", "cohere", "command", "You review content")?,
    ];

    // Run concurrent execution with different models!
    let result = Orchestrator::new(agents)
        .pattern(Pattern::Concurrent {
            aggregation: Aggregation::Combine
        })
        .build()?
        .execute("Analyze this code and document it")
        .await?;

    println!("{}", result.output);
    Ok(())
}
```

### Model Recommendations

**For Speed & Cost:**
- OpenAI: `gpt-3.5-turbo`
- Anthropic: `claude-3-haiku-20240307`
- Cohere: `command-light`

**For Quality:**
- OpenAI: `gpt-4`
- Anthropic: `claude-3-opus-20240229`
- Cohere: `command`

**Balanced:**
- OpenAI: `gpt-4-turbo-preview`
- Anthropic: `claude-3-sonnet-20240229`
- Cohere: `command`

### Complete Integration Example

See [`examples/real_llm_integration.rs`](examples/real_llm_integration.rs) for a comprehensive example showing:
- Single provider setup
- Multi-provider orchestration
- Environment variable usage
- Pattern switching with real LLMs

Run it with:
```bash
export OPENAI_API_KEY="sk-..."
cargo run --example real_llm_integration
```

## Available Patterns

### 1. Sequential
**When to use:** Linear workflows where each step builds on the previous one

```rust
Pattern::Sequential
```

**Example:** Draft → Review → Refine

### 2. Concurrent
**When to use:** You want multiple perspectives on the same input

```rust
Pattern::Concurrent {
    aggregation: Aggregation::Vote  // or Consensus, Combine
}
```

**Aggregation strategies:**
- `Vote`: Simple majority on similar outputs
- `Consensus`: LLM reconciles differences (simplified)
- `Combine`: Concatenate all outputs

**Example:** Multiple experts analyze a problem, results are voted on

### 3. GroupChat
**When to use:** Agents should debate and refine ideas through conversation

```rust
Pattern::GroupChat { max_rounds: 5 }
```

**Example:** Agents discuss a design decision until reaching agreement

### 4. Handoff
**When to use:** Route tasks to the most qualified specialist

```rust
Pattern::Handoff { max_hops: 10 }
```

**Example:** Triage agent → Specialist agent → Review agent

### 5. Magentic
**When to use:** Complex tasks that need to be broken down and coordinated

```rust
Pattern::Magentic { max_iterations: 10 }
```

**Example:** Manager breaks down "build a website" into subtasks, coordinates workers

## Pattern Selection Guide

| Pattern | Best For | Agent Interaction | Stop Condition |
|---------|----------|-------------------|----------------|
| **Sequential** | Linear pipelines | A → B → C | All agents complete |
| **Concurrent** | Multiple perspectives | All get same input | All agents complete |
| **GroupChat** | Collaborative refinement | Round-robin conversation | Consensus or max rounds |
| **Handoff** | Task routing | Explicit handoffs | Task handled or max hops |
| **Magentic** | Complex projects | Manager coordinates workers | Tasks complete or max iterations |

**Quick recommendation:** Start with `Sequential`, then experiment!

## YAML Configuration (Optional)

Enable the `yaml` feature to configure orchestrators via YAML:

```toml
[dependencies]
rig-patterns = { version = "0.1", features = ["yaml"] }
```

Example config:

```yaml
agents:
  - id: summarizer
    provider: openai
    model: gpt-4
    system_prompt: "You summarize text concisely"

orchestration:
  type: concurrent
  aggregation: vote
```

Load and use:

```rust
use rig_patterns::config::OrchestratorConfig;

let config = OrchestratorConfig::from_file("config.yaml")?;
// Create agents based on config, then build orchestrator
```

See [`examples/yaml_config.rs`](examples/yaml_config.rs) for complete example.

## Examples

Run the examples to see patterns in action:

```bash
# Basic usage demonstrating pattern switching
cargo run --example basic_usage

# YAML configuration (requires 'yaml' feature)
cargo run --example yaml_config --features yaml

# Compare all patterns
cargo run --example pattern_comparison

# Multi-provider agent setup
cargo run --example custom_agents
```

## rig-patterns vs. Raw rig-core

| Feature | rig-patterns | Raw rig-core |
|---------|--------------|--------------|
| **Pattern switching** | Change one line | Rewrite orchestration |
| **Complexity** | High-level abstractions | Low-level primitives |
| **Flexibility** | Fixed patterns | Full customization |
| **Streaming** | Not supported | Fully supported |
| **Tool calling** | Not supported | Fully supported |
| **Setup time** | Minutes | Hours to days |
| **Best for** | Rapid experimentation | Production systems |

### When to Use rig-patterns

✅ Experimenting with different orchestration strategies
✅ Prototyping multi-agent applications
✅ Learning about agent patterns
✅ Simple string-based workflows
✅ You prioritize development speed

### When to Use Raw rig-core

✅ You need streaming responses
✅ Agents must call external tools
✅ You need structured data (not just strings)
✅ You need fine-grained control
✅ You're building production systems

## Migration Path

Start with `rig-patterns` for rapid prototyping:

```rust
// Phase 1: Prototype with rig-patterns
let result = Orchestrator::new(agents)
    .pattern(Pattern::Sequential)
    .build()?
    .execute(input)
    .await?;
```

When you need more control, drop down to `rig-core`:

```rust
// Phase 2: Migrate to rig-core for production
let agent = client
    .agent("gpt-4")
    .preamble("You are a helpful assistant")
    .tool(my_custom_tool)  // Now you can use tools!
    .build();

let response = agent.prompt(input).await?;
```

The concepts transfer directly - you're just getting more control.

## Design Tradeoffs

To prioritize ergonomics and pattern-switching simplicity, `rig-patterns` makes these tradeoffs:

| Limitation | Reason | Workaround |
|------------|--------|------------|
| String-only I/O | Simpler interface | Use raw rig-core for structured data |
| No streaming | Simpler patterns | Use raw rig-core for streaming |
| Basic error handling | Fail-fast is clearer | Add retry logic in application layer |
| No tool calling | Simplified agent model | Use raw rig-core for tool use |
| Implicit managers | Fewer config options | First agent is manager in GroupChat/Magentic |

These are **intentional** tradeoffs to achieve the goal: change patterns with one line of code.

## Features

- **Default features:** Core orchestration patterns
- **`yaml`:** YAML configuration file support

```toml
# Just the core library
rig-patterns = "0.1"

# With YAML config support
rig-patterns = { version = "0.1", features = ["yaml"] }
```

## Architecture

```
src/
├── lib.rs              # Public API
├── orchestrator.rs     # Orchestrator builder
├── agent.rs            # Agent wrapper
├── patterns/
│   ├── mod.rs          # Pattern trait and types
│   ├── sequential.rs   # Sequential execution
│   ├── concurrent.rs   # Concurrent with aggregation
│   ├── group_chat.rs   # Multi-round conversation
│   ├── handoff.rs      # Task routing
│   └── magentic.rs     # Manager-worker coordination
└── config.rs           # YAML support (optional)
```

Each pattern implements a simple trait:

```rust
#[async_trait]
trait PatternExecutor {
    async fn execute(&self, agents: &[Agent], input: &str)
        -> Result<OrchestratorResult>;
}
```

## Contributing

Contributions are welcome! This library prioritizes simplicity and ergonomics over features.

**Good contributions:**
- Bug fixes
- Performance improvements
- Better documentation
- New examples
- Test coverage

**Please discuss first:**
- New orchestration patterns
- Breaking API changes
- New required dependencies

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

Built on top of the excellent [rig-core](https://github.com/0xPlaygrounds/rig) framework.

Inspired by the need for rapid experimentation with multi-agent systems.
