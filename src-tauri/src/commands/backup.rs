use crate::services::backup_service::{BackupConfig, BackupService, CreateBackupRequest};
use crate::state::app_state::AppState;
use tauri::State;

/// Create a new backup configuration
#[tauri::command]
pub async fn create_backup_config(
    _state: State<'_, AppState>,
    request: CreateBackupRequest,
) -> Result<BackupConfig, String> {
    let backup_service = BackupService::new().map_err(|e| e.to_string())?;
    backup_service
        .create_backup_config(request)
        .map_err(|e| e.to_string())
}

/// List all backup configurations
#[tauri::command]
pub async fn list_backup_configs(
    _state: State<'_, AppState>,
) -> Result<Vec<BackupConfig>, String> {
    let backup_service = BackupService::new().map_err(|e| e.to_string())?;
    backup_service
        .list_backup_configs()
        .map_err(|e| e.to_string())
}

/// Get a backup configuration by ID
#[tauri::command]
pub async fn get_backup_config(
    _state: State<'_, AppState>,
    id: String,
) -> Result<BackupConfig, String> {
    let backup_service = BackupService::new().map_err(|e| e.to_string())?;
    backup_service
        .get_backup_config(&id)
        .map_err(|e| e.to_string())
}

/// Update backup enabled status
#[tauri::command]
pub async fn update_backup_status(
    _state: State<'_, AppState>,
    id: String,
    enabled: bool,
) -> Result<BackupConfig, String> {
    let backup_service = BackupService::new().map_err(|e| e.to_string())?;
    backup_service
        .update_backup_status(&id, enabled)
        .map_err(|e| e.to_string())
}

/// Delete a backup configuration
#[tauri::command]
pub async fn delete_backup_config(
    _state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let backup_service = BackupService::new().map_err(|e| e.to_string())?;
    backup_service
        .delete_backup_config(&id)
        .map_err(|e| e.to_string())
}

/// Get backup configurations for a specific VM
#[tauri::command]
pub async fn get_vm_backup_configs(
    _state: State<'_, AppState>,
    vm_id: String,
) -> Result<Vec<BackupConfig>, String> {
    let backup_service = BackupService::new().map_err(|e| e.to_string())?;
    backup_service
        .get_vm_backup_configs(&vm_id)
        .map_err(|e| e.to_string())
}

/// Record a backup execution
#[tauri::command]
pub async fn record_backup(
    _state: State<'_, AppState>,
    id: String,
) -> Result<BackupConfig, String> {
    let backup_service = BackupService::new().map_err(|e| e.to_string())?;
    backup_service.record_backup(&id).map_err(|e| e.to_string())
}
