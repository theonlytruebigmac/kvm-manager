use crate::services::scheduler_service::{CreateScheduleRequest, ScheduledOperation, SchedulerService};
use crate::state::app_state::AppState;
use tauri::State;

/// Create a new scheduled operation
#[tauri::command]
pub async fn create_schedule(
    _state: State<'_, AppState>,
    request: CreateScheduleRequest,
) -> Result<ScheduledOperation, String> {
    let scheduler_service = SchedulerService::new().map_err(|e| e.to_string())?;
    scheduler_service.create_schedule(request).map_err(|e| e.to_string())
}

/// List all scheduled operations
#[tauri::command]
pub async fn list_schedules(
    _state: State<'_, AppState>,
) -> Result<Vec<ScheduledOperation>, String> {
    let scheduler_service = SchedulerService::new().map_err(|e| e.to_string())?;
    scheduler_service.list_schedules().map_err(|e| e.to_string())
}

/// Get a schedule by ID
#[tauri::command]
pub async fn get_schedule(
    _state: State<'_, AppState>,
    id: String,
) -> Result<ScheduledOperation, String> {
    let scheduler_service = SchedulerService::new().map_err(|e| e.to_string())?;
    scheduler_service.get_schedule(&id).map_err(|e| e.to_string())
}

/// Update schedule enabled status
#[tauri::command]
pub async fn update_schedule_status(
    _state: State<'_, AppState>,
    id: String,
    enabled: bool,
) -> Result<ScheduledOperation, String> {
    let scheduler_service = SchedulerService::new().map_err(|e| e.to_string())?;
    scheduler_service.update_schedule_status(&id, enabled).map_err(|e| e.to_string())
}

/// Delete a schedule
#[tauri::command]
pub async fn delete_schedule(
    _state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let scheduler_service = SchedulerService::new().map_err(|e| e.to_string())?;
    scheduler_service.delete_schedule(&id).map_err(|e| e.to_string())
}

/// Get schedules for a specific VM
#[tauri::command]
pub async fn get_vm_schedules(
    _state: State<'_, AppState>,
    vm_id: String,
) -> Result<Vec<ScheduledOperation>, String> {
    let scheduler_service = SchedulerService::new().map_err(|e| e.to_string())?;
    scheduler_service.get_vm_schedules(&vm_id).map_err(|e| e.to_string())
}
