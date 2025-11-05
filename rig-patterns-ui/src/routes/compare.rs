//! Pattern comparison endpoint

use crate::state::{CompareRequest, CompareResponse, ExecuteRequest, PatternConfig};
use axum::{extract::State as AxumState, http::StatusCode, Json};
use rig_patterns::Aggregation;
use std::collections::HashMap;
use std::sync::Arc;

/// Compare all patterns with the same agents and input
pub async fn compare_patterns(
    state: AxumState<Arc<crate::state::AppState>>,
    Json(request): Json<CompareRequest>,
) -> Result<Json<CompareResponse>, (StatusCode, String)> {
    tracing::info!(
        "Comparing all patterns with {} agents",
        request.agents.len()
    );

    let patterns = vec![
        ("Sequential", PatternConfig::Sequential),
        (
            "Concurrent (Vote)",
            PatternConfig::Concurrent {
                aggregation: Aggregation::Vote,
            },
        ),
        (
            "Concurrent (Combine)",
            PatternConfig::Concurrent {
                aggregation: Aggregation::Combine,
            },
        ),
        (
            "Group Chat",
            PatternConfig::GroupChat { max_rounds: 3 },
        ),
        ("Handoff", PatternConfig::Handoff { max_hops: 5 }),
        (
            "Magentic",
            PatternConfig::Magentic { max_iterations: 5 },
        ),
    ];

    let mut results = HashMap::new();

    for (name, pattern) in patterns {
        let execute_request = ExecuteRequest {
            agents: request.agents.clone(),
            pattern,
            input: request.input.clone(),
        };

        match super::execute::execute_pattern(state.clone(), Json(execute_request)).await {
            Ok(Json(response)) => {
                results.insert(name.to_string(), response);
            }
            Err((code, msg)) => {
                tracing::error!("Pattern {} failed: {} - {}", name, code, msg);
                // Continue with other patterns even if one fails
            }
        }
    }

    Ok(Json(CompareResponse { results }))
}
