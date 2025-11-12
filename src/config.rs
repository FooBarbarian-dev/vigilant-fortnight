//! YAML configuration support for orchestrators
//!
//! This module provides convenience loading of orchestrator configurations from YAML files.
//! Note: This is a simplified implementation that demonstrates the structure. In practice,
//! you would need to handle API client initialization with credentials.

use crate::patterns::Pattern;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Configuration for a single agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Unique identifier for the agent
    pub id: String,
    /// Provider name (e.g., "openai", "anthropic", "cohere")
    pub provider: String,
    /// Model name (e.g., "gpt-4", "claude-3-opus")
    pub model: String,
    /// System prompt defining the agent's role
    pub system_prompt: String,
}

/// Orchestration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationConfig {
    /// The pattern to use
    #[serde(flatten)]
    pub pattern: Pattern,
}

/// Top-level orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    /// List of agents
    pub agents: Vec<AgentConfig>,
    /// Orchestration pattern and options
    pub orchestration: OrchestrationConfig,
}

impl OrchestratorConfig {
    /// Load configuration from a YAML file
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rig_patterns::config::OrchestratorConfig;
    ///
    /// # fn example() -> anyhow::Result<()> {
    /// let config = OrchestratorConfig::from_file("config.yaml")?;
    /// println!("Loaded {} agents", config.agents.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;

        Self::from_yaml(&content)
    }

    /// Parse configuration from a YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml).context("Failed to parse YAML configuration")
    }

    /// Convert configuration to YAML string
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).context("Failed to serialize configuration to YAML")
    }

    /// Save configuration to a YAML file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let yaml = self.to_yaml()?;
        std::fs::write(path.as_ref(), yaml)
            .with_context(|| format!("Failed to write config file: {:?}", path.as_ref()))?;
        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.agents.is_empty() {
            anyhow::bail!("Configuration must have at least one agent");
        }

        // Check for duplicate agent IDs
        let mut ids = std::collections::HashSet::new();
        for agent in &self.agents {
            if !ids.insert(&agent.id) {
                anyhow::bail!("Duplicate agent ID: {}", agent.id);
            }
        }

        // Validate pattern-specific requirements
        match &self.orchestration.pattern {
            Pattern::Sequential => {
                // No special requirements
            }
            Pattern::Concurrent { .. } => {
                // No special requirements
            }
            Pattern::GroupChat { max_rounds, .. } => {
                if *max_rounds == 0 {
                    anyhow::bail!("GroupChat pattern requires max_rounds > 0");
                }
            }
            Pattern::Handoff { max_hops } => {
                if *max_hops == 0 {
                    anyhow::bail!("Handoff pattern requires max_hops > 0");
                }
            }
            Pattern::Magentic { max_iterations } => {
                if *max_iterations == 0 {
                    anyhow::bail!("Magentic pattern requires max_iterations > 0");
                }
                if self.agents.len() < 2 {
                    anyhow::bail!("Magentic pattern requires at least 2 agents (manager + workers)");
                }
            }
        }

        Ok(())
    }

    /// Get the pattern from the configuration
    pub fn pattern(&self) -> &Pattern {
        &self.orchestration.pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::Aggregation;

    #[test]
    fn test_parse_sequential_config() {
        let yaml = r#"
agents:
  - id: summarizer
    provider: openai
    model: gpt-4
    system_prompt: "You summarize text"
  - id: critic
    provider: openai
    model: gpt-4
    system_prompt: "You critique summaries"

orchestration:
  type: sequential
"#;

        let config = OrchestratorConfig::from_yaml(yaml).unwrap();
        assert_eq!(config.agents.len(), 2);
        assert!(matches!(config.orchestration.pattern, Pattern::Sequential));
        config.validate().unwrap();
    }

    #[test]
    fn test_parse_concurrent_config() {
        let yaml = r#"
agents:
  - id: agent1
    provider: openai
    model: gpt-4
    system_prompt: "Agent 1"

orchestration:
  type: concurrent
  aggregation: vote
"#;

        let config = OrchestratorConfig::from_yaml(yaml).unwrap();
        assert!(matches!(
            config.orchestration.pattern,
            Pattern::Concurrent { aggregation: Aggregation::Vote }
        ));
    }

    #[test]
    fn test_parse_group_chat_config() {
        let yaml = r#"
agents:
  - id: agent1
    provider: openai
    model: gpt-4
    system_prompt: "Agent 1"

orchestration:
  type: group_chat
  max_rounds: 5
"#;

        let config = OrchestratorConfig::from_yaml(yaml).unwrap();
        if let Pattern::GroupChat { max_rounds } = config.orchestration.pattern {
            assert_eq!(max_rounds, 5);
        } else {
            panic!("Expected GroupChat pattern");
        }
    }

    #[test]
    fn test_validate_duplicate_ids() {
        let yaml = r#"
agents:
  - id: agent1
    provider: openai
    model: gpt-4
    system_prompt: "Agent 1"
  - id: agent1
    provider: openai
    model: gpt-4
    system_prompt: "Duplicate"

orchestration:
  type: sequential
"#;

        let config = OrchestratorConfig::from_yaml(yaml).unwrap();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_roundtrip() {
        let config = OrchestratorConfig {
            agents: vec![AgentConfig {
                id: "test".to_string(),
                provider: "openai".to_string(),
                model: "gpt-4".to_string(),
                system_prompt: "Test agent".to_string(),
            }],
            orchestration: OrchestrationConfig {
                pattern: Pattern::Sequential,
            },
        };

        let yaml = config.to_yaml().unwrap();
        let parsed = OrchestratorConfig::from_yaml(&yaml).unwrap();
        assert_eq!(parsed.agents.len(), 1);
        assert_eq!(parsed.agents[0].id, "test");
    }
}
