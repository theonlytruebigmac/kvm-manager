use tauri::State;
use crate::state::app_state::AppState;
use crate::services::metrics_service::{VmMetrics, HistoricalMetrics};

/// Store VM performance metrics
#[tauri::command]
pub async fn store_vm_metrics(
    state: State<'_, AppState>,
    metrics: VmMetrics,
) -> Result<(), String> {
    tracing::debug!("store_vm_metrics command called for VM: {}", metrics.vm_id);

    state.metrics.store_metrics(&metrics)
        .map_err(|e| e.to_string())
}

/// Get historical metrics for a VM
#[tauri::command]
pub async fn get_historical_metrics(
    state: State<'_, AppState>,
    vm_id: String,
    start_time: i64,
    end_time: i64,
    max_points: Option<usize>,
) -> Result<HistoricalMetrics, String> {
    tracing::debug!(
        "get_historical_metrics command called for VM: {}, range: {} to {}, max_points: {:?}",
        vm_id, start_time, end_time, max_points
    );

    state.metrics.get_historical_metrics(&vm_id, start_time, end_time, max_points)
        .map_err(|e| e.to_string())
}

/// Cleanup old metrics
#[tauri::command]
pub async fn cleanup_old_metrics(
    state: State<'_, AppState>,
    older_than: i64,
) -> Result<usize, String> {
    tracing::info!("cleanup_old_metrics command called: older than {}", older_than);

    state.metrics.cleanup_old_metrics(older_than)
        .map_err(|e| e.to_string())
}

/// Get total metrics count
#[tauri::command]
pub async fn get_metrics_count(
    state: State<'_, AppState>,
) -> Result<i64, String> {
    tracing::debug!("get_metrics_count command called");

    state.metrics.get_metrics_count()
        .map_err(|e| e.to_string())
}
