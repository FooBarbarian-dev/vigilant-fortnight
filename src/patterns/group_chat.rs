//! Group chat pattern: agents converse in rounds until consensus or max rounds

use super::{PatternExecutor, PatternMetadata, ResolutionStrategy};
use crate::{Agent, OrchestratorResult};
use async_trait::async_trait;

/// Group chat pattern executor
///
/// Agents converse in rounds, building on each other's messages.
/// Stops based on the resolution strategy (consensus, first complete, all rounds, or majority).
pub(crate) struct GroupChatExecutor {
    max_rounds: usize,
    resolution: ResolutionStrategy,
}

impl GroupChatExecutor {
    pub fn new(max_rounds: usize, resolution: ResolutionStrategy) -> Self {
        Self { max_rounds, resolution }
    }

    /// Check if the message indicates consensus has been reached
    fn check_consensus(message: &str) -> bool {
        let normalized = message.to_uppercase();
        normalized.contains("CONSENSUS_REACHED")
            || normalized.contains("CONSENSUS REACHED")
            || normalized.contains("WE AGREE")
            || normalized.contains("AGREED")
    }

}

#[async_trait]
impl PatternExecutor for GroupChatExecutor {
    async fn execute(&self, agents: &[Agent], input: &str) -> anyhow::Result<OrchestratorResult> {
        if agents.is_empty() {
            anyhow::bail!("GroupChat pattern requires at least one agent");
        }

        if self.max_rounds == 0 {
            anyhow::bail!("GroupChat pattern requires max_rounds > 0");
        }

        tracing::info!(
            "Executing group chat pattern with {} agents, max {} rounds",
            agents.len(),
            self.max_rounds
        );

        let mut metadata = PatternMetadata::new();
        metadata.add_detail("pattern", "group_chat");
        metadata.add_detail("agent_count", agents.len().to_string());
        metadata.add_detail("max_rounds", self.max_rounds.to_string());
        metadata.add_detail("resolution", format!("{:?}", self.resolution));
        metadata.add_trace(format!(
            "Starting group chat with {} agents, max {} rounds, resolution: {:?}",
            agents.len(),
            self.max_rounds,
            self.resolution
        ));

        let mut conversation_history: Vec<(String, String)> = Vec::new();
        let mut consensus_count = 0;

        for round in 0..self.max_rounds {
            metadata.add_trace(format!("--- Round {} (parallel execution) ---", round + 1));

            // Build context once - all agents see the same history at round start
            let base_context = if conversation_history.is_empty() {
                format!(
                    "TOPIC: {}\n\n\
                    You are participating in a group discussion with other agents. \
                    Please provide your initial perspective on this topic. \
                    Be thoughtful and contribute to a productive discussion.",
                    input
                )
            } else {
                let mut context = format!("TOPIC: {}\n\nDISCUSSION HISTORY:\n", input);

                // Calculate which round each message belongs to
                let agents_per_round = agents.len();
                for (idx, (agent_id, message)) in conversation_history.iter().enumerate() {
                    let msg_round = (idx / agents_per_round) + 1;
                    context.push_str(&format!("\n[Round {}] {}: {}\n", msg_round, agent_id, message));
                }

                if round + 1 == self.max_rounds {
                    context.push_str("\n\nYour turn to respond. This is the final round - reference specific points made by others. If you believe we've reached a good conclusion, include 'CONSENSUS_REACHED' in your response.");
                } else {
                    context.push_str("\n\nYour turn to respond. Reference specific points made by others and engage with what has been said.");
                }
                context
            };

            // Spawn all agents in parallel for this round
            let mut tasks = Vec::new();
            for agent in agents.iter() {
                let agent_id = agent.id().to_string();
                let context_clone = base_context.clone();
                let agent_clone = agent.clone();

                tracing::debug!(
                    round = round + 1,
                    agent_id = %agent_id,
                    "GroupChat: spawning agent task (parallel)"
                );

                tasks.push(tokio::spawn(async move {
                    let response = agent_clone.prompt(&context_clone).await.map_err(|e| {
                        anyhow::anyhow!("Agent '{}' failed: {}", agent_id, e)
                    })?;
                    Ok::<(String, String), anyhow::Error>((agent_id, response))
                }));
            }

            // Wait for all agents to complete
            for task in tasks {
                let (agent_id, response) = task.await.map_err(|e| {
                    anyhow::anyhow!("Task join error: {}", e)
                })??;

                metadata.add_trace(format!(
                    "Round {}: {} said: {}",
                    round + 1,
                    agent_id,
                    response.chars().take(100).collect::<String>()
                        + if response.len() > 100 { "..." } else { "" }
                ));

                conversation_history.push((agent_id.clone(), response.clone()));

                // Check for consensus/completion signals
                if Self::check_consensus(&response) {
                    consensus_count += 1;
                    metadata.add_trace(format!(
                        "Consensus signal from agent '{}' in round {} (total: {}/{})",
                        agent_id,
                        round + 1,
                        consensus_count,
                        agents.len()
                    ));
                }
            }

            // Check resolution strategy
            let should_stop = match self.resolution {
                ResolutionStrategy::Consensus => consensus_count > 0,
                ResolutionStrategy::FirstToComplete => consensus_count > 0,
                ResolutionStrategy::AllRounds => false, // Never stop early
                ResolutionStrategy::Majority => consensus_count > agents.len() / 2,
            };

            if should_stop && round + 1 < self.max_rounds {
                metadata.add_trace(format!(
                    "Stopping early due to {:?} resolution strategy",
                    self.resolution
                ));
                break;
            }

            // Reset consensus count for next round (for strategies that check per-round)
            consensus_count = 0;
        }

        let rounds_completed = ((conversation_history.len() - 1) / agents.len()).max(0) + 1;
        let consensus_reached = consensus_count > 0;

        metadata.add_detail("rounds_completed", rounds_completed.to_string());
        metadata.add_detail("consensus_reached", consensus_reached.to_string());
        metadata.add_detail("messages_exchanged", conversation_history.len().to_string());

        // Build final output with structured format
        let mut output = format!("# GROUP CHAT DISCUSSION\n\n**Topic:** {}\n\n## Conversation\n", input);

        // Calculate rounds and format conversation by round
        let agents_per_round = agents.len();
        for (idx, (agent_id, message)) in conversation_history.iter().enumerate() {
            let msg_round = (idx / agents_per_round) + 1;
            output.push_str(&format!("\n**[Round {}] {}:**\n\n{}\n", msg_round, agent_id, message));
        }

        // Add conclusion
        output.push_str("\n---\n\n");
        if consensus_reached {
            output.push_str(&format!(
                "✅ **Consensus reached** after {} rounds with {} participants\n",
                rounds_completed, agents.len()
            ));
            metadata.add_trace("Group chat ended with consensus".to_string());
        } else {
            output.push_str(&format!(
                "⏱️ **Discussion ended** after {} rounds (maximum reached)\n",
                self.max_rounds
            ));
            metadata.add_trace(format!(
                "Group chat ended after {} rounds (max reached)",
                self.max_rounds
            ));
        }

        Ok(OrchestratorResult::new(output, metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_consensus() {
        assert!(GroupChatExecutor::check_consensus("I think CONSENSUS_REACHED here"));
        assert!(GroupChatExecutor::check_consensus("consensus reached"));
        assert!(GroupChatExecutor::check_consensus("We agree on this"));
        assert!(GroupChatExecutor::check_consensus("Agreed!"));
        assert!(!GroupChatExecutor::check_consensus("I disagree"));
    }

    #[test]
    fn test_group_chat_requires_agents() {
        let executor = GroupChatExecutor::new(3, ResolutionStrategy::Consensus);
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(executor.execute(&[], "test input"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one agent"));
    }

    #[test]
    fn test_group_chat_requires_max_rounds() {
        let executor = GroupChatExecutor::new(0, ResolutionStrategy::Consensus);
        let rt = tokio::runtime::Runtime::new().unwrap();

        // Even with a mock agent, should fail validation
        let result = rt.block_on(executor.execute(&[], "test input"));
        assert!(result.is_err());
    }
}
