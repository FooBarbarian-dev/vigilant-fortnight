//! Agent wrapper providing a simplified interface over rig-core models

use anyhow::Result;
use rig::completion::{CompletionModel, CompletionRequestBuilder, ModelChoice};

/// Supported LLM providers
#[derive(Clone)]
enum ModelProvider {
    OpenAI(rig::providers::openai::CompletionModel),
    Anthropic(rig::providers::anthropic::completion::CompletionModel),
    Cohere(rig::providers::cohere::CompletionModel),
}

/// A simplified agent wrapper around rig-core completion models
#[derive(Clone)]
pub struct Agent {
    /// Unique identifier for this agent
    pub id: String,
    /// The underlying completion model
    model: ModelProvider,
    /// System prompt that defines the agent's role
    pub system_prompt: String,
}

impl Agent {
    /// Create a new agent with a unique ID, model, and system prompt
    ///
    /// Note: For most use cases, prefer using `from_openai()`, `from_anthropic()`,
    /// `from_cohere()`, or `from_env()` helper methods instead.
    fn new_with_provider(id: &str, model: ModelProvider, prompt: &str) -> Self {
        Self {
            id: id.to_string(),
            model,
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

        // Create a completion request with system message and user input
        let full_prompt = format!("{}\n\nUser: {}", self.system_prompt, input);

        // Use rig's completion API with proper request builder
        let response_text = match &self.model {
            ModelProvider::OpenAI(model) => {
                let request = CompletionRequestBuilder::new(model.clone(), full_prompt)
                    .temperature(1.0) // GPT-5 requires explicit temperature (default is 1.0)
                    .build();
                let response = model.completion(request).await
                    .map_err(|e| anyhow::anyhow!("OpenAI completion failed: {}", e))?;

                // Extract text from response
                match response.choice {
                    ModelChoice::Message(text) => text,
                    ModelChoice::ToolCall(name, _) => {
                        anyhow::bail!("Unexpected tool call: {}", name)
                    }
                }
            }
            ModelProvider::Anthropic(model) => {
                let request = CompletionRequestBuilder::new(model.clone(), full_prompt)
                    .max_tokens(4096) // Anthropic requires max_tokens to be set
                    .build();
                let response = model.completion(request).await
                    .map_err(|e| anyhow::anyhow!("Anthropic completion failed: {}", e))?;

                // Extract text from response
                match response.choice {
                    ModelChoice::Message(text) => text,
                    ModelChoice::ToolCall(name, _) => {
                        anyhow::bail!("Unexpected tool call: {}", name)
                    }
                }
            }
            ModelProvider::Cohere(model) => {
                let request = CompletionRequestBuilder::new(model.clone(), full_prompt)
                    .build();
                let response = model.completion(request).await
                    .map_err(|e| anyhow::anyhow!("Cohere completion failed: {}", e))?;

                // Extract text from response
                match response.choice {
                    ModelChoice::Message(text) => text,
                    ModelChoice::ToolCall(name, _) => {
                        anyhow::bail!("Unexpected tool call: {}", name)
                    }
                }
            }
        };

        tracing::debug!(agent_id = %self.id, response_len = response_text.len(), "Agent responded");

        Ok(response_text)
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
    pub fn from_openai(id: &str, api_key: &str, model: &str, system_prompt: &str) -> Result<Self> {
        let client = rig::providers::openai::Client::new(api_key);
        let completion_model = client.completion_model(model);
        Ok(Self::new_with_provider(
            id,
            ModelProvider::OpenAI(completion_model),
            system_prompt,
        ))
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
        // Anthropic client requires: api_key, base_url, betas (optional), version
        let client = rig::providers::anthropic::Client::new(
            api_key,
            "https://api.anthropic.com", // base URL
            None,                          // no beta features
            "2023-06-01",                 // API version
        );
        let completion_model = client.completion_model(model);
        Ok(Self::new_with_provider(
            id,
            ModelProvider::Anthropic(completion_model),
            system_prompt,
        ))
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
    pub fn from_cohere(id: &str, api_key: &str, model: &str, system_prompt: &str) -> Result<Self> {
        let client = rig::providers::cohere::Client::new(api_key);
        let completion_model = client.completion_model(model);
        Ok(Self::new_with_provider(
            id,
            ModelProvider::Cohere(completion_model),
            system_prompt,
        ))
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
    pub fn from_env(id: &str, provider: &str, model: &str, system_prompt: &str) -> Result<Self> {
        match provider.to_lowercase().as_str() {
            "openai" => {
                let api_key = std::env::var("OPENAI_API_KEY")
                    .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY environment variable not set"))?;
                Self::from_openai(id, &api_key, model, system_prompt)
            }
            "anthropic" => {
                let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
                    anyhow::anyhow!("ANTHROPIC_API_KEY environment variable not set")
                })?;
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
