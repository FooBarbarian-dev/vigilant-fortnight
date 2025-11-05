# rig-patterns-ui

**Interactive web demonstration for the rig-patterns LLM orchestration library**

A beautiful, real-time web interface that makes pattern behavior **visually obvious** through live execution visualization. Perfect for understanding, experimenting with, and demonstrating different multi-agent orchestration patterns.

## Features

### 🎨 Visual Pattern Demonstrations

Each orchestration pattern has a distinct, real-time visualization:

- **Sequential (→→→)**: Watch agents chain together, each building on the previous output
- **Concurrent (⇉)**: See agents execute in parallel, then aggregate results
- **Group Chat (💬)**: Follow the conversation as agents debate and refine ideas
- **Handoff (↷)**: Trace task routing between specialist agents
- **Magentic (☰)**: Monitor task breakdown and execution by worker agents

### 🚀 Key Capabilities

- **Zero-config Demo**: Load page → select preset → execute → see visualization
- **Real-time Streaming**: Watch execution unfold step-by-step via WebSocket
- **Pattern Comparison**: Run the same input through all 5 patterns side-by-side
- **Preset Configurations**: 5 ready-to-use examples for common use cases
- **Dynamic Agent Management**: Add, remove, and configure agents on the fly
- **Responsive Design**: Works on desktop, tablet, and mobile devices

## Quick Start

### Prerequisites

- Rust 1.70+ with Cargo
- A modern web browser (Chrome, Firefox, Safari, Edge)

### Installation & Running

```bash
# From the repository root
cd rig-patterns-ui

# Run the server (builds and starts on http://localhost:3000)
cargo run

# Or build for release
cargo build --release
./target/release/rig-patterns-ui
```

The server will start on `http://localhost:3000`. Open this URL in your browser.

### First-Time Usage

1. **Load a preset** from the dropdown (e.g., "Document Processing Pipeline")
2. Click **"Execute Pattern"** to see the mock execution
3. Try **"Execute with Streaming"** to see real-time visualization
4. Click **"Compare All Patterns"** to see side-by-side results

## UI Overview

### Left Panel: Agent Configuration

- **Add agents** with custom IDs, providers, models, and system prompts
- **Remove agents** by clicking the × button
- **Load presets** to quickly configure common scenarios

### Center Area: Pattern Execution

- **Select pattern type** from dropdown (Sequential, Concurrent, etc.)
- **Configure pattern options** (aggregation strategy, max rounds, etc.)
- **Enter input prompt** for agents to process
- **Execute** with standard HTTP or streaming WebSocket

### Visualization Panel

Real-time, pattern-specific visualizations:

```
Sequential:  [Agent1] → [Agent2] → [Agent3]

Concurrent:     [Agent1]
                [Agent2]  ⇉ Aggregation
                [Agent3]

Group Chat:     💬 Conversation thread

Handoff:        [Triage] ↷ [Specialist] ↷ [Resolver]

Magentic:       ☰ Task List with completion status
```

### Results Display

- **Final output** with syntax highlighting
- **Execution metadata** (pattern, duration, agent count)
- **Execution trace** (expandable detail view)

## Available Presets

1. **Document Processing Pipeline** - Sequential extraction, summarization, and formatting
2. **Multi-Perspective Code Review** - Concurrent review from security, performance, and style experts
3. **Customer Support Triage** - Handoff routing to specialist agents
4. **Research Synthesis** - Manager-coordinated research with fact-checking
5. **Collaborative Writing** - Group chat with writers, editors, and fact-checkers

## API Endpoints

### `POST /api/execute`

Execute a single pattern configuration.

**Request:**
```json
{
  "agents": [
    {
      "id": "writer",
      "provider": "openai",
      "model": "gpt-4",
      "system_prompt": "You write clear content."
    }
  ],
  "pattern": {
    "type": "sequential"
  },
  "input": "Write a haiku about Rust"
}
```

**Response:**
```json
{
  "output": "Final aggregated result",
  "execution_trace": ["Step 1...", "Step 2..."],
  "metadata": { "pattern": "sequential", "steps": 3 },
  "pattern_name": "Sequential",
  "duration_ms": 4200
}
```

### `POST /api/compare`

Compare all patterns with the same input.

**Request:**
```json
{
  "agents": [...],
  "input": "Explain async programming"
}
```

**Response:**
```json
{
  "results": {
    "Sequential": { "output": "...", ... },
    "Concurrent (Vote)": { "output": "...", ... },
    ...
  }
}
```

### `GET /api/presets`

Get all available preset configurations.

**Response:**
```json
[
  {
    "name": "Document Processing Pipeline",
    "description": "Sequential processing: extract → summarize → format",
    "agents": [...],
    "pattern": { "type": "sequential" },
    "sample_input": "Process this quarterly report..."
  },
  ...
]
```

### `GET /ws` (WebSocket)

Real-time execution streaming.

**Client sends:**
```json
{
  "agents": [...],
  "pattern": { "type": "sequential" },
  "input": "Your prompt"
}
```

**Server sends events:**
```json
{"type": "agent_start", "agent_id": "writer", "timestamp": "..."}
{"type": "agent_complete", "agent_id": "writer", "output_preview": "...", "timestamp": "..."}
{"type": "pattern_complete", "output": "...", "metadata": {...}, "timestamp": "..."}
```

## Architecture

```
rig-patterns-ui/
├── src/
│   ├── main.rs              # Axum server setup
│   ├── state.rs             # Shared app state and models
│   └── routes/
│       ├── mod.rs
│       ├── execute.rs       # POST /api/execute
│       ├── compare.rs       # POST /api/compare
│       ├── presets.rs       # GET /api/presets
│       └── websocket.rs     # WebSocket handler
├── static/
│   ├── index.html           # Main UI
│   ├── styles.css           # Styling and animations
│   └── app.js               # Frontend logic
└── Cargo.toml
```

## Development

### Local Development

The server automatically serves static files from the `static/` directory with hot reload support (restart the server to see changes).

### Adding New Presets

Edit `src/state.rs` and add to the `default_presets()` function:

```rust
PresetConfig {
    name: "My Custom Preset".to_string(),
    description: "Describe what this preset does".to_string(),
    agents: vec![
        AgentConfig {
            id: "agent1".to_string(),
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            system_prompt: "You are...".to_string(),
        },
    ],
    pattern: PatternConfig::Sequential,
    sample_input: "Sample prompt...".to_string(),
}
```

### Customizing Visualizations

Edit `static/styles.css` for visual styling, and `static/app.js` for interaction logic. Each pattern has its own CSS classes and JavaScript handlers.

## Mock vs. Real Execution

**Current Implementation**: The UI currently uses **mock execution** to demonstrate the interface without requiring real LLM API credentials. Mock responses simulate realistic execution patterns.

**Real LLM Integration**: To connect real LLM providers:

1. Add environment variables for API keys (`OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, etc.)
2. Update `src/routes/execute.rs` to create actual `Agent` instances using `rig-core`
3. Replace mock responses with real orchestrator execution

Example real implementation:
```rust
// In execute.rs, replace mock with:
let mut agents = Vec::new();
for config in request.agents {
    let client = create_client(&config.provider, &config.model)?;
    let agent = Agent::new(&config.id, client, &config.system_prompt);
    agents.push(agent);
}

let orchestrator = Orchestrator::new(agents)
    .pattern(request.pattern.into())
    .build()?;

let result = orchestrator.execute(&request.input).await?;
```

## Production Considerations

For production deployment, consider:

- **Authentication**: Add user authentication and API key management
- **Rate limiting**: Prevent abuse of expensive LLM calls
- **Caching**: Cache results for identical requests
- **Monitoring**: Track pattern usage, execution times, and errors
- **Scaling**: Deploy behind a load balancer for high traffic

## Educational Use

This UI is perfect for:

- **Teaching** multi-agent orchestration concepts
- **Workshops** on LLM patterns
- **Demos** for stakeholders
- **Rapid prototyping** of agent configurations
- **Research** into pattern effectiveness

## Contributing

Contributions are welcome! Areas for improvement:

- Additional preset configurations
- Enhanced visualizations
- Mobile UI improvements
- New pattern implementations
- Performance optimizations

## License

Licensed under MIT OR Apache-2.0 (same as parent project).

## Acknowledgments

Built with:
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [rig-patterns](../README.md) - Core orchestration library
- [Tower](https://github.com/tower-rs/tower) - Middleware
- Vanilla JavaScript (no framework dependencies!)

---

**Ready to explore?** Run `cargo run` and open http://localhost:3000 🚀
