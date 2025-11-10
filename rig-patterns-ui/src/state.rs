//! Shared application state and models

use rig_patterns::{Aggregation, Pattern};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Agent configuration from the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub provider: String,
    pub model: String,
    pub system_prompt: String,
}

/// Pattern configuration from the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PatternConfig {
    Sequential,
    Concurrent { aggregation: Aggregation },
    GroupChat { max_rounds: usize },
    Handoff { max_hops: usize },
    Magentic { max_iterations: usize },
}

impl From<PatternConfig> for Pattern {
    fn from(config: PatternConfig) -> Self {
        match config {
            PatternConfig::Sequential => Pattern::Sequential,
            PatternConfig::Concurrent { aggregation } => Pattern::Concurrent { aggregation },
            PatternConfig::GroupChat { max_rounds } => Pattern::GroupChat { max_rounds },
            PatternConfig::Handoff { max_hops } => Pattern::Handoff { max_hops },
            PatternConfig::Magentic { max_iterations } => Pattern::Magentic { max_iterations },
        }
    }
}

/// Configuration for a single pattern execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternExecutionConfig {
    pub pattern: PatternConfig,
    pub agents: Vec<AgentConfig>,
}

/// Request to compare all patterns (executes all in parallel)
#[derive(Debug, Deserialize)]
pub struct CompareRequest {
    /// The root input prompt that all patterns will process
    pub input: String,
    /// Configuration for each pattern (if not provided, uses default agents for all)
    pub pattern_configs: Option<std::collections::HashMap<String, PatternExecutionConfig>>,
}

/// Unified execution request for backward compatibility and single pattern execution
#[derive(Debug, Deserialize)]
pub struct ExecuteRequest {
    pub agents: Vec<AgentConfig>,
    pub pattern: PatternConfig,
    pub input: String,
}

/// Response from pattern execution
#[derive(Debug, Serialize)]
pub struct ExecuteResponse {
    pub output: String,
    pub execution_trace: Vec<String>,
    pub metadata: serde_json::Value,
    pub pattern_name: String,
    pub duration_ms: u128,
}

/// Response with all pattern results
#[derive(Debug, Serialize)]
pub struct CompareResponse {
    pub results: std::collections::HashMap<String, ExecuteResponse>,
}

/// Events streamed via WebSocket
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExecutionEvent {
    #[allow(dead_code)]
    AgentStart {
        pattern_id: String,
        agent_id: String,
        provider: String,
        timestamp: String,
    },
    AgentReceivesInput {
        pattern_id: String,
        agent_id: String,
        provider: String,
        input: String,
        timestamp: String,
    },
    AgentThinking {
        pattern_id: String,
        agent_id: String,
        provider: String,
        timestamp: String,
    },
    AgentResponds {
        pattern_id: String,
        agent_id: String,
        provider: String,
        response: String,
        timestamp: String,
    },
    #[allow(dead_code)]
    AgentComplete {
        pattern_id: String,
        agent_id: String,
        provider: String,
        output_preview: String,
        timestamp: String,
    },
    #[allow(dead_code)]
    AgentError {
        pattern_id: String,
        agent_id: String,
        error: String,
        timestamp: String,
    },
    AgentHandoff {
        pattern_id: String,
        from_agent: String,
        to_agent: String,
        from_provider: String,
        to_provider: String,
        message: String,
        timestamp: String,
    },
    PatternStep {
        pattern_id: String,
        message: String,
        timestamp: String,
    },
    ConversationMessage {
        pattern_id: String,
        from: String,
        provider: String,
        message: String,
        message_type: String, // "input", "output", "handoff", "consensus"
        timestamp: String,
    },
    PatternComplete {
        pattern_id: String,
        output: String,
        metadata: serde_json::Value,
        timestamp: String,
    },
    PatternError {
        pattern_id: String,
        error: String,
        timestamp: String,
    },
}

/// Preset configuration for common use cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetConfig {
    pub name: String,
    pub description: String,
    pub agents: Vec<AgentConfig>,
    pub pattern: PatternConfig,
    pub sample_input: String,
}

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub presets: Arc<RwLock<Vec<PresetConfig>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            presets: Arc::new(RwLock::new(Self::default_presets())),
        }
    }

    fn default_presets() -> Vec<PresetConfig> {
        vec![
            PresetConfig {
                name: "Document Processing Pipeline".to_string(),
                description: "Sequential processing: extract → summarize → format".to_string(),
                agents: vec![
                    AgentConfig {
                        id: "extractor".to_string(),
                        provider: "openai".to_string(),
                        model: "gpt-4".to_string(),
                        system_prompt: "You extract key information from documents.".to_string(),
                    },
                    AgentConfig {
                        id: "summarizer".to_string(),
                        provider: "openai".to_string(),
                        model: "gpt-4".to_string(),
                        system_prompt: "You create concise summaries.".to_string(),
                    },
                    AgentConfig {
                        id: "formatter".to_string(),
                        provider: "openai".to_string(),
                        model: "gpt-4".to_string(),
                        system_prompt: "You format content into professional documents.".to_string(),
                    },
                ],
                pattern: PatternConfig::Sequential,
                sample_input: "Process this quarterly report and create an executive summary.".to_string(),
            },
            PresetConfig {
                name: "Multi-Perspective Code Review".to_string(),
                description: "Concurrent review from multiple experts with consensus".to_string(),
                agents: vec![
                    AgentConfig {
                        id: "security-reviewer".to_string(),
                        provider: "openai".to_string(),
                        model: "gpt-4".to_string(),
                        system_prompt: "You review code for security vulnerabilities.".to_string(),
                    },
                    AgentConfig {
                        id: "performance-reviewer".to_string(),
                        provider: "openai".to_string(),
                        model: "gpt-4".to_string(),
                        system_prompt: "You review code for performance issues.".to_string(),
                    },
                    AgentConfig {
                        id: "style-reviewer".to_string(),
                        provider: "openai".to_string(),
                        model: "gpt-4".to_string(),
                        system_prompt: "You review code for style and best practices.".to_string(),
                    },
                ],
                pattern: PatternConfig::Concurrent {
                    aggregation: Aggregation::Combine,
                },
                sample_input: "Review this authentication function for issues.".to_string(),
            },
            PresetConfig {
                name: "Customer Support Triage".to_string(),
                description: "Handoff to specialist agents based on request type".to_string(),
                agents: vec![
                    AgentConfig {
                        id: "triage".to_string(),
                        provider: "openai".to_string(),
                        model: "gpt-4".to_string(),
                        system_prompt: "You triage customer requests and route to specialists. Use HANDOFF:agent_id to route.".to_string(),
                    },
                    AgentConfig {
                        id: "technical".to_string(),
                        provider: "openai".to_string(),
                        model: "gpt-4".to_string(),
                        system_prompt: "You handle technical support issues.".to_string(),
                    },
                    AgentConfig {
                        id: "billing".to_string(),
                        provider: "openai".to_string(),
                        model: "gpt-4".to_string(),
                        system_prompt: "You handle billing and payment issues.".to_string(),
                    },
                    AgentConfig {
                        id: "account".to_string(),
                        provider: "openai".to_string(),
                        model: "gpt-4".to_string(),
                        system_prompt: "You handle account management issues.".to_string(),
                    },
                ],
                pattern: PatternConfig::Handoff { max_hops: 5 },
                sample_input: "I can't log in to my account and need help.".to_string(),
            },
            PresetConfig {
                name: "Research Synthesis".to_string(),
                description: "Manager coordinates researchers and synthesizes findings".to_string(),
                agents: vec![
                    AgentConfig {
                        id: "research-manager".to_string(),
                        provider: "openai".to_string(),
                        model: "gpt-4".to_string(),
                        system_prompt: "You manage research projects by breaking them into tasks.".to_string(),
                    },
                    AgentConfig {
                        id: "researcher".to_string(),
                        provider: "openai".to_string(),
                        model: "gpt-4".to_string(),
                        system_prompt: "You research topics thoroughly.".to_string(),
                    },
                    AgentConfig {
                        id: "fact-checker".to_string(),
                        provider: "openai".to_string(),
                        model: "gpt-4".to_string(),
                        system_prompt: "You verify facts and sources.".to_string(),
                    },
                ],
                pattern: PatternConfig::Magentic { max_iterations: 5 },
                sample_input: "Research the impact of AI on software development productivity.".to_string(),
            },
            PresetConfig {
                name: "Collaborative Writing".to_string(),
                description: "Group chat with agents refining content through discussion".to_string(),
                agents: vec![
                    AgentConfig {
                        id: "writer".to_string(),
                        provider: "openai".to_string(),
                        model: "gpt-4".to_string(),
                        system_prompt: "You write compelling content. Include CONSENSUS_REACHED when satisfied.".to_string(),
                    },
                    AgentConfig {
                        id: "editor".to_string(),
                        provider: "openai".to_string(),
                        model: "gpt-4".to_string(),
                        system_prompt: "You edit for clarity and correctness.".to_string(),
                    },
                    AgentConfig {
                        id: "fact-checker".to_string(),
                        provider: "openai".to_string(),
                        model: "gpt-4".to_string(),
                        system_prompt: "You verify accuracy and suggest improvements.".to_string(),
                    },
                ],
                pattern: PatternConfig::GroupChat { max_rounds: 4 },
                sample_input: "Write a blog post about Rust async programming.".to_string(),
            },
        ]
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
