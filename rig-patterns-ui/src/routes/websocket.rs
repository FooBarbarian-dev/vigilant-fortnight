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

    // Simulate execution with streaming events
    match &request.pattern {
        PatternConfig::Sequential => {
            for (idx, agent) in request.agents.iter().enumerate() {
                // Agent start
                send_event(
                    sender,
                    ExecutionEvent::AgentStart {
                        agent_id: agent.id.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;

                // Simulate processing delay
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                // Agent complete
                send_event(
                    sender,
                    ExecutionEvent::AgentComplete {
                        agent_id: agent.id.clone(),
                        output_preview: format!("Output from {} (step {})", agent.id, idx + 1),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;
            }

            send_event(
                sender,
                ExecutionEvent::PatternComplete {
                    output: "Sequential execution complete".to_string(),
                    metadata: serde_json::json!({
                        "pattern": "sequential",
                        "steps": request.agents.len(),
                    }),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;
        }

        PatternConfig::Concurrent { .. } => {
            // Start all agents concurrently
            for agent in &request.agents {
                send_event(
                    sender,
                    ExecutionEvent::AgentStart {
                        agent_id: agent.id.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;
            }

            // Simulate concurrent execution
            tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

            // All complete
            for agent in &request.agents {
                send_event(
                    sender,
                    ExecutionEvent::AgentComplete {
                        agent_id: agent.id.clone(),
                        output_preview: format!("Concurrent output from {}", agent.id),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;
            }

            send_event(
                sender,
                ExecutionEvent::PatternStep {
                    message: "Aggregating results...".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;

            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

            send_event(
                sender,
                ExecutionEvent::PatternComplete {
                    output: "Concurrent execution complete".to_string(),
                    metadata: serde_json::json!({
                        "pattern": "concurrent",
                        "agents": request.agents.len(),
                    }),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;
        }

        PatternConfig::GroupChat { max_rounds } => {
            let rounds = (*max_rounds).min(3);

            for round in 1..=rounds {
                send_event(
                    sender,
                    ExecutionEvent::PatternStep {
                        message: format!("Round {} of {}", round, rounds),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;

                for agent in &request.agents {
                    send_event(
                        sender,
                        ExecutionEvent::AgentStart {
                            agent_id: agent.id.clone(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    )
                    .await?;

                    tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

                    send_event(
                        sender,
                        ExecutionEvent::AgentComplete {
                            agent_id: agent.id.clone(),
                            output_preview: format!("{} speaks in round {}", agent.id, round),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    )
                    .await?;
                }
            }

            send_event(
                sender,
                ExecutionEvent::PatternComplete {
                    output: format!("Group chat complete after {} rounds", rounds),
                    metadata: serde_json::json!({
                        "pattern": "group_chat",
                        "rounds": rounds,
                    }),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;
        }

        PatternConfig::Handoff { .. } => {
            let hops = request.agents.len().min(3);

            for (idx, agent) in request.agents.iter().take(hops).enumerate() {
                send_event(
                    sender,
                    ExecutionEvent::AgentStart {
                        agent_id: agent.id.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;

                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                if idx < hops - 1 {
                    send_event(
                        sender,
                        ExecutionEvent::PatternStep {
                            message: format!(
                                "{} → {} (handoff)",
                                agent.id,
                                request.agents[idx + 1].id
                            ),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    )
                    .await?;
                } else {
                    send_event(
                        sender,
                        ExecutionEvent::AgentComplete {
                            agent_id: agent.id.clone(),
                            output_preview: format!("{} handled the task", agent.id),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    )
                    .await?;
                }
            }

            send_event(
                sender,
                ExecutionEvent::PatternComplete {
                    output: "Handoff complete".to_string(),
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
            send_event(
                sender,
                ExecutionEvent::PatternStep {
                    message: "Manager creating task list...".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;

            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            let tasks = vec!["Task 1", "Task 2", "Task 3"];

            for task in tasks {
                send_event(
                    sender,
                    ExecutionEvent::PatternStep {
                        message: format!("Working on: {}", task),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;

                tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

                send_event(
                    sender,
                    ExecutionEvent::PatternStep {
                        message: format!("Completed: {}", task),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                )
                .await?;
            }

            send_event(
                sender,
                ExecutionEvent::PatternStep {
                    message: "Manager synthesizing results...".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await?;

            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            send_event(
                sender,
                ExecutionEvent::PatternComplete {
                    output: "Magentic orchestration complete".to_string(),
                    metadata: serde_json::json!({
                        "pattern": "magentic",
                        "tasks_completed": 3,
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
