//! Agent wrapper providing a simplified interface over rig-core models

use anyhow::Result;
use rig::completion::CompletionModel;
use rig::completion::Prompt;

/// A simplified agent wrapper around rig-core completion models
#[derive(Clone)]
pub struct Agent {
    /// Unique identifier for this agent
    pub id: String,
    /// The underlying completion model from rig-core
    model: Box<dyn CompletionModel>,
    /// System prompt that defines the agent's role
    pub system_prompt: String,
}

impl Agent {
    /// Create a new agent with a unique ID, model, and system prompt
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this agent
    /// * `model` - Any type implementing rig-core's CompletionModel trait
    /// * `prompt` - System prompt defining the agent's role and behavior
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rig_patterns::Agent;
    /// # async fn example() -> anyhow::Result<()> {
    /// // let model = ...; // some rig CompletionModel
    /// // let agent = Agent::new("summarizer", model, "You are a helpful summarizer");
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(id: &str, model: impl CompletionModel + 'static, prompt: &str) -> Self {
        Self {
            id: id.to_string(),
            model: Box::new(model),
            system_prompt: prompt.to_string(),
        }
    }

    /// Send a prompt to the agent and get a response
    ///
    /// # Arguments
    ///
    /// * `input` - The user input/prompt to send to the agent
    ///
    /// # Returns
    ///
    /// The agent's text response
    pub async fn prompt(&self, input: &str) -> Result<String> {
        tracing::debug!(agent_id = %self.id, "Prompting agent");

        // Create a prompt with system message and user input
        let prompt = format!("{}\n\nUser: {}", self.system_prompt, input);

        // Use rig's completion API
        let response = self.model
            .completion(prompt, None)
            .await
            .map_err(|e| anyhow::anyhow!("Agent {} completion failed: {}", self.id, e))?;

        tracing::debug!(agent_id = %self.id, response_len = response.len(), "Agent responded");

        Ok(response)
    }

    /// Get the agent's ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the agent's system prompt
    pub fn system_prompt(&self) -> &str {
        &self.system_prompt
    }
}

impl std::fmt::Debug for Agent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Agent")
            .field("id", &self.id)
            .field("system_prompt", &self.system_prompt)
            .finish()
    }
}

// Note: We're not implementing Clone for the trait object directly,
// but we can provide a manual clone implementation if needed
// For now, we'll use the derived Clone which clones the Box
