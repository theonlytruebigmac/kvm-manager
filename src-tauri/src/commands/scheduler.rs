use crate::services::scheduler_service::{
    CreateScheduleRequest, ScheduledOperation, SchedulerService,
};
use crate::state::app_state::AppState;
use crate::utils::error::SafeFailure;
use tauri::State;

/// Create a new scheduled operation
#[tauri::command]
pub async fn create_schedule(
    _state: State<'_, AppState>,
    request: CreateScheduleRequest,
) -> Result<ScheduledOperation, SafeFailure> {
    let scheduler_service = SchedulerService::new().map_err(SafeFailure::from)?;
    scheduler_service
        .create_schedule(request)
        .map_err(SafeFailure::from)
}

/// List all scheduled operations
#[tauri::command]
pub async fn list_schedules(
    _state: State<'_, AppState>,
) -> Result<Vec<ScheduledOperation>, SafeFailure> {
    let scheduler_service = SchedulerService::new().map_err(SafeFailure::from)?;
    scheduler_service
        .list_schedules()
        .map_err(SafeFailure::from)
}

/// Get a schedule by ID
#[tauri::command]
pub async fn get_schedule(
    _state: State<'_, AppState>,
    id: String,
) -> Result<ScheduledOperation, SafeFailure> {
    let scheduler_service = SchedulerService::new().map_err(SafeFailure::from)?;
    scheduler_service
        .get_schedule(&id)
        .map_err(SafeFailure::from)
}

/// Update schedule enabled status
#[tauri::command]
pub async fn update_schedule_status(
    _state: State<'_, AppState>,
    id: String,
    enabled: bool,
) -> Result<ScheduledOperation, SafeFailure> {
    let scheduler_service = SchedulerService::new().map_err(SafeFailure::from)?;
    scheduler_service
        .update_schedule_status(&id, enabled)
        .map_err(SafeFailure::from)
}

/// Delete a schedule
#[tauri::command]
pub async fn delete_schedule(
    state: State<'_, AppState>,
    id: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "delete_schedule",
        "schedule",
        &id,
        None,
        "delete",
    )
    .map_err(SafeFailure::from)?;
    let scheduler_service = SchedulerService::new().map_err(SafeFailure::from)?;
    scheduler_service
        .delete_schedule(&id)
        .map_err(SafeFailure::from)
}

/// Get schedules for a specific VM
#[tauri::command]
pub async fn get_vm_schedules(
    _state: State<'_, AppState>,
    vm_id: String,
) -> Result<Vec<ScheduledOperation>, SafeFailure> {
    let scheduler_service = SchedulerService::new().map_err(SafeFailure::from)?;
    scheduler_service
        .get_vm_schedules(&vm_id)
        .map_err(SafeFailure::from)
}
