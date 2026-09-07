use crate::services::backup_service::{BackupConfig, BackupService, CreateBackupRequest};
use crate::state::app_state::AppState;
use crate::utils::error::SafeFailure;
use tauri::State;

/// Create a new backup configuration
#[tauri::command]
pub async fn create_backup_config(
    _state: State<'_, AppState>,
    request: CreateBackupRequest,
) -> Result<BackupConfig, SafeFailure> {
    let backup_service = BackupService::new().map_err(SafeFailure::from)?;
    backup_service
        .create_backup_config(request)
        .map_err(SafeFailure::from)
}

/// List all backup configurations
#[tauri::command]
pub async fn list_backup_configs(
    _state: State<'_, AppState>,
) -> Result<Vec<BackupConfig>, SafeFailure> {
    let backup_service = BackupService::new().map_err(SafeFailure::from)?;
    backup_service
        .list_backup_configs()
        .map_err(SafeFailure::from)
}

/// Get a backup configuration by ID
#[tauri::command]
pub async fn get_backup_config(
    _state: State<'_, AppState>,
    id: String,
) -> Result<BackupConfig, SafeFailure> {
    let backup_service = BackupService::new().map_err(SafeFailure::from)?;
    backup_service
        .get_backup_config(&id)
        .map_err(SafeFailure::from)
}

/// Update backup enabled status
#[tauri::command]
pub async fn update_backup_status(
    _state: State<'_, AppState>,
    id: String,
    enabled: bool,
) -> Result<BackupConfig, SafeFailure> {
    let backup_service = BackupService::new().map_err(SafeFailure::from)?;
    backup_service
        .update_backup_status(&id, enabled)
        .map_err(SafeFailure::from)
}

/// Delete a backup configuration
#[tauri::command]
pub async fn delete_backup_config(
    state: State<'_, AppState>,
    id: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "delete_backup_config",
        "backup_config",
        &id,
        None,
        "delete",
    )
    .map_err(SafeFailure::from)?;
    let backup_service = BackupService::new().map_err(SafeFailure::from)?;
    backup_service
        .delete_backup_config(&id)
        .map_err(SafeFailure::from)
}

/// Get backup configurations for a specific VM
#[tauri::command]
pub async fn get_vm_backup_configs(
    _state: State<'_, AppState>,
    vm_id: String,
) -> Result<Vec<BackupConfig>, SafeFailure> {
    let backup_service = BackupService::new().map_err(SafeFailure::from)?;
    backup_service
        .get_vm_backup_configs(&vm_id)
        .map_err(SafeFailure::from)
}

/// Record a backup execution
#[tauri::command]
pub async fn record_backup(
    _state: State<'_, AppState>,
    id: String,
) -> Result<BackupConfig, SafeFailure> {
    let backup_service = BackupService::new().map_err(SafeFailure::from)?;
    backup_service.record_backup(&id).map_err(SafeFailure::from)
}
