//! # rig-patterns
//!
//! Dead-simple orchestration pattern abstractions over the `rig-core` LLM framework.
//!
//! This library enables developers to switch between different LLM orchestration patterns
//! with minimal code changes - typically just changing a single enum variant.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rig_patterns::{Orchestrator, Pattern, Agent};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let agents = vec![
//!     // Agent::new("summarizer", provider, "You summarize text"),
//!     // Agent::new("critic", provider, "You critique summaries"),
//! ];
//!
//! // Switch patterns by changing this ONE line
//! let orchestrator = Orchestrator::new(agents)
//!     .pattern(Pattern::Sequential)  // or Concurrent, GroupChat, Handoff, Magentic
//!     .build()?;
//!
//! let result = orchestrator.execute("input text").await?;
//! println!("Output: {}", result.output);
//! # Ok(())
//! # }
//! ```
//!
//! ## Available Patterns
//!
//! - **Sequential**: Chain agents where each receives the previous agent's output
//! - **Concurrent**: Run all agents in parallel and aggregate results
//! - **GroupChat**: Agents converse in rounds until consensus or max rounds
//! - **Handoff**: Agents explicitly hand off tasks to each other
//! - **Magentic**: Manager agent coordinates worker agents on a task list
//!

pub mod agent;
pub mod orchestrator;
pub mod patterns;

#[cfg(feature = "yaml")]
pub mod config;

pub use agent::Agent;
pub use orchestrator::{Orchestrator, OrchestratorBuilder};
pub use patterns::{Aggregation, Pattern, PatternMetadata};

/// Result type for orchestration execution
#[derive(Debug, Clone)]
pub struct OrchestratorResult {
    /// The final output from the orchestration pattern
    pub output: String,
    /// Pattern-specific metadata (execution trace, timing, etc.)
    pub pattern_metadata: PatternMetadata,
}

impl OrchestratorResult {
    /// Create a new orchestrator result
    pub fn new(output: String, pattern_metadata: PatternMetadata) -> Self {
        Self {
            output,
            pattern_metadata,
        }
    }
}

/// Re-export common error type
pub type Result<T> = anyhow::Result<T>;
