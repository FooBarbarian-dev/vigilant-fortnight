//! WebSocket handler for real-time execution streaming

use crate::state::{ExecuteRequest, ExecutionEvent};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State as AxumState,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    state: AxumState<Arc<crate::state::AppState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, _state: AxumState<Arc<crate::state::AppState>>) {
    let (mut sender, mut receiver) = socket.split();

    // Listen for execution requests
    while let Some(msg) = receiver.next().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // Client disconnected
            return;
        };

        if let Message::Text(text) = msg {
            // Parse the execute request
            match serde_json::from_str::<ExecuteRequest>(&text) {
                Ok(request) => {
                    tracing::info!(
                        "WebSocket: Executing pattern {:?} with {} agents",
                        request.pattern,
                        request.agents.len()
                    );

                    // Execute with streaming events
                    if let Err(e) = execute_with_streaming(&mut sender, request).await {
                        tracing::error!("WebSocket execution error: {}", e);
                        let _ = send_event(
                            &mut sender,
                            ExecutionEvent::PatternError {
                                error: e.to_string(),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            },
                        )
                        .await;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse execute request: {}", e);
                    let _ = send_event(
                        &mut sender,
                        ExecutionEvent::PatternError {
                            error: format!("Invalid request: {}", e),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    )
                    .await;
                }
            }
        }
    }
}

/// Execute pattern with streaming events
async fn execute_with_streaming(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    request: ExecuteRequest,
) -> anyhow::Result<()> {
    use crate::state::PatternConfig;

    // Simulate execution with detailed streaming events
    match &request.pattern {
        PatternConfig::Sequential => {
            let mut current_input = request.input.clone();

            for (idx, agent) in request.agents.iter().enumerate() {
                // Agent receives input
                send_event(
                    sender,
                    ExecutionEvent::AgentReceivesInput {
                        agent_id: agent.id.clone(),
                        input: current_input.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;

                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

                // Agent thinking
                send_event(
                    sender,
                    ExecutionEvent::AgentThinking {
                        agent_id: agent.id.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;

                tokio::time::sleep(tokio::time::Duration::from_millis(700)).await;

                // Agent responds
                let response = format!(
                    "Processed by {}: Enhanced and refined the input with my expertise in {}. Ready for next stage.",
                    agent.id,
                    agent.system_prompt.split_whitespace().take(5).collect::<Vec<_>>().join(" ")
                );

                send_event(
                    sender,
                    ExecutionEvent::AgentResponds {
                        agent_id: agent.id.clone(),
                        response: response.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;

                current_input = response;

                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }

            send_event(
                sender,
                ExecutionEvent::PatternComplete {
                    output: current_input,
                    metadata: serde_json::json!({
                        "pattern": "sequential",
                        "steps": request.agents.len(),
                    }),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;
        }

        PatternConfig::Concurrent { aggregation } => {
            // All agents receive the same input
            for agent in &request.agents {
                send_event(
                    sender,
                    ExecutionEvent::AgentReceivesInput {
                        agent_id: agent.id.clone(),
                        input: request.input.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

            // All thinking concurrently
            for agent in &request.agents {
                send_event(
                    sender,
                    ExecutionEvent::AgentThinking {
                        agent_id: agent.id.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

            // All respond
            for agent in &request.agents {
                let response = format!(
                    "From {}'s perspective: I analyzed this from my specialized viewpoint. My findings suggest: [detailed analysis based on {}]",
                    agent.id,
                    agent.system_prompt.split_whitespace().take(4).collect::<Vec<_>>().join(" ")
                );

                send_event(
                    sender,
                    ExecutionEvent::AgentResponds {
                        agent_id: agent.id.clone(),
                        response,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;

                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }

            send_event(
                sender,
                ExecutionEvent::PatternStep {
                    message: format!("Aggregating results using {:?} strategy...", aggregation),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;

            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

            send_event(
                sender,
                ExecutionEvent::PatternComplete {
                    output: format!("Combined insights from {} agents using {:?} aggregation", request.agents.len(), aggregation),
                    metadata: serde_json::json!({
                        "pattern": "concurrent",
                        "agents": request.agents.len(),
                        "aggregation": format!("{:?}", aggregation),
                    }),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;
        }

        PatternConfig::GroupChat { max_rounds } => {
            let rounds = (*max_rounds).min(3);
            let mut conversation_history = request.input.clone();

            for round in 1..=rounds {
                send_event(
                    sender,
                    ExecutionEvent::PatternStep {
                        message: format!("🔄 Discussion Round {} of {}", round, rounds),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;

                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

                for agent in &request.agents {
                    // Agent receives conversation history
                    send_event(
                        sender,
                        ExecutionEvent::AgentReceivesInput {
                            agent_id: agent.id.clone(),
                            input: conversation_history.clone(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    )
                    .await?;

                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

                    send_event(
                        sender,
                        ExecutionEvent::AgentThinking {
                            agent_id: agent.id.clone(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    )
                    .await?;

                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                    let response = if round == rounds {
                        format!("{}: I agree with the direction we've reached. This looks good! CONSENSUS_REACHED", agent.id)
                    } else {
                        format!(
                            "{}: Here's my take - {}. What do others think?",
                            agent.id,
                            agent.system_prompt.split_whitespace().take(6).collect::<Vec<_>>().join(" ")
                        )
                    };

                    send_event(
                        sender,
                        ExecutionEvent::ConversationMessage {
                            from: agent.id.clone(),
                            message: response.clone(),
                            message_type: if round == rounds { "consensus".to_string() } else { "output".to_string() },
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    )
                    .await?;

                    conversation_history.push_str(&format!("\n{}", response));

                    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
                }
            }

            send_event(
                sender,
                ExecutionEvent::PatternComplete {
                    output: format!("Group consensus reached after {} rounds", rounds),
                    metadata: serde_json::json!({
                        "pattern": "group_chat",
                        "rounds": rounds,
                        "participants": request.agents.len(),
                    }),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;
        }

        PatternConfig::Handoff { .. } => {
            let hops = request.agents.len().min(3);
            let mut current_input = request.input.clone();

            for (idx, agent) in request.agents.iter().take(hops).enumerate() {
                send_event(
                    sender,
                    ExecutionEvent::AgentReceivesInput {
                        agent_id: agent.id.clone(),
                        input: current_input.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;

                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

                send_event(
                    sender,
                    ExecutionEvent::AgentThinking {
                        agent_id: agent.id.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;

                tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

                if idx < hops - 1 {
                    let next_agent = &request.agents[idx + 1];
                    let handoff_msg = format!(
                        "I've handled the {} part. Handing off to {} for specialized {}",
                        agent.system_prompt.split_whitespace().take(3).collect::<Vec<_>>().join(" "),
                        next_agent.id,
                        next_agent.system_prompt.split_whitespace().take(3).collect::<Vec<_>>().join(" ")
                    );

                    send_event(
                        sender,
                        ExecutionEvent::AgentHandoff {
                            from_agent: agent.id.clone(),
                            to_agent: next_agent.id.clone(),
                            message: handoff_msg.clone(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    )
                    .await?;

                    current_input = handoff_msg;
                } else {
                    let response = format!("{}: Task completed successfully. Final result ready.", agent.id);
                    send_event(
                        sender,
                        ExecutionEvent::AgentResponds {
                            agent_id: agent.id.clone(),
                            response: response.clone(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    )
                    .await?;
                    current_input = response;
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }

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

        PatternConfig::Magentic { .. } => {
            // Manager creates tasks
            let manager_id = &request.agents[0].id;

            send_event(
                sender,
                ExecutionEvent::AgentReceivesInput {
                    agent_id: manager_id.clone(),
                    input: request.input.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;

            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

            send_event(
                sender,
                ExecutionEvent::AgentThinking {
                    agent_id: manager_id.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;

            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            let tasks = vec![
                "Analyze core requirements",
                "Research relevant approaches",
                "Synthesize findings"
            ];

            send_event(
                sender,
                ExecutionEvent::AgentResponds {
                    agent_id: manager_id.clone(),
                    response: format!("Task breakdown: {}", tasks.join(", ")),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;

            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

            for task in &tasks {
                send_event(
                    sender,
                    ExecutionEvent::PatternStep {
                        message: format!("📋 Assigning: {}", task),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;

                if request.agents.len() > 1 {
                    let worker = &request.agents[1];

                    send_event(
                        sender,
                        ExecutionEvent::AgentReceivesInput {
                            agent_id: worker.id.clone(),
                            input: task.to_string(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    )
                    .await?;

                    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

                    send_event(
                        sender,
                        ExecutionEvent::AgentThinking {
                            agent_id: worker.id.clone(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    )
                    .await?;

                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                    send_event(
                        sender,
                        ExecutionEvent::AgentResponds {
                            agent_id: worker.id.clone(),
                            response: format!("Completed: {} ✓", task),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    )
                    .await?;
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
            }

            send_event(
                sender,
                ExecutionEvent::PatternStep {
                    message: "Manager synthesizing all results...".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;

            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            send_event(
                sender,
                ExecutionEvent::PatternComplete {
                    output: format!("All {} tasks completed and synthesized", tasks.len()),
                    metadata: serde_json::json!({
                        "pattern": "magentic",
                        "tasks_completed": tasks.len(),
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
    let json = serde_json::to_string(&event)?;
    sender.send(Message::Text(json)).await?;
    Ok(())
}
