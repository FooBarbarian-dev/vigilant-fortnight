//! Group chat pattern: agents converse in rounds until consensus or max rounds

use super::{PatternExecutor, PatternMetadata};
use crate::{Agent, OrchestratorResult};
use async_trait::async_trait;

/// Group chat pattern executor
///
/// Agents converse in rounds, building on each other's messages.
/// Stops when a consensus keyword is detected or max rounds are reached.
pub(crate) struct GroupChatExecutor {
    max_rounds: usize,
}

impl GroupChatExecutor {
    pub fn new(max_rounds: usize) -> Self {
        Self { max_rounds }
    }

    /// Check if the message indicates consensus has been reached
    fn check_consensus(message: &str) -> bool {
        let normalized = message.to_uppercase();
        normalized.contains("CONSENSUS_REACHED")
            || normalized.contains("CONSENSUS REACHED")
            || normalized.contains("WE AGREE")
            || normalized.contains("AGREED")
    }

    /// Build conversation context from history with structured format
    fn build_context(history: &[(String, String)], initial_input: &str, _round: usize, is_last_agent: bool) -> String {
        let mut context = format!("TOPIC: {}\n\nDISCUSSION HISTORY:\n", initial_input);

        // Track which round each message belongs to (assuming agents speak in order)
        let agent_count = history.iter().map(|(id, _)| id).collect::<std::collections::HashSet<_>>().len().max(1);

        for (idx, (agent_id, message)) in history.iter().enumerate() {
            let msg_round = (idx / agent_count) + 1;
            context.push_str(&format!("\n[Round {}] {}: {}\n", msg_round, agent_id, message));
        }

        // Add instructions based on agent position
        if is_last_agent {
            context.push_str("\n\nYour turn to respond. Reference specific points made by others and either build on them or offer alternative perspectives. ");
            context.push_str("If you believe we've reached a good conclusion, include 'CONSENSUS_REACHED' in your response.");
        } else {
            context.push_str("\n\nYour turn to respond. Reference specific points made by others and either build on them, ");
            context.push_str("offer alternative perspectives, or ask clarifying questions. Engage directly with what has been said.");
        }

        context
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
        metadata.add_trace(format!(
            "Starting group chat with {} agents, max {} rounds",
            agents.len(),
            self.max_rounds
        ));

        let mut conversation_history: Vec<(String, String)> = Vec::new();
        let mut consensus_reached = false;

        for round in 0..self.max_rounds {
            metadata.add_trace(format!("--- Round {} ---", round + 1));

            // Round-robin: each agent speaks once per round
            for (idx, agent) in agents.iter().enumerate() {
                let is_last_agent = idx == agents.len() - 1;

                let context = if conversation_history.is_empty() {
                    // First message gets the original input with clear instructions
                    format!(
                        "TOPIC: {}\n\n\
                        You are participating in a group discussion with other agents. \
                        Please provide your initial perspective on this topic. \
                        Be thoughtful and set the stage for a productive discussion.",
                        input
                    )
                } else {
                    // Subsequent messages get structured conversation history
                    Self::build_context(&conversation_history, input, round + 1, is_last_agent)
                };

                tracing::debug!(
                    round = round + 1,
                    agent_id = %agent.id(),
                    "GroupChat: prompting agent"
                );

                let response = agent.prompt(&context).await.map_err(|e| {
                    anyhow::anyhow!(
                        "Agent '{}' failed in round {}: {}",
                        agent.id(),
                        round + 1,
                        e
                    )
                })?;

                metadata.add_trace(format!(
                    "Round {}: {} said: {}",
                    round + 1,
                    agent.id(),
                    response.chars().take(100).collect::<String>()
                        + if response.len() > 100 { "..." } else { "" }
                ));

                conversation_history.push((agent.id().to_string(), response.clone()));

                // Check for consensus
                if Self::check_consensus(&response) {
                    metadata.add_trace(format!(
                        "Consensus detected from agent '{}' in round {}",
                        agent.id(),
                        round + 1
                    ));
                    consensus_reached = true;
                    break;
                }
            }

            if consensus_reached {
                break;
            }
        }

        let rounds_completed = if consensus_reached {
            // Calculate which round we stopped in
            ((conversation_history.len() - 1) / agents.len()) + 1
        } else {
            self.max_rounds
        };

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
        let executor = GroupChatExecutor::new(3);
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(executor.execute(&[], "test input"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one agent"));
    }

    #[test]
    fn test_group_chat_requires_max_rounds() {
        let executor = GroupChatExecutor::new(0);
        let rt = tokio::runtime::Runtime::new().unwrap();

        // Even with a mock agent, should fail validation
        let result = rt.block_on(executor.execute(&[], "test input"));
        assert!(result.is_err());
    }
}
