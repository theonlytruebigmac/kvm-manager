use tauri::State;
use crate::services::optimization_service::{OptimizationService, OptimizationSuggestion};
use crate::state::app_state::AppState;

/// Analyze a single VM and get performance optimization suggestions
#[tauri::command]
pub async fn analyze_vm_performance(
    state: State<'_, AppState>,
    vm_id: String,
    vm_name: String,
    time_range_hours: Option<i64>,
) -> Result<Vec<OptimizationSuggestion>, String> {
    tracing::info!("analyze_vm_performance command called for VM: {}", vm_id);

    let time_range = time_range_hours.unwrap_or(24);

    OptimizationService::analyze_vm_performance(&state, &vm_id, &vm_name, time_range)
        .map_err(|e| e.to_string())
}

/// Analyze all VMs and get performance optimization suggestions
#[tauri::command]
pub async fn analyze_all_vms(
    state: State<'_, AppState>,
    time_range_hours: Option<i64>,
) -> Result<Vec<OptimizationSuggestion>, String> {
    tracing::info!("analyze_all_vms command called");

    let time_range = time_range_hours.unwrap_or(24);

    OptimizationService::analyze_all_vms(&state, time_range)
        .map_err(|e| e.to_string())
}
