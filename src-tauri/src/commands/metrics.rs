use crate::services::metrics_service::{HistoricalMetrics, VmMetrics};
use crate::state::app_state::AppState;
use crate::utils::error::SafeFailure;
use tauri::State;

/// Store VM performance metrics
#[tauri::command]
pub async fn store_vm_metrics(
    state: State<'_, AppState>,
    metrics: VmMetrics,
) -> Result<(), SafeFailure> {
    tracing::debug!("store_vm_metrics command called for VM: {}", metrics.vm_id);

    state
        .metrics
        .store_metrics(&metrics)
        .map_err(SafeFailure::from)
}

/// Get historical metrics for a VM
#[tauri::command]
pub async fn get_historical_metrics(
    state: State<'_, AppState>,
    vm_id: String,
    start_time: i64,
    end_time: i64,
    max_points: Option<usize>,
) -> Result<HistoricalMetrics, SafeFailure> {
    tracing::debug!(
        "get_historical_metrics command called for VM: {}, range: {} to {}, max_points: {:?}",
        vm_id,
        start_time,
        end_time,
        max_points
    );

    state
        .metrics
        .get_historical_metrics(&vm_id, start_time, end_time, max_points)
        .map_err(SafeFailure::from)
}

/// Cleanup old metrics
#[tauri::command]
pub async fn cleanup_old_metrics(
    state: State<'_, AppState>,
    older_than: i64,
) -> Result<usize, SafeFailure> {
    tracing::info!(
        "cleanup_old_metrics command called: older than {}",
        older_than
    );

    state
        .metrics
        .cleanup_old_metrics(older_than)
        .map_err(SafeFailure::from)
}

/// Get total metrics count
#[tauri::command]
pub async fn get_metrics_count(state: State<'_, AppState>) -> Result<i64, SafeFailure> {
    tracing::debug!("get_metrics_count command called");

    state.metrics.get_metrics_count().map_err(SafeFailure::from)
}
