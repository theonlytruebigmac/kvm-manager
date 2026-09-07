use crate::models::operation::{OperationKind, TargetIdentity};
use crate::services::optimization_service::{OptimizationService, OptimizationSuggestion};
use crate::state::app_state::AppState;
use crate::utils::error::SafeFailure;
use tauri::State;

/// Analyze a single VM and get performance optimization suggestions
#[tauri::command]
pub async fn analyze_vm_performance(
    state: State<'_, AppState>,
    vm_id: String,
    vm_name: String,
    time_range_hours: Option<i64>,
) -> Result<Vec<OptimizationSuggestion>, SafeFailure> {
    tracing::info!("analyze_vm_performance command called for VM: {}", vm_id);

    let time_range = time_range_hours.unwrap_or(24);
    state
        .resolve_operation(
            OperationKind::Query,
            Some(TargetIdentity {
                resource_kind: "vm".to_string(),
                stable_id: vm_id.clone(),
                display_name: None,
            }),
        )
        .map_err(SafeFailure::from)?;

    OptimizationService::analyze_vm_performance(&state, &vm_id, &vm_name, time_range)
        .map_err(SafeFailure::from)
}

/// Analyze all VMs and get performance optimization suggestions
#[tauri::command]
pub async fn analyze_all_vms(
    state: State<'_, AppState>,
    time_range_hours: Option<i64>,
) -> Result<Vec<OptimizationSuggestion>, SafeFailure> {
    tracing::info!("analyze_all_vms command called");

    let time_range = time_range_hours.unwrap_or(24);
    let operation = state
        .resolve_operation(OperationKind::Query, None)
        .map_err(SafeFailure::from)?;

    OptimizationService::analyze_all_vms(&state, &operation.connection, time_range)
        .map_err(SafeFailure::from)
}
