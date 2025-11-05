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

    /// Build conversation context from history
    fn build_context(history: &[(String, String)], initial_input: &str) -> String {
        let mut context = format!("Initial topic: {}\n\nConversation history:\n", initial_input);

        for (agent_id, message) in history {
            context.push_str(&format!("\n{}: {}\n", agent_id, message));
        }

        context.push_str("\nYour turn to respond. If you believe consensus has been reached, include 'CONSENSUS_REACHED' in your response:");
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
            for agent in agents.iter() {
                let context = if conversation_history.is_empty() {
                    // First message gets the original input
                    format!("{}\n\nPlease provide your perspective. If consensus is reached, include 'CONSENSUS_REACHED' in your response.", input)
                } else {
                    // Subsequent messages get conversation history
                    Self::build_context(&conversation_history, input)
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

        // Build final output: full conversation + last message
        let mut output = String::from("GROUP CHAT CONVERSATION:\n\n");
        for (agent_id, message) in &conversation_history {
            output.push_str(&format!("{}: {}\n\n", agent_id, message));
        }

        output.push_str("\n--- FINAL MESSAGE ---\n");
        if let Some((agent_id, last_message)) = conversation_history.last() {
            output.push_str(&format!("From {}: {}", agent_id, last_message));
        }

        if consensus_reached {
            metadata.add_trace("Group chat ended with consensus".to_string());
        } else {
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
