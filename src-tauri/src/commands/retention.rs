use tauri::State;
use crate::services::retention_service::RetentionPolicy;
use crate::state::app_state::AppState;

/// Get current retention policy
#[tauri::command]
pub async fn get_retention_policy(
    state: State<'_, AppState>,
) -> Result<RetentionPolicy, String> {
    tracing::info!("get_retention_policy command called");

    state.retention_service.load_policy()
        .map_err(|e| e.to_string())
}

/// Update retention policy
#[tauri::command]
pub async fn update_retention_policy(
    state: State<'_, AppState>,
    policy: RetentionPolicy,
) -> Result<RetentionPolicy, String> {
    tracing::info!("update_retention_policy command called: enabled={}, days={}",
        policy.enabled, policy.retention_days);

    state.retention_service.update_policy(policy)
        .map_err(|e| e.to_string())
}

/// Execute cleanup now (manual trigger)
#[tauri::command]
pub async fn execute_retention_cleanup(
    state: State<'_, AppState>,
) -> Result<usize, String> {
    tracing::info!("execute_retention_cleanup command called");

    state.retention_service.execute_cleanup()
        .map_err(|e| e.to_string())
}
