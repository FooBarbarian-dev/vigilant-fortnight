//! Concurrent pattern: run agents in parallel and aggregate results

use super::{Aggregation, PatternExecutor, PatternMetadata};
use crate::{Agent, OrchestratorResult};
use async_trait::async_trait;
use std::collections::HashMap;

/// Concurrent pattern executor
///
/// Runs all agents in parallel with the same input, then aggregates results
/// using the specified aggregation strategy.
pub(crate) struct ConcurrentExecutor {
    aggregation: Aggregation,
}

impl ConcurrentExecutor {
    pub fn new(aggregation: Aggregation) -> Self {
        Self { aggregation }
    }

    /// Aggregate results using consensus (simplified version)
    async fn aggregate_consensus(&self, results: Vec<(String, String)>) -> anyhow::Result<String> {
        // For simplicity, we'll use the first agent's result as the "consensus"
        // In a real implementation, you'd use another LLM call to reconcile differences
        tracing::warn!(
            "Consensus aggregation is simplified - using first agent's result as consensus"
        );

        if let Some((_, first_result)) = results.first() {
            Ok(format!(
                "CONSENSUS (simplified from {} agents):\n\n{}",
                results.len(),
                first_result
            ))
        } else {
            Ok("No results to aggregate".to_string())
        }
    }

    /// Aggregate results using voting (simple majority)
    fn aggregate_vote(&self, results: Vec<(String, String)>) -> String {
        let mut counts: HashMap<String, usize> = HashMap::new();
        let mut agent_map: HashMap<String, Vec<String>> = HashMap::new();

        for (agent_id, result) in results.iter() {
            let normalized = result.trim().to_lowercase();
            *counts.entry(normalized.clone()).or_insert(0) += 1;
            agent_map
                .entry(normalized)
                .or_insert_with(Vec::new)
                .push(agent_id.clone());
        }

        if let Some((winner, count)) = counts.iter().max_by_key(|(_, &count)| count) {
            let agents = agent_map.get(winner).unwrap();
            format!(
                "VOTE RESULT ({} of {} agents):\nAgents in agreement: {}\n\nResult:\n{}",
                count,
                results.len(),
                agents.join(", "),
                // Find original (non-normalized) result
                results
                    .iter()
                    .find(|(id, _)| agents.contains(id))
                    .map(|(_, r)| r.as_str())
                    .unwrap_or(winner)
            )
        } else {
            "No clear majority".to_string()
        }
    }

    /// Aggregate results by combining all outputs
    fn aggregate_combine(&self, results: Vec<(String, String)>) -> String {
        let mut combined = String::from("COMBINED RESULTS:\n\n");

        for (agent_id, result) in results {
            combined.push_str(&format!("=== Agent: {} ===\n{}\n\n", agent_id, result));
        }

        combined
    }
}

#[async_trait]
impl PatternExecutor for ConcurrentExecutor {
    async fn execute(&self, agents: &[Agent], input: &str) -> anyhow::Result<OrchestratorResult> {
        if agents.is_empty() {
            anyhow::bail!("Concurrent pattern requires at least one agent");
        }

        tracing::info!(
            "Executing concurrent pattern with {} agents, aggregation: {:?}",
            agents.len(),
            self.aggregation
        );

        let mut metadata = PatternMetadata::new();
        metadata.add_detail("pattern", "concurrent");
        metadata.add_detail("agent_count", agents.len().to_string());
        metadata.add_detail("aggregation", format!("{:?}", self.aggregation));
        metadata.add_trace(format!(
            "Starting concurrent execution with {} agents",
            agents.len()
        ));

        // Launch all agents in parallel using real threads (not just cooperative async)
        let mut tasks = Vec::new();
        for agent in agents.iter() {
            let agent_clone = agent.clone();
            let input_clone = input.to_string();
            let agent_id = agent.id().to_string();

            tracing::debug!(agent_id = %agent_id, "Concurrent: spawning thread for agent");

            // Use std::thread for true parallel execution
            tasks.push(std::thread::spawn(move || {
                // Create a tokio runtime in this thread for the async agent call
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| anyhow::anyhow!("Failed to create runtime: {}", e))?;

                let result = rt.block_on(async {
                    agent_clone.prompt(&input_clone).await
                });

                Ok::<(String, anyhow::Result<String>), anyhow::Error>((agent_id, result))
            }));
        }

        metadata.add_trace(format!("Launched {} concurrent threads", tasks.len()));

        // Wait for all results
        let mut results = Vec::new();
        for task in tasks {
            match task.join() {
                Ok(Ok((agent_id, result))) => results.push((agent_id, result)),
                Ok(Err(e)) => return Err(e),
                Err(_) => return Err(anyhow::anyhow!("Thread panicked")),
            }
        }

        // Separate successful and failed results
        let mut successes = Vec::new();
        let mut failures = Vec::new();

        for (agent_id, result) in results {
            match result {
                Ok(output) => {
                    metadata.add_trace(format!(
                        "Agent '{}' succeeded ({} chars)",
                        agent_id,
                        output.len()
                    ));
                    successes.push((agent_id, output));
                }
                Err(e) => {
                    metadata.add_trace(format!("Agent '{}' failed: {}", agent_id, e));
                    failures.push((agent_id, e));
                }
            }
        }

        metadata.add_detail("successful_agents", successes.len().to_string());
        metadata.add_detail("failed_agents", failures.len().to_string());

        if successes.is_empty() {
            anyhow::bail!(
                "All {} agents failed. First error: {}",
                failures.len(),
                failures.first().map(|(_, e)| e.to_string()).unwrap_or_default()
            );
        }

        // Aggregate successful results
        metadata.add_trace(format!("Aggregating {} results", successes.len()));

        let output = match self.aggregation {
            Aggregation::Consensus => self.aggregate_consensus(successes).await?,
            Aggregation::Vote => self.aggregate_vote(successes),
            Aggregation::Combine => self.aggregate_combine(successes),
        };

        metadata.add_trace(format!("Aggregation complete. Final output: {} chars", output.len()));

        Ok(OrchestratorResult::new(output, metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concurrent_requires_agents() {
        let executor = ConcurrentExecutor::new(Aggregation::Combine);
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(executor.execute(&[], "test input"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one agent"));
    }

    #[test]
    fn test_aggregate_combine() {
        let executor = ConcurrentExecutor::new(Aggregation::Combine);
        let results = vec![
            ("agent1".to_string(), "result1".to_string()),
            ("agent2".to_string(), "result2".to_string()),
        ];

        let combined = executor.aggregate_combine(results);
        assert!(combined.contains("agent1"));
        assert!(combined.contains("agent2"));
        assert!(combined.contains("result1"));
        assert!(combined.contains("result2"));
    }
}
