use tauri::State;
use crate::models::storage::{StoragePool, Volume, VolumeConfig, StoragePoolConfig, VolumeEncryptionInfo};
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

/// Upload a file to create/replace a volume
#[tauri::command]
pub async fn upload_volume(
    state: State<'_, AppState>,
    pool_id: String,
    volume_name: String,
    source_path: String,
    format: Option<String>,
) -> Result<Volume, String> {
    tracing::info!("upload_volume command called: {} -> volume {} in pool {}",
                   source_path, volume_name, pool_id);

    StorageService::upload_volume(
        &state.libvirt,
        &pool_id,
        &volume_name,
        &source_path,
        format.as_deref(),
    )
    .map_err(|e| e.to_string())
}

/// Download a volume to a local file
#[tauri::command]
pub async fn download_volume(
    state: State<'_, AppState>,
    pool_id: String,
    volume_name: String,
    dest_path: String,
) -> Result<u64, String> {
    tracing::info!("download_volume command called: volume {} from pool {} -> {}",
                   volume_name, pool_id, dest_path);

    StorageService::download_volume(
        &state.libvirt,
        &pool_id,
        &volume_name,
        &dest_path,
    )
    .map_err(|e| e.to_string())
}

/// Get the file path of a volume
#[tauri::command]
pub async fn get_volume_path(
    state: State<'_, AppState>,
    pool_id: String,
    volume_name: String,
) -> Result<String, String> {
    tracing::debug!("get_volume_path command called for volume: {} in pool: {}",
                    volume_name, pool_id);

    StorageService::get_volume_path(&state.libvirt, &pool_id, &volume_name)
        .map_err(|e| e.to_string())
}

/// Get encryption info for a volume
#[tauri::command]
pub async fn get_volume_encryption_info(
    state: State<'_, AppState>,
    pool_id: String,
    volume_name: String,
) -> Result<VolumeEncryptionInfo, String> {
    tracing::debug!("get_volume_encryption_info command called for volume: {} in pool: {}",
                    volume_name, pool_id);

    StorageService::get_volume_encryption_info(&state.libvirt, &pool_id, &volume_name)
        .map_err(|e| e.to_string())
}

/// Get metadata from OVA/OVF file
#[tauri::command]
pub async fn get_ova_metadata(
    source_path: String,
) -> Result<crate::services::ova_service::OvfMetadata, String> {
    tracing::info!("get_ova_metadata command called for: {}", source_path);

    crate::services::ova_service::OvaService::get_ova_metadata(&source_path)
        .map_err(|e| e.to_string())
}

/// Import OVA/OVF file and convert disks
#[tauri::command]
pub async fn import_ova(
    config: crate::services::ova_service::OvaImportConfig,
) -> Result<String, String> {
    tracing::info!("import_ova command called for: {}", config.source_path);

    let result_path = crate::services::ova_service::OvaService::import_ova(config)
        .map_err(|e| e.to_string())?;

    Ok(result_path.to_string_lossy().to_string())
}
