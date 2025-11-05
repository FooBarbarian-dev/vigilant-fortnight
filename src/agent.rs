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

    /// Create an agent using OpenAI models
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this agent
    /// * `api_key` - OpenAI API key
    /// * `model` - Model name (e.g., "gpt-4", "gpt-4-turbo-preview", "gpt-3.5-turbo")
    /// * `system_prompt` - System prompt defining the agent's role
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rig_patterns::Agent;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let agent = Agent::from_openai(
    ///     "summarizer",
    ///     "sk-...",  // Your OpenAI API key
    ///     "gpt-4",
    ///     "You are a helpful summarizer"
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_openai(
        id: &str,
        api_key: &str,
        model: &str,
        system_prompt: &str,
    ) -> Result<Self> {
        let client = rig::providers::openai::Client::new(api_key);
        let completion_model = client.completion_model(model);
        Ok(Self::new(id, completion_model, system_prompt))
    }

    /// Create an agent using Anthropic Claude models
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this agent
    /// * `api_key` - Anthropic API key
    /// * `model` - Model name (e.g., "claude-3-opus-20240229", "claude-3-sonnet-20240229", "claude-3-haiku-20240307")
    /// * `system_prompt` - System prompt defining the agent's role
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rig_patterns::Agent;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let agent = Agent::from_anthropic(
    ///     "analyst",
    ///     "sk-ant-...",  // Your Anthropic API key
    ///     "claude-3-opus-20240229",
    ///     "You are a thorough analyst"
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_anthropic(
        id: &str,
        api_key: &str,
        model: &str,
        system_prompt: &str,
    ) -> Result<Self> {
        let client = rig::providers::anthropic::Client::new(api_key);
        let completion_model = client.completion_model(model);
        Ok(Self::new(id, completion_model, system_prompt))
    }

    /// Create an agent using Cohere models
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this agent
    /// * `api_key` - Cohere API key
    /// * `model` - Model name (e.g., "command", "command-light", "command-nightly")
    /// * `system_prompt` - System prompt defining the agent's role
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rig_patterns::Agent;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let agent = Agent::from_cohere(
    ///     "writer",
    ///     "...",  // Your Cohere API key
    ///     "command",
    ///     "You write clear content"
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_cohere(
        id: &str,
        api_key: &str,
        model: &str,
        system_prompt: &str,
    ) -> Result<Self> {
        let client = rig::providers::cohere::Client::new(api_key);
        let completion_model = client.completion_model(model);
        Ok(Self::new(id, completion_model, system_prompt))
    }

    /// Create an agent from environment variables
    ///
    /// This helper reads API keys from environment variables and creates an agent
    /// based on the specified provider.
    ///
    /// # Environment Variables
    ///
    /// * `OPENAI_API_KEY` - For OpenAI provider
    /// * `ANTHROPIC_API_KEY` - For Anthropic provider
    /// * `COHERE_API_KEY` - For Cohere provider
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this agent
    /// * `provider` - Provider name ("openai", "anthropic", or "cohere")
    /// * `model` - Model name
    /// * `system_prompt` - System prompt defining the agent's role
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rig_patterns::Agent;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// // Reads from OPENAI_API_KEY environment variable
    /// let agent = Agent::from_env(
    ///     "summarizer",
    ///     "openai",
    ///     "gpt-4",
    ///     "You are a helpful summarizer"
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_env(
        id: &str,
        provider: &str,
        model: &str,
        system_prompt: &str,
    ) -> Result<Self> {
        match provider.to_lowercase().as_str() {
            "openai" => {
                let api_key = std::env::var("OPENAI_API_KEY")
                    .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY environment variable not set"))?;
                Self::from_openai(id, &api_key, model, system_prompt)
            }
            "anthropic" => {
                let api_key = std::env::var("ANTHROPIC_API_KEY")
                    .map_err(|_| anyhow::anyhow!("ANTHROPIC_API_KEY environment variable not set"))?;
                Self::from_anthropic(id, &api_key, model, system_prompt)
            }
            "cohere" => {
                let api_key = std::env::var("COHERE_API_KEY")
                    .map_err(|_| anyhow::anyhow!("COHERE_API_KEY environment variable not set"))?;
                Self::from_cohere(id, &api_key, model, system_prompt)
            }
            _ => Err(anyhow::anyhow!(
                "Unknown provider '{}'. Supported providers: openai, anthropic, cohere",
                provider
            )),
        }
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
