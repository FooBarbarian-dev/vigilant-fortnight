//! Handoff pattern: agents explicitly pass tasks to each other

use super::{PatternExecutor, PatternMetadata};
use crate::{Agent, OrchestratorResult};
use async_trait::async_trait;
use std::collections::HashMap;

/// Handoff pattern executor
///
/// Agents decide whether to handle a task or pass it to another agent.
/// Uses special markers in agent responses:
/// - "HANDLE" or no marker: agent handles the task
/// - "HANDOFF:agent_id": pass to specified agent
pub(crate) struct HandoffExecutor {
    max_hops: usize,
}

impl HandoffExecutor {
    pub fn new(max_hops: usize) -> Self {
        Self { max_hops }
    }

    /// Parse the agent's response for handoff signals
    ///
    /// Returns (should_handoff, next_agent_id, actual_content)
    fn parse_handoff(response: &str) -> (bool, Option<String>, String) {
        let lines: Vec<&str> = response.lines().collect();

        // Check first few lines for HANDOFF marker
        for line in lines.iter().take(5) {
            let trimmed = line.trim().to_uppercase();

            if let Some(rest) = trimmed.strip_prefix("HANDOFF:") {
                let next_agent = rest.trim().to_string();
                return (true, Some(next_agent), response.to_string());
            }

            if trimmed.starts_with("HANDOFF ") {
                // Handle "HANDOFF agent_id" format
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    let next_agent = parts[1].to_string();
                    return (true, Some(next_agent), response.to_string());
                }
            }
        }

        // Check if response explicitly says "HANDLE"
        let upper = response.to_uppercase();
        if upper.contains("HANDLE") && !upper.contains("HANDOFF") {
            return (false, None, response.to_string());
        }

        // Default: no handoff detected, agent handles it
        (false, None, response.to_string())
    }

    /// Find an agent by ID
    fn find_agent<'a>(agents: &'a [Agent], agent_id: &str) -> Option<&'a Agent> {
        agents.iter().find(|a| a.id() == agent_id)
    }
}

#[async_trait]
impl PatternExecutor for HandoffExecutor {
    async fn execute(&self, agents: &[Agent], input: &str) -> anyhow::Result<OrchestratorResult> {
        if agents.is_empty() {
            anyhow::bail!("Handoff pattern requires at least one agent");
        }

        if self.max_hops == 0 {
            anyhow::bail!("Handoff pattern requires max_hops > 0");
        }

        tracing::info!(
            "Executing handoff pattern with {} agents, max {} hops",
            agents.len(),
            self.max_hops
        );

        let mut metadata = PatternMetadata::new();
        metadata.add_detail("pattern", "handoff");
        metadata.add_detail("agent_count", agents.len().to_string());
        metadata.add_detail("max_hops", self.max_hops.to_string());
        metadata.add_trace(format!(
            "Starting handoff with {} agents, max {} hops",
            agents.len(),
            self.max_hops
        ));

        // Track visit counts to prevent loops
        let mut visit_counts: HashMap<String, usize> = HashMap::new();
        let mut handoff_chain: Vec<String> = Vec::new();

        // Start with the first agent
        let mut current_agent = &agents[0];
        let mut current_input = input.to_string();
        let mut final_output = String::new();

        metadata.add_trace(format!("Starting with agent '{}'", current_agent.id()));
        handoff_chain.push(current_agent.id().to_string());

        for hop in 0..self.max_hops {
            let agent_id = current_agent.id().to_string();

            // Track visits
            *visit_counts.entry(agent_id.clone()).or_insert(0) += 1;

            // Prevent infinite loops (agent visited too many times)
            if visit_counts[&agent_id] > 2 {
                metadata.add_trace(format!(
                    "Agent '{}' visited too many times, stopping handoff",
                    agent_id
                ));
                anyhow::bail!(
                    "Handoff loop detected: agent '{}' visited {} times",
                    agent_id,
                    visit_counts[&agent_id]
                );
            }

            // Build context with handoff instructions
            let context = format!(
                "{}\n\nInstructions: Evaluate if you can handle this task.\n\
                - If you can handle it, process the request and respond with your answer.\n\
                - If another agent is better suited, respond with 'HANDOFF:agent_id' on the first line.\n\
                Available agents: {}",
                current_input,
                agents
                    .iter()
                    .map(|a| format!("{} ({})", a.id(), a.system_prompt()))
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            tracing::debug!(
                hop = hop + 1,
                agent_id = %current_agent.id(),
                "Handoff: prompting agent"
            );

            let response = current_agent.prompt(&context).await.map_err(|e| {
                anyhow::anyhow!("Agent '{}' failed at hop {}: {}", current_agent.id(), hop + 1, e)
            })?;

            // Parse for handoff
            let (should_handoff, next_agent_id, content) = Self::parse_handoff(&response);

            if should_handoff {
                if let Some(next_id) = next_agent_id {
                    metadata.add_trace(format!(
                        "Hop {}: Agent '{}' handed off to '{}'",
                        hop + 1,
                        current_agent.id(),
                        next_id
                    ));

                    // Find next agent
                    if let Some(next_agent) = Self::find_agent(agents, &next_id) {
                        current_agent = next_agent;
                        handoff_chain.push(next_id.clone());
                        // Keep the current input for the next agent
                        continue;
                    } else {
                        metadata.add_trace(format!(
                            "Agent '{}' not found, using current agent's response",
                            next_id
                        ));
                        final_output = content;
                        break;
                    }
                } else {
                    // Handoff requested but no agent specified
                    metadata.add_trace(format!(
                        "Hop {}: Agent '{}' requested handoff but didn't specify target",
                        hop + 1,
                        current_agent.id()
                    ));
                    final_output = content;
                    break;
                }
            } else {
                // Agent handled the task
                metadata.add_trace(format!(
                    "Hop {}: Agent '{}' handled the task",
                    hop + 1,
                    current_agent.id()
                ));
                final_output = content;
                break;
            }
        }

        metadata.add_detail("handoff_chain", handoff_chain.join(" → "));
        metadata.add_detail("total_hops", handoff_chain.len().to_string());
        metadata.add_trace(format!("Handoff chain: {}", handoff_chain.join(" → ")));

        let output = format!(
            "HANDOFF RESULT:\n\nHandoff chain: {}\n\n--- Final Response ---\n{}",
            handoff_chain.join(" → "),
            final_output
        );

        Ok(OrchestratorResult::new(output, metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_handoff() {
        let (handoff, agent, _) = HandoffExecutor::parse_handoff("HANDOFF:agent2\nSome content");
        assert!(handoff);
        assert_eq!(agent, Some("AGENT2".to_string()));

        let (handoff, agent, _) = HandoffExecutor::parse_handoff("HANDLE\nI'll do it");
        assert!(!handoff);
        assert_eq!(agent, None);

        let (handoff, agent, _) = HandoffExecutor::parse_handoff("Just a normal response");
        assert!(!handoff);
        assert_eq!(agent, None);
    }

    #[test]
    fn test_handoff_requires_agents() {
        let executor = HandoffExecutor::new(5);
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(executor.execute(&[], "test input"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one agent"));
    }
}
