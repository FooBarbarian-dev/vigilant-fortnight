//! WebSocket handler for real-time execution streaming

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
            if let Ok(request) = serde_json::from_str::<CompareRequest>(&text) {
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
            // Fall back to single pattern execution for backward compatibility
            else if let Ok(request) = serde_json::from_str::<ExecuteRequest>(&text) {
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
            } else {
                tracing::error!("Failed to parse request - Raw text: {}", &text[..text.len().min(200)]);
                let _ = send_event(
                    &mut sender,
                    ExecutionEvent::PatternError {
                        pattern_id: "unknown".to_string(),
                        error: "Invalid request format".to_string(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await;
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
            id: "agent1".to_string(),
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            system_prompt: "You are a helpful AI assistant.".to_string(),
        },
        AgentConfig {
            id: "agent2".to_string(),
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            system_prompt: "You are an expert analyzer.".to_string(),
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

    // Create real agents from the request
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
                tracing::debug!("[{}]  → Agent {} receives input: {}...", pattern_id, agent_id, &current_input[..current_input.len().min(80)]);
                sender.send(ExecutionEvent::AgentReceivesInput {
                    pattern_id: pattern_id.clone(),
                    agent_id: agent_id.to_string(),
                    provider: provider.clone(),
                    input: current_input.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }).await?;

                // Agent thinking
                tracing::debug!("[{}]  → Agent {} is thinking...", pattern_id, agent_id);
                sender.send(ExecutionEvent::AgentThinking {
                    pattern_id: pattern_id.clone(),
                    agent_id: agent_id.to_string(),
                    provider: provider.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }).await?;

                // *** REAL LLM CALL ***
                let response = agent.prompt(&current_input).await?;

                tracing::debug!("[{}]  → Agent {} responds: {}...", pattern_id, agent_id, &response[..response.len().min(80)]);
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
            tracing::info!("Starting CONCURRENT pattern with {} agents", agents.len());

            // All agents receive the same input
            for agent in &agents {
                send_event(
                    sender,
                    ExecutionEvent::AgentReceivesInput {
                        agent_id: agent.id().to_string(),
                        input: request.input.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;
            }

            // All thinking concurrently
            for agent in &agents {
                send_event(
                    sender,
                    ExecutionEvent::AgentThinking {
                        agent_id: agent.id().to_string(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;
            }

            // *** REAL CONCURRENT LLM CALLS ***
            let mut tasks = Vec::new();
            for agent in &agents {
                let input = request.input.clone();
                let agent_clone = agent.clone();
                tasks.push(tokio::spawn(async move {
                    let response = agent_clone.prompt(&input).await?;
                    Ok::<(String, String), anyhow::Error>((agent_clone.id().to_string(), response))
                }));
            }

            // Wait for all responses
            let mut responses = Vec::new();
            for task in tasks {
                let (agent_id, response) = task.await??;

                send_event(
                    sender,
                    ExecutionEvent::AgentResponds {
                        agent_id: agent_id.clone(),
                        response: response.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;

                responses.push(response);
            }

            send_event(
                sender,
                ExecutionEvent::PatternStep {
                    message: format!("Aggregating results using {:?} strategy...", aggregation),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;

            // Simple aggregation for now
            let aggregated = responses.join("\n\n---\n\n");

            tracing::info!("Concurrent pattern completed");

            send_event(
                sender,
                ExecutionEvent::PatternComplete {
                    output: aggregated,
                    metadata: serde_json::json!({
                        "pattern": "concurrent",
                        "agents": agents.len(),
                        "aggregation": format!("{:?}", aggregation),
                    }),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;
        }

        PatternConfig::GroupChat { max_rounds } => {
            tracing::info!("Starting GROUP_CHAT pattern with {} agents", agents.len());
            let rounds = *max_rounds;
            let mut conversation_history = format!("Initial prompt: {}", request.input);

            for round in 1..=rounds {
                send_event(
                    sender,
                    ExecutionEvent::PatternStep {
                        message: format!("🔄 Discussion Round {} of {}", round, rounds),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;

                for agent in &agents {
                    // Agent receives conversation history
                    send_event(
                        sender,
                        ExecutionEvent::AgentReceivesInput {
                            agent_id: agent.id().to_string(),
                            input: conversation_history.clone(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    )
                    .await?;

                    send_event(
                        sender,
                        ExecutionEvent::AgentThinking {
                            agent_id: agent.id().to_string(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    )
                    .await?;

                    // *** REAL LLM CALL ***
                    let prompt = format!("{}\n\nRespond to the discussion. Include CONSENSUS_REACHED in your response if you believe we've reached a good conclusion.", conversation_history);
                    let response = agent.prompt(&prompt).await?;

                    send_event(
                        sender,
                        ExecutionEvent::ConversationMessage {
                            from: agent.id().to_string(),
                            message: response.clone(),
                            message_type: if response.contains("CONSENSUS_REACHED") {
                                "consensus".to_string()
                            } else {
                                "output".to_string()
                            },
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    )
                    .await?;

                    conversation_history.push_str(&format!("\n{}: {}", agent.id(), response));

                    // Check for consensus
                    if response.contains("CONSENSUS_REACHED") {
                        tracing::info!("Consensus reached at round {}", round);
                        send_event(
                            sender,
                            ExecutionEvent::PatternComplete {
                                output: conversation_history,
                                metadata: serde_json::json!({
                                    "pattern": "group_chat",
                                    "rounds": round,
                                    "participants": agents.len(),
                                }),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            },
                        )
                        .await?;
                        return Ok(());
                    }
                }
            }

            tracing::info!("Group chat completed after {} rounds", rounds);

            send_event(
                sender,
                ExecutionEvent::PatternComplete {
                    output: conversation_history,
                    metadata: serde_json::json!({
                        "pattern": "group_chat",
                        "rounds": rounds,
                        "participants": agents.len(),
                    }),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;
        }

        PatternConfig::Handoff { max_hops } => {
            tracing::info!("Starting HANDOFF pattern with {} agents", agents.len());
            let mut current_input = request.input.clone();
            let mut hops = 0;

            for (idx, agent) in agents.iter().enumerate() {
                if hops >= *max_hops {
                    break;
                }

                send_event(
                    sender,
                    ExecutionEvent::AgentReceivesInput {
                        agent_id: agent.id().to_string(),
                        input: current_input.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;

                send_event(
                    sender,
                    ExecutionEvent::AgentThinking {
                        agent_id: agent.id().to_string(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;

                // *** REAL LLM CALL ***
                let prompt = if idx < agents.len() - 1 {
                    format!(
                        "{}\n\nProcess this task. If you need to hand off to another agent, include 'HANDOFF:agent_id' in your response.",
                        current_input
                    )
                } else {
                    format!("{}\n\nComplete this task and provide the final result.", current_input)
                };

                let response = agent.prompt(&prompt).await?;

                // Check for handoff
                if response.contains("HANDOFF:") && idx < agents.len() - 1 {
                    let next_agent = &agents[idx + 1];
                    send_event(
                        sender,
                        ExecutionEvent::AgentHandoff {
                            from_agent: agent.id().to_string(),
                            to_agent: next_agent.id().to_string(),
                            message: response.clone(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    )
                    .await?;
                    current_input = response;
                } else {
                    send_event(
                        sender,
                        ExecutionEvent::AgentResponds {
                            agent_id: agent.id().to_string(),
                            response: response.clone(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    )
                    .await?;
                    current_input = response;
                    break;
                }

                hops += 1;
            }

            tracing::info!("Handoff pattern completed after {} hops", hops);

            send_event(
                sender,
                ExecutionEvent::PatternComplete {
                    output: current_input,
                    metadata: serde_json::json!({
                        "pattern": "handoff",
                        "hops": hops,
                    }),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;
        }

        PatternConfig::Magentic { max_iterations } => {
            tracing::info!("Starting MAGENTIC pattern with {} agents", agents.len());

            if agents.is_empty() {
                return Err(anyhow::anyhow!("Magentic pattern requires at least one agent (manager)"));
            }

            // First agent is the manager
            let manager = &agents[0];
            let workers = &agents[1..];

            // Manager receives input and creates task breakdown
            send_event(
                sender,
                ExecutionEvent::AgentReceivesInput {
                    agent_id: manager.id().to_string(),
                    input: request.input.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;

            send_event(
                sender,
                ExecutionEvent::AgentThinking {
                    agent_id: manager.id().to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;

            // *** REAL LLM CALL - Manager breaks down tasks ***
            let task_breakdown_prompt = format!(
                "{}\n\nBreak this down into 2-4 specific subtasks. List each task on a new line starting with '- '.",
                request.input
            );
            let task_breakdown = manager.prompt(&task_breakdown_prompt).await?;

            send_event(
                sender,
                ExecutionEvent::AgentResponds {
                    agent_id: manager.id().to_string(),
                    response: task_breakdown.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;

            // Parse tasks from response
            let tasks: Vec<String> = task_breakdown
                .lines()
                .filter(|line| line.trim().starts_with("- "))
                .map(|line| line.trim_start_matches("- ").trim().to_string())
                .collect();

            let mut task_results = Vec::new();

            // Assign tasks to workers
            for (idx, task) in tasks.iter().take(*max_iterations).enumerate() {
                send_event(
                    sender,
                    ExecutionEvent::PatternStep {
                        message: format!("📋 Assigning task {}/{}: {}", idx + 1, tasks.len(), task),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;

                if !workers.is_empty() {
                    // Round-robin task assignment to workers
                    let worker = &workers[idx % workers.len()];

                    send_event(
                        sender,
                        ExecutionEvent::AgentReceivesInput {
                            agent_id: worker.id().to_string(),
                            input: task.clone(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    )
                    .await?;

                    send_event(
                        sender,
                        ExecutionEvent::AgentThinking {
                            agent_id: worker.id().to_string(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    )
                    .await?;

                    // *** REAL LLM CALL - Worker completes task ***
                    let result = worker.prompt(task).await?;

                    send_event(
                        sender,
                        ExecutionEvent::AgentResponds {
                            agent_id: worker.id().to_string(),
                            response: result.clone(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    )
                    .await?;

                    task_results.push(format!("Task: {}\nResult: {}", task, result));
                }
            }

            // Manager synthesizes results
            send_event(
                sender,
                ExecutionEvent::PatternStep {
                    message: "Manager synthesizing all results...".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;

            send_event(
                sender,
                ExecutionEvent::AgentThinking {
                    agent_id: manager.id().to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;

            // *** REAL LLM CALL - Manager synthesizes ***
            let synthesis_prompt = format!(
                "Synthesize these task results into a cohesive final answer:\n\n{}",
                task_results.join("\n\n")
            );
            let final_output = manager.prompt(&synthesis_prompt).await?;

            tracing::info!("Magentic pattern completed");

            send_event(
                sender,
                ExecutionEvent::PatternComplete {
                    output: final_output,
                    metadata: serde_json::json!({
                        "pattern": "magentic",
                        "tasks_completed": tasks.len(),
                        "workers_used": workers.len(),
                    }),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;
        }
    }

    Ok(())
}

/// Helper to send an event
async fn send_event(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    event: ExecutionEvent,
) -> anyhow::Result<()> {
    // Log event being sent
    let event_type = match &event {
        ExecutionEvent::AgentStart { agent_id, .. } => format!("AgentStart({})", agent_id),
        ExecutionEvent::AgentReceivesInput { agent_id, input, .. } => format!("AgentReceivesInput({}, {}...)", agent_id, &input[..input.len().min(50)]),
        ExecutionEvent::AgentThinking { agent_id, .. } => format!("AgentThinking({})", agent_id),
        ExecutionEvent::AgentResponds { agent_id, response, .. } => format!("AgentResponds({}, {}...)", agent_id, &response[..response.len().min(50)]),
        ExecutionEvent::AgentComplete { agent_id, .. } => format!("AgentComplete({})", agent_id),
        ExecutionEvent::AgentError { agent_id, error, .. } => format!("AgentError({}, {})", agent_id, error),
        ExecutionEvent::AgentHandoff { from_agent, to_agent, .. } => format!("AgentHandoff({} -> {})", from_agent, to_agent),
        ExecutionEvent::PatternStep { message, .. } => format!("PatternStep({})", message),
        ExecutionEvent::ConversationMessage { from, .. } => format!("ConversationMessage({})", from),
        ExecutionEvent::PatternComplete { .. } => "PatternComplete".to_string(),
        ExecutionEvent::PatternError { error, .. } => format!("PatternError({})", error),
    };

    tracing::debug!("→ Sending event: {}", event_type);

    let json = serde_json::to_string(&event)?;
    sender.send(Message::Text(json)).await?;
    Ok(())
}
