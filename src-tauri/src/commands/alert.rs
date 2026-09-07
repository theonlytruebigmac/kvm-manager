use crate::services::alert_service::{AlertEvent, AlertService, CreateAlertRequest, ResourceAlert};
use crate::state::app_state::AppState;
use crate::utils::error::SafeFailure;
use tauri::State;

/// Create a new resource alert
#[tauri::command]
pub async fn create_alert(
    _state: State<'_, AppState>,
    request: CreateAlertRequest,
) -> Result<ResourceAlert, SafeFailure> {
    let alert_service = AlertService::new().map_err(SafeFailure::from)?;
    alert_service
        .create_alert(request)
        .map_err(SafeFailure::from)
}

/// List all resource alerts
#[tauri::command]
pub async fn list_alerts(_state: State<'_, AppState>) -> Result<Vec<ResourceAlert>, SafeFailure> {
    let alert_service = AlertService::new().map_err(SafeFailure::from)?;
    alert_service.list_alerts().map_err(SafeFailure::from)
}

/// Get an alert by ID
#[tauri::command]
pub async fn get_alert(
    _state: State<'_, AppState>,
    id: String,
) -> Result<ResourceAlert, SafeFailure> {
    let alert_service = AlertService::new().map_err(SafeFailure::from)?;
    alert_service.get_alert(&id).map_err(SafeFailure::from)
}

/// Update alert enabled status
#[tauri::command]
pub async fn update_alert_status(
    _state: State<'_, AppState>,
    id: String,
    enabled: bool,
) -> Result<ResourceAlert, SafeFailure> {
    let alert_service = AlertService::new().map_err(SafeFailure::from)?;
    alert_service
        .update_alert_status(&id, enabled)
        .map_err(SafeFailure::from)
}

/// Delete an alert
#[tauri::command]
pub async fn delete_alert(
    state: State<'_, AppState>,
    id: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "delete_alert",
        "alert",
        &id,
        None,
        "delete",
    )
    .map_err(SafeFailure::from)?;
    let alert_service = AlertService::new().map_err(SafeFailure::from)?;
    alert_service.delete_alert(&id).map_err(SafeFailure::from)
}

/// Get alerts for a specific VM
#[tauri::command]
pub async fn get_vm_alerts(
    _state: State<'_, AppState>,
    vm_id: String,
) -> Result<Vec<ResourceAlert>, SafeFailure> {
    let alert_service = AlertService::new().map_err(SafeFailure::from)?;
    alert_service
        .get_vm_alerts(&vm_id)
        .map_err(SafeFailure::from)
}

/// Check threshold and get alert event if triggered
#[tauri::command]
pub async fn check_alert_threshold(
    _state: State<'_, AppState>,
    alert_id: String,
    current_value: f64,
) -> Result<Option<AlertEvent>, SafeFailure> {
    let alert_service = AlertService::new().map_err(SafeFailure::from)?;
    alert_service
        .check_threshold(&alert_id, current_value)
        .map_err(SafeFailure::from)
}
