//! WebSocket handler for real-time execution streaming with parallel pattern comparison

use crate::state::{AgentConfig, CompareRequest, ExecuteRequest, ExecutionEvent, PatternConfig};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State as AxumState,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use rig_patterns::{Agent, Aggregation};
use std::sync::Arc;
use std::time::Duration;

/// Trait for sending execution events (abstracts over WebSocket and channel)
trait EventSender {
    async fn send(&mut self, event: ExecutionEvent) -> anyhow::Result<()>;
}

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    state: AxumState<Arc<crate::state::AppState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, _state: AxumState<Arc<crate::state::AppState>>) {
    tracing::info!("WebSocket connection established");
    let (mut sender, mut receiver) = socket.split();

    // Listen for execution requests
    while let Some(msg) = receiver.next().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            tracing::warn!("Client disconnected during message receive");
            return;
        };

        if let Message::Text(text) = msg {
            tracing::debug!("Received WebSocket message: {} bytes", text.len());

            // Try parsing as CompareRequest first (executes all patterns)
            match serde_json::from_str::<CompareRequest>(&text) {
                Ok(request) => {
                    tracing::info!(
                        "========== PATTERN COMPARISON START ==========\n\
                         Input: {}\n\
                         Executing all patterns in parallel",
                        request.input
                    );

                    // Execute all patterns in parallel
                    if let Err(e) = execute_all_patterns(&mut sender, request).await {
                        tracing::error!("Pattern comparison error: {}", e);
                    }

                    tracing::info!("========== PATTERN COMPARISON COMPLETE ==========");
                }
                Err(compare_err) => {
                    // Try parsing as ExecuteRequest for backward compatibility
                    match serde_json::from_str::<ExecuteRequest>(&text) {
                        Ok(request) => {
                            tracing::info!(
                                "========== SINGLE PATTERN EXECUTION START ==========\n\
                                 Pattern: {:?}\n\
                                 Agents: {}\n\
                                 Input: {}",
                                request.pattern,
                                request.agents.len(),
                                request.input
                            );

                            let pattern_id = format!("{:?}", request.pattern).to_lowercase();
                            if let Err(e) = execute_single_pattern(&mut sender, pattern_id, request.agents, request.pattern, request.input).await {
                                tracing::error!("Single pattern execution error: {}", e);
                            }

                            tracing::info!("========== SINGLE PATTERN EXECUTION COMPLETE ==========");
                        }
                        Err(execute_err) => {
                            tracing::error!("❌ Failed to parse as CompareRequest: {}", compare_err);
                            tracing::error!("❌ Failed to parse as ExecuteRequest: {}", execute_err);
                            tracing::error!("📄 JSON length: {} bytes", text.len());
                            tracing::error!("📄 JSON start: {}", &text[..text.len().min(500)]);
                            if text.len() > 500 {
                                tracing::error!("📄 JSON end: {}", &text[text.len().saturating_sub(300)..]);
                            }

                            let _ = send_event(
                                &mut sender,
                                ExecutionEvent::PatternError {
                                    pattern_id: "unknown".to_string(),
                                    error: format!("JSON parse error: {}", compare_err),
                                    timestamp: chrono::Utc::now().to_rfc3339(),
                                },
                            )
                            .await;
                        }
                    }
                }
            }
        }
    }

    tracing::info!("WebSocket connection closed");
}

/// Execute all patterns in parallel for comparison
async fn execute_all_patterns(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    request: CompareRequest,
) -> anyhow::Result<()> {
    tracing::info!("🚀 EXECUTING ALL PATTERNS IN PARALLEL");

    // Get default agents if not provided per-pattern
    let default_agents = vec![
        AgentConfig {
            id: "gatherer".to_string(),
            provider: "openai".to_string(),
            model: "gpt-5".to_string(),
            system_prompt: "You are an Information Researcher and Gatherer. When you receive a query, your mission is to comprehensively research the topic and compile all relevant information. Extract key facts, definitions, historical context, current state, and important nuances. Structure your output with clear sections: Overview, Key Facts, Context, and Details. Your goal is to provide a complete information foundation that enables deep analysis in the next stage. Be thorough and factual.".to_string(),
        },
        AgentConfig {
            id: "analyzer".to_string(),
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-5-20250929".to_string(),
            system_prompt: "You are an Expert Analyst and Critical Thinker. You receive researched information from the previous agent and perform sophisticated analysis. Synthesize the information, identify underlying patterns and connections, evaluate implications, assess strengths and limitations, and formulate actionable insights. Provide expert interpretation that goes beyond the raw facts. Structure your output with: Analysis, Key Insights, Implications, and Recommendations. Think deeply and critically.".to_string(),
        },
    ];

    // Define all patterns to execute
    let patterns = vec![
        ("sequential", PatternConfig::Sequential),
        ("concurrent", PatternConfig::Concurrent { aggregation: Aggregation::Combine }),
        ("group_chat", PatternConfig::GroupChat { max_rounds: 3 }),
        ("handoff", PatternConfig::Handoff { max_hops: 5 }),
        ("magentic", PatternConfig::Magentic { max_iterations: 5 }),
    ];

    // Create channels for each pattern to send events
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<ExecutionEvent>();

    // Spawn tasks for each pattern
    let mut tasks = Vec::new();
    for (pattern_id, pattern_config) in patterns {
        let tx = tx.clone();
        let input = request.input.clone();
        let agents = request.pattern_configs
            .as_ref()
            .and_then(|configs| configs.get(pattern_id))
            .map(|config| config.agents.clone())
            .unwrap_or_else(|| default_agents.clone());

        let pattern_id = pattern_id.to_string();
        let task = tokio::spawn(async move {
            tracing::info!("[{}] Starting pattern execution", pattern_id);
            if let Err(e) = execute_pattern_to_channel(tx, pattern_id.clone(), agents, pattern_config, input).await {
                tracing::error!("[{}] Pattern execution failed: {}", pattern_id, e);
            }
        });
        tasks.push(task);
    }

    // Drop the original sender so the channel closes when all tasks complete
    drop(tx);

    // Forward all events from the channel to the WebSocket
    while let Some(event) = rx.recv().await {
        send_event(sender, event).await?;
    }

    // Wait for all tasks to complete
    for task in tasks {
        let _ = task.await;
    }

    tracing::info!("All patterns completed");
    Ok(())
}

/// Execute a single pattern and send events to a channel
async fn execute_pattern_to_channel(
    tx: tokio::sync::mpsc::UnboundedSender<ExecutionEvent>,
    pattern_id: String,
    agents_config: Vec<AgentConfig>,
    pattern: PatternConfig,
    input: String,
) -> anyhow::Result<()> {
    // Create a wrapper sender that sends to the channel
    struct ChannelSender(tokio::sync::mpsc::UnboundedSender<ExecutionEvent>);

    impl EventSender for ChannelSender {
        async fn send(&mut self, event: ExecutionEvent) -> anyhow::Result<()> {
            self.0.send(event).map_err(|e| anyhow::anyhow!("Channel send error: {}", e))?;
            Ok(())
        }
    }

    let mut channel_sender = ChannelSender(tx);
    execute_pattern_impl(&mut channel_sender, pattern_id, agents_config, pattern, input).await
}

/// Execute a single pattern directly to WebSocket
async fn execute_single_pattern(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    pattern_id: String,
    agents_config: Vec<AgentConfig>,
    pattern: PatternConfig,
    input: String,
) -> anyhow::Result<()> {
    // Create a wrapper sender that sends directly to WebSocket
    struct WebSocketSender<'a>(&'a mut futures::stream::SplitSink<WebSocket, Message>);

    impl<'a> EventSender for WebSocketSender<'a> {
        async fn send(&mut self, event: ExecutionEvent) -> anyhow::Result<()> {
            send_event(self.0, event).await
        }
    }

    let mut ws_sender = WebSocketSender(sender);
    execute_pattern_impl(&mut ws_sender, pattern_id, agents_config, pattern, input).await
}

/// Timeout duration for agent LLM calls (60 seconds)
const AGENT_TIMEOUT: Duration = Duration::from_secs(60);

/// Helper function to call an agent with timeout
async fn call_agent_with_timeout(agent: &Agent, prompt: &str, agent_id: &str) -> anyhow::Result<String> {
    match tokio::time::timeout(AGENT_TIMEOUT, agent.prompt(prompt)).await {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(e)) => Err(anyhow::anyhow!("Agent {} failed: {}", agent_id, e)),
        Err(_) => Err(anyhow::anyhow!("Agent {} timed out after {} seconds", agent_id, AGENT_TIMEOUT.as_secs())),
    }
}

/// Core pattern execution logic (shared by both parallel and single execution)
async fn execute_pattern_impl<S>(
    sender: &mut S,
    pattern_id: String,
    agents_config: Vec<AgentConfig>,
    pattern: PatternConfig,
    input: String,
) -> anyhow::Result<()>
where
    S: EventSender,
{
    tracing::info!("[{}] 🚀 REAL EXECUTION - Making actual LLM API calls", pattern_id);

    // Create real agents from the config
    let agents: Vec<Agent> = agents_config
        .iter()
        .map(|cfg| {
            Agent::from_env(&cfg.id, &cfg.provider, &cfg.model, &cfg.system_prompt)
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    tracing::info!("[{}] Created {} agents successfully", pattern_id, agents.len());

    // Helper to get provider for an agent ID
    let get_provider = |agent_id: &str| -> String {
        agents_config
            .iter()
            .find(|cfg| cfg.id == agent_id)
            .map(|cfg| cfg.provider.clone())
            .unwrap_or_else(|| "unknown".to_string())
    };

    // Execute with real LLM calls and detailed streaming events
    match &pattern {
        PatternConfig::Sequential => {
            tracing::info!("[{}] Starting SEQUENTIAL pattern with {} agents", pattern_id, agents.len());
            let mut current_input = input.clone();

            for (idx, agent) in agents.iter().enumerate() {
                let agent_id = agent.id();
                let provider = get_provider(agent_id);
                tracing::info!("[{}][Sequential Step {}/{}] Agent: {} ({})", pattern_id, idx + 1, agents.len(), agent_id, provider);

                // Agent receives input
                sender.send(ExecutionEvent::AgentReceivesInput {
                    pattern_id: pattern_id.clone(),
                    agent_id: agent_id.to_string(),
                    provider: provider.clone(),
                    input: current_input.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }).await?;

                // Agent thinking
                sender.send(ExecutionEvent::AgentThinking {
                    pattern_id: pattern_id.clone(),
                    agent_id: agent_id.to_string(),
                    provider: provider.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }).await?;

                // *** REAL LLM CALL WITH TIMEOUT ***
                let response = call_agent_with_timeout(agent, &current_input, agent_id).await?;

                sender.send(ExecutionEvent::AgentResponds {
                    pattern_id: pattern_id.clone(),
                    agent_id: agent_id.to_string(),
                    provider: provider.clone(),
                    response: response.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }).await?;

                current_input = response;
            }

            tracing::info!("[{}] Sequential pattern completed", pattern_id);

            sender.send(ExecutionEvent::PatternComplete {
                pattern_id: pattern_id.clone(),
                output: current_input,
                metadata: serde_json::json!({
                    "pattern": "sequential",
                    "steps": agents.len(),
                }),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }).await?;
        }

        PatternConfig::Concurrent { aggregation } => {
            tracing::info!("[{}] Starting CONCURRENT pattern with {} agents", pattern_id, agents.len());

            // All agents receive the same input
            for agent in &agents {
                let agent_id = agent.id();
                let provider = get_provider(agent_id);
                sender.send(ExecutionEvent::AgentReceivesInput {
                    pattern_id: pattern_id.clone(),
                    agent_id: agent_id.to_string(),
                    provider: provider.clone(),
                    input: input.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }).await?;
            }

            // All thinking concurrently
            for agent in &agents {
                let agent_id = agent.id();
                let provider = get_provider(agent_id);
                sender.send(ExecutionEvent::AgentThinking {
                    pattern_id: pattern_id.clone(),
                    agent_id: agent_id.to_string(),
                    provider: provider.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }).await?;
            }

            // *** REAL CONCURRENT LLM CALLS WITH TIMEOUT ***
            let mut tasks = Vec::new();
            for agent in &agents {
                let input_clone = input.clone();
                let agent_clone = agent.clone();
                let agent_id = agent.id().to_string();
                tasks.push(tokio::spawn(async move {
                    let response = call_agent_with_timeout(&agent_clone, &input_clone, &agent_id).await?;
                    Ok::<(String, String), anyhow::Error>((agent_clone.id().to_string(), response))
                }));
            }

            // Wait for all responses
            let mut responses = Vec::new();
            for task in tasks {
                let (agent_id, response) = task.await??;
                let provider = get_provider(&agent_id);

                sender.send(ExecutionEvent::AgentResponds {
                    pattern_id: pattern_id.clone(),
                    agent_id: agent_id.clone(),
                    provider: provider.clone(),
                    response: response.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }).await?;

                responses.push(response);
            }

            sender.send(ExecutionEvent::PatternStep {
                pattern_id: pattern_id.clone(),
                message: format!("Aggregating results using {:?} strategy...", aggregation),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }).await?;

            // Simple aggregation
            let aggregated = responses.join("\n\n---\n\n");

            tracing::info!("[{}] Concurrent pattern completed", pattern_id);

            sender.send(ExecutionEvent::PatternComplete {
                pattern_id: pattern_id.clone(),
                output: aggregated,
                metadata: serde_json::json!({
                    "pattern": "concurrent",
                    "agents": agents.len(),
                    "aggregation": format!("{:?}", aggregation),
                }),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }).await?;
        }

        PatternConfig::GroupChat { max_rounds } => {
            tracing::info!("[{}] Starting GROUP_CHAT pattern with {} agents", pattern_id, agents.len());
            let rounds = *max_rounds;

            // Store conversation as structured turns: Vec<(round, agent_id, message)>
            let mut conversation_turns: Vec<(usize, String, String)> = Vec::new();

            for round in 1..=rounds {
                sender.send(ExecutionEvent::PatternStep {
                    pattern_id: pattern_id.clone(),
                    message: format!("🔄 Discussion Round {} of {}", round, rounds),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }).await?;

                for (idx, agent) in agents.iter().enumerate() {
                    let agent_id = agent.id();
                    let provider = get_provider(agent_id);
                    let is_first_message = conversation_turns.is_empty();
                    let is_last_agent = idx == agents.len() - 1;

                    // Build structured conversation context
                    let prompt = if is_first_message {
                        // First agent in first round: introduce the topic
                        format!(
                            "TOPIC: {}\n\n\
                            You are participating in a group discussion with other agents. \
                            Please provide your initial perspective on this topic. \
                            Be thoughtful and set the stage for a productive discussion.",
                            input
                        )
                    } else {
                        // Build conversation history with clear structure
                        let mut context = format!("TOPIC: {}\n\nDISCUSSION HISTORY:\n", input);

                        for (r, speaker, msg) in &conversation_turns {
                            context.push_str(&format!("\n[Round {}] {}: {}\n", r, speaker, msg));
                        }

                        // Instructions for responding
                        let instructions = if is_last_agent && round == rounds {
                            "\n\nYour turn to respond. This is the final round - please provide a concluding response. \
                            Reference specific points made by others and either build on them or offer alternative perspectives. \
                            If you believe we've reached a good conclusion, include 'CONSENSUS_REACHED' in your response."
                        } else if is_last_agent {
                            "\n\nYour turn to respond. Reference specific points made by others and either build on them or offer alternative perspectives. \
                            If you believe we've reached a good conclusion, include 'CONSENSUS_REACHED' in your response."
                        } else {
                            "\n\nYour turn to respond. Reference specific points made by others and either build on them, \
                            offer alternative perspectives, or ask clarifying questions. Engage directly with what has been said."
                        };

                        context.push_str(instructions);
                        context
                    };

                    sender.send(ExecutionEvent::AgentReceivesInput {
                        pattern_id: pattern_id.clone(),
                        agent_id: agent_id.to_string(),
                        provider: provider.clone(),
                        input: prompt.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    }).await?;

                    sender.send(ExecutionEvent::AgentThinking {
                        pattern_id: pattern_id.clone(),
                        agent_id: agent_id.to_string(),
                        provider: provider.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    }).await?;

                    // *** REAL LLM CALL WITH TIMEOUT ***
                    let response = call_agent_with_timeout(agent, &prompt, agent_id).await?;

                    sender.send(ExecutionEvent::ConversationMessage {
                        pattern_id: pattern_id.clone(),
                        from: agent_id.to_string(),
                        provider: provider.clone(),
                        message: response.clone(),
                        message_type: if response.contains("CONSENSUS_REACHED") {
                            "consensus".to_string()
                        } else {
                            "output".to_string()
                        },
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    }).await?;

                    // Add turn to history
                    conversation_turns.push((round, agent_id.to_string(), response.clone()));

                    // Check for consensus
                    if response.contains("CONSENSUS_REACHED") {
                        tracing::info!("[{}] Consensus reached at round {}", pattern_id, round);

                        // Build final output with structured format
                        let mut final_output = format!("GROUP CHAT DISCUSSION\n\nTopic: {}\n\nConversation:\n", input);
                        for (r, speaker, msg) in &conversation_turns {
                            final_output.push_str(&format!("\n[Round {}] **{}**: {}\n", r, speaker, msg));
                        }
                        final_output.push_str(&format!("\n✅ Consensus reached after {} rounds with {} participants", round, agents.len()));

                        sender.send(ExecutionEvent::PatternComplete {
                            pattern_id: pattern_id.clone(),
                            output: final_output,
                            metadata: serde_json::json!({
                                "pattern": "group_chat",
                                "rounds": round,
                                "participants": agents.len(),
                                "consensus": true,
                            }),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        }).await?;
                        return Ok(());
                    }
                }
            }

            tracing::info!("[{}] Group chat completed after {} rounds", pattern_id, rounds);

            // Build final output with structured format
            let mut final_output = format!("GROUP CHAT DISCUSSION\n\nTopic: {}\n\nConversation:\n", input);
            for (r, speaker, msg) in &conversation_turns {
                final_output.push_str(&format!("\n[Round {}] **{}**: {}\n", r, speaker, msg));
            }
            final_output.push_str(&format!("\n⏱️ Discussion ended after {} rounds (maximum reached)", rounds));

            sender.send(ExecutionEvent::PatternComplete {
                pattern_id: pattern_id.clone(),
                output: final_output,
                metadata: serde_json::json!({
                    "pattern": "group_chat",
                    "rounds": rounds,
                    "participants": agents.len(),
                    "consensus": false,
                }),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }).await?;
        }

        PatternConfig::Handoff { max_hops } => {
            tracing::info!("[{}] Starting HANDOFF pattern with {} agents", pattern_id, agents.len());
            let mut current_input = input.clone();
            let mut hops = 0;

            for (idx, agent) in agents.iter().enumerate() {
                if hops >= *max_hops {
                    break;
                }

                let agent_id = agent.id();
                let provider = get_provider(agent_id);

                sender.send(ExecutionEvent::AgentReceivesInput {
                    pattern_id: pattern_id.clone(),
                    agent_id: agent_id.to_string(),
                    provider: provider.clone(),
                    input: current_input.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }).await?;

                sender.send(ExecutionEvent::AgentThinking {
                    pattern_id: pattern_id.clone(),
                    agent_id: agent_id.to_string(),
                    provider: provider.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }).await?;

                // *** REAL LLM CALL WITH TIMEOUT ***
                let prompt = if idx < agents.len() - 1 {
                    format!(
                        "{}\n\nProcess this task. If you need to hand off to another agent, include 'HANDOFF:agent_id' in your response.",
                        current_input
                    )
                } else {
                    format!("{}\n\nComplete this task and provide the final result.", current_input)
                };

                let response = call_agent_with_timeout(agent, &prompt, agent_id).await?;

                // Check for handoff
                if response.contains("HANDOFF:") && idx < agents.len() - 1 {
                    let next_agent = &agents[idx + 1];
                    let next_agent_id = next_agent.id();
                    let next_provider = get_provider(next_agent_id);

                    sender.send(ExecutionEvent::AgentHandoff {
                        pattern_id: pattern_id.clone(),
                        from_agent: agent_id.to_string(),
                        to_agent: next_agent_id.to_string(),
                        from_provider: provider.clone(),
                        to_provider: next_provider,
                        message: response.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    }).await?;
                    current_input = response;
                } else {
                    sender.send(ExecutionEvent::AgentResponds {
                        pattern_id: pattern_id.clone(),
                        agent_id: agent_id.to_string(),
                        provider: provider.clone(),
                        response: response.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    }).await?;
                    current_input = response;
                    break;
                }

                hops += 1;
            }

            tracing::info!("[{}] Handoff pattern completed after {} hops", pattern_id, hops);

            sender.send(ExecutionEvent::PatternComplete {
                pattern_id: pattern_id.clone(),
                output: current_input,
                metadata: serde_json::json!({
                    "pattern": "handoff",
                    "hops": hops,
                }),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }).await?;
        }

        PatternConfig::Magentic { max_iterations } => {
            tracing::info!("[{}] Starting MAGENTIC pattern with {} agents", pattern_id, agents.len());

            if agents.is_empty() {
                return Err(anyhow::anyhow!("Magentic pattern requires at least one agent (manager)"));
            }

            // First agent is the manager
            let manager = &agents[0];
            let workers = &agents[1..];
            let manager_id = manager.id();
            let manager_provider = get_provider(manager_id);

            // Manager receives input and creates task breakdown
            sender.send(ExecutionEvent::AgentReceivesInput {
                pattern_id: pattern_id.clone(),
                agent_id: manager_id.to_string(),
                provider: manager_provider.clone(),
                input: input.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }).await?;

            sender.send(ExecutionEvent::AgentThinking {
                pattern_id: pattern_id.clone(),
                agent_id: manager_id.to_string(),
                provider: manager_provider.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }).await?;

            // *** REAL LLM CALL WITH TIMEOUT - Manager breaks down tasks ***
            let task_breakdown_prompt = format!(
                "{}\n\nBreak this down into 2-4 specific subtasks. List each task on a new line starting with '- '.",
                input
            );
            let task_breakdown = call_agent_with_timeout(manager, &task_breakdown_prompt, manager_id).await?;

            sender.send(ExecutionEvent::AgentResponds {
                pattern_id: pattern_id.clone(),
                agent_id: manager_id.to_string(),
                provider: manager_provider.clone(),
                response: task_breakdown.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }).await?;

            // Parse tasks from response
            let tasks: Vec<String> = task_breakdown
                .lines()
                .filter(|line| line.trim().starts_with("- "))
                .map(|line| line.trim_start_matches("- ").trim().to_string())
                .collect();

            let mut task_results = Vec::new();

            // Assign tasks to workers
            for (idx, task) in tasks.iter().take(*max_iterations).enumerate() {
                sender.send(ExecutionEvent::PatternStep {
                    pattern_id: pattern_id.clone(),
                    message: format!("📋 Assigning task {}/{}: {}", idx + 1, tasks.len(), task),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }).await?;

                if !workers.is_empty() {
                    // Round-robin task assignment to workers
                    let worker = &workers[idx % workers.len()];
                    let worker_id = worker.id();
                    let worker_provider = get_provider(worker_id);

                    sender.send(ExecutionEvent::AgentReceivesInput {
                        pattern_id: pattern_id.clone(),
                        agent_id: worker_id.to_string(),
                        provider: worker_provider.clone(),
                        input: task.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    }).await?;

                    sender.send(ExecutionEvent::AgentThinking {
                        pattern_id: pattern_id.clone(),
                        agent_id: worker_id.to_string(),
                        provider: worker_provider.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    }).await?;

                    // *** REAL LLM CALL WITH TIMEOUT - Worker completes task ***
                    let result = call_agent_with_timeout(worker, task, worker_id).await?;

                    sender.send(ExecutionEvent::AgentResponds {
                        pattern_id: pattern_id.clone(),
                        agent_id: worker_id.to_string(),
                        provider: worker_provider.clone(),
                        response: result.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    }).await?;

                    task_results.push(format!("Task: {}\nResult: {}", task, result));
                }
            }

            // Manager synthesizes results
            sender.send(ExecutionEvent::PatternStep {
                pattern_id: pattern_id.clone(),
                message: "Manager synthesizing all results...".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }).await?;

            sender.send(ExecutionEvent::AgentThinking {
                pattern_id: pattern_id.clone(),
                agent_id: manager_id.to_string(),
                provider: manager_provider.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }).await?;

            // *** REAL LLM CALL - Manager synthesizes ***
            let synthesis_prompt = format!(
                "Synthesize these task results into a cohesive final answer:\n\n{}",
                task_results.join("\n\n")
            );
            let final_output = manager.prompt(&synthesis_prompt).await?;

            tracing::info!("[{}] Magentic pattern completed", pattern_id);

            sender.send(ExecutionEvent::PatternComplete {
                pattern_id: pattern_id.clone(),
                output: final_output,
                metadata: serde_json::json!({
                    "pattern": "magentic",
                    "tasks_completed": tasks.len(),
                    "workers_used": workers.len(),
                }),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }).await?;
        }
    }

    Ok(())
}

/// Helper to send an event over WebSocket
async fn send_event(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    event: ExecutionEvent,
) -> anyhow::Result<()> {
    let json = serde_json::to_string(&event)?;
    sender.send(Message::Text(json)).await?;
    Ok(())
}
