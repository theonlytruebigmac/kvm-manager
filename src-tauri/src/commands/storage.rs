use tauri::State;
use crate::models::storage::{StoragePool, Volume, VolumeConfig, StoragePoolConfig};
use crate::services::storage_service::StorageService;
use crate::state::app_state::AppState;

/// Get all storage pools (active and inactive)
#[tauri::command]
pub async fn get_storage_pools(state: State<'_, AppState>) -> Result<Vec<StoragePool>, String> {
    tracing::debug!("get_storage_pools command called");

    StorageService::list_storage_pools(&state.libvirt)
        .map_err(|e| e.to_string())
}

/// Get all volumes in a storage pool
#[tauri::command]
pub async fn get_volumes(state: State<'_, AppState>, pool_id: String) -> Result<Vec<Volume>, String> {
    tracing::debug!("get_volumes command called for pool: {}", pool_id);

    StorageService::list_volumes(&state.libvirt, &pool_id)
        .map_err(|e| e.to_string())
}

/// Create a new volume in a storage pool
#[tauri::command]
pub async fn create_volume(
    state: State<'_, AppState>,
    pool_id: String,
    config: VolumeConfig,
) -> Result<String, String> {
    tracing::info!("create_volume command called for volume: {} in pool: {}", config.name, pool_id);

    StorageService::create_volume(&state.libvirt, &pool_id, config)
        .map_err(|e| e.to_string())
}

/// Delete a volume from a storage pool
#[tauri::command]
pub async fn delete_volume(
    state: State<'_, AppState>,
    pool_id: String,
    volume_name: String,
) -> Result<(), String> {
    tracing::info!("delete_volume command called for volume: {} in pool: {}", volume_name, pool_id);

    StorageService::delete_volume(&state.libvirt, &pool_id, &volume_name)
        .map_err(|e| e.to_string())
}

/// Create a new storage pool
#[tauri::command]
pub async fn create_storage_pool(
    state: State<'_, AppState>,
    config: StoragePoolConfig,
) -> Result<String, String> {
    tracing::info!("create_storage_pool command called for pool: {}", config.name);

    StorageService::create_storage_pool(&state.libvirt, config)
        .map_err(|e| e.to_string())
}

/// Resize a volume
#[tauri::command]
pub async fn resize_volume(
    state: State<'_, AppState>,
    pool_id: String,
    volume_name: String,
    new_capacity_gb: u64,
) -> Result<(), String> {
    tracing::info!("resize_volume command called for volume: {} in pool: {} to {}GB",
                   volume_name, pool_id, new_capacity_gb);

    StorageService::resize_volume(&state.libvirt, &pool_id, &volume_name, new_capacity_gb)
        .map_err(|e| e.to_string())
}
