//! Orchestration pattern implementations

use crate::{Agent, OrchestratorResult};
use async_trait::async_trait;
use std::collections::HashMap;

pub mod sequential;
pub mod concurrent;
pub mod group_chat;
pub mod handoff;
pub mod magentic;

/// Aggregation strategy for concurrent pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "yaml", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "yaml", serde(rename_all = "lowercase"))]
pub enum Aggregation {
    /// Use an LLM to reconcile differences between outputs
    Consensus,
    /// Simple majority vote on similar outputs
    Vote,
    /// Concatenate all outputs with separators
    Combine,
}

/// Available orchestration patterns
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "yaml", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "yaml", serde(tag = "type", rename_all = "snake_case"))]
pub enum Pattern {
    /// Sequential chaining: agent[0] → agent[1] → ... → agent[n]
    Sequential,

    /// Concurrent execution with result aggregation
    Concurrent {
        /// How to aggregate the concurrent results
        aggregation: Aggregation,
    },

    /// Group chat with agents conversing in rounds
    GroupChat {
        /// Maximum number of conversation rounds
        max_rounds: usize,
    },

    /// Explicit task handoff between agents
    Handoff {
        /// Maximum number of handoffs to prevent loops
        max_hops: usize,
    },

    /// Manager-coordinated task execution
    Magentic {
        /// Maximum task execution iterations
        max_iterations: usize,
    },
}

impl Default for Pattern {
    fn default() -> Self {
        Self::Sequential
    }
}

/// Metadata about pattern execution
#[derive(Debug, Clone)]
pub struct PatternMetadata {
    /// Pattern-specific execution details
    pub details: HashMap<String, String>,
    /// Execution trace/log
    pub trace: Vec<String>,
}

impl PatternMetadata {
    /// Create new empty metadata
    pub fn new() -> Self {
        Self {
            details: HashMap::new(),
            trace: Vec::new(),
        }
    }

    /// Add a detail entry
    pub fn add_detail(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.details.insert(key.into(), value.into());
    }

    /// Add a trace entry
    pub fn add_trace(&mut self, message: impl Into<String>) {
        self.trace.push(message.into());
    }
}

impl Default for PatternMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal trait for pattern execution
#[async_trait]
pub(crate) trait PatternExecutor: Send + Sync {
    async fn execute(&self, agents: &[Agent], input: &str) -> anyhow::Result<OrchestratorResult>;
}

/// Create a pattern executor from a Pattern enum
pub(crate) fn create_executor(pattern: Pattern) -> Box<dyn PatternExecutor> {
    match pattern {
        Pattern::Sequential => Box::new(sequential::SequentialExecutor),
        Pattern::Concurrent { aggregation } => {
            Box::new(concurrent::ConcurrentExecutor::new(aggregation))
        }
        Pattern::GroupChat { max_rounds } => {
            Box::new(group_chat::GroupChatExecutor::new(max_rounds))
        }
        Pattern::Handoff { max_hops } => Box::new(handoff::HandoffExecutor::new(max_hops)),
        Pattern::Magentic { max_iterations } => {
            Box::new(magentic::MagenticExecutor::new(max_iterations))
        }
    }
}
