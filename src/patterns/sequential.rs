//! Sequential pattern: chain agents where each receives the previous output

use super::{PatternExecutor, PatternMetadata};
use crate::{Agent, OrchestratorResult};
use async_trait::async_trait;

/// Sequential pattern executor
///
/// Chains agents together: agent[0] → agent[1] → ... → agent[n]
/// Each agent receives the previous agent's output as input.
pub(crate) struct SequentialExecutor;

#[async_trait]
impl PatternExecutor for SequentialExecutor {
    async fn execute(&self, agents: &[Agent], input: &str) -> anyhow::Result<OrchestratorResult> {
        if agents.is_empty() {
            anyhow::bail!("Sequential pattern requires at least one agent");
        }

        tracing::info!("Executing sequential pattern with {} agents", agents.len());

        let mut metadata = PatternMetadata::new();
        metadata.add_detail("pattern", "sequential");
        metadata.add_detail("agent_count", agents.len().to_string());
        metadata.add_trace(format!("Starting sequential execution with {} agents", agents.len()));

        let mut current_input = input.to_string();

        for (idx, agent) in agents.iter().enumerate() {
            metadata.add_trace(format!(
                "Step {}/{}: Invoking agent '{}' ({})",
                idx + 1,
                agents.len(),
                agent.id(),
                agent.system_prompt()
            ));

            tracing::debug!(
                step = idx + 1,
                total = agents.len(),
                agent_id = %agent.id(),
                "Sequential: prompting agent"
            );

            let response = agent.prompt(&current_input).await.map_err(|e| {
                anyhow::anyhow!("Agent '{}' failed at step {}: {}", agent.id(), idx + 1, e)
            })?;

            metadata.add_trace(format!(
                "Step {}/{}: Agent '{}' responded ({} chars)",
                idx + 1,
                agents.len(),
                agent.id(),
                response.len()
            ));

            current_input = response;
        }

        metadata.add_trace(format!(
            "Sequential execution completed. Final output: {} chars",
            current_input.len()
        ));

        Ok(OrchestratorResult::new(current_input, metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full integration tests would require mock agents
    // For now, we test the structure

    #[test]
    fn test_sequential_requires_agents() {
        let executor = SequentialExecutor;
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(executor.execute(&[], "test input"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one agent"));
    }
}
