//! Orchestrator builder for managing LLM agent patterns

use crate::{Agent, OrchestratorResult};
use crate::patterns::{Pattern, create_executor};
use anyhow::Result;

/// Main orchestrator for executing agent patterns
pub struct Orchestrator {
    agents: Vec<Agent>,
    executor: Box<dyn crate::patterns::PatternExecutor>,
}

impl Orchestrator {
    /// Create a new orchestrator builder
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rig_patterns::{Orchestrator, Pattern, Agent};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let agents = vec![
    ///     // Agent::new("agent1", model, "system prompt"),
    /// ];
    ///
    /// let orchestrator = Orchestrator::new(agents)
    ///     .pattern(Pattern::Sequential)
    ///     .build()?;
    ///
    /// let result = orchestrator.execute("input").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(agents: Vec<Agent>) -> OrchestratorBuilder {
        OrchestratorBuilder {
            agents,
            pattern: Pattern::default(),
        }
    }

    /// Execute the orchestrator with the given input
    ///
    /// # Arguments
    ///
    /// * `input` - The input text to process
    ///
    /// # Returns
    ///
    /// An `OrchestratorResult` containing the output and metadata
    pub async fn execute(&self, input: &str) -> Result<OrchestratorResult> {
        tracing::info!("Orchestrator executing with {} agents", self.agents.len());
        self.executor.execute(&self.agents, input).await
    }

    /// Get the agents in this orchestrator
    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    /// Get the number of agents
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }
}

/// Builder for constructing an Orchestrator
pub struct OrchestratorBuilder {
    agents: Vec<Agent>,
    pattern: Pattern,
}

impl OrchestratorBuilder {
    /// Set the orchestration pattern
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rig_patterns::{Orchestrator, Pattern, Aggregation};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let orchestrator = Orchestrator::new(vec![])
    ///     .pattern(Pattern::Concurrent {
    ///         aggregation: Aggregation::Vote
    ///     })
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn pattern(mut self, pattern: Pattern) -> Self {
        self.pattern = pattern;
        self
    }

    /// Build the orchestrator
    ///
    /// # Returns
    ///
    /// A configured `Orchestrator` ready to execute
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid (e.g., no agents provided)
    pub fn build(self) -> Result<Orchestrator> {
        if self.agents.is_empty() {
            anyhow::bail!("Orchestrator requires at least one agent");
        }

        let executor = create_executor(self.pattern);

        Ok(Orchestrator {
            agents: self.agents,
            executor,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_requires_agents() {
        let result = Orchestrator::new(vec![]).build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one agent"));
    }
}
