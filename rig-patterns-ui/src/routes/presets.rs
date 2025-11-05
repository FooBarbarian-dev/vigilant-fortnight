//! Presets endpoint

use crate::state::PresetConfig;
use axum::{extract::State as AxumState, Json};
use std::sync::Arc;

/// Get all available preset configurations
pub async fn get_presets(
    state: AxumState<Arc<crate::state::AppState>>,
) -> Json<Vec<PresetConfig>> {
    let presets = state.presets.read().await;
    Json(presets.clone())
}
