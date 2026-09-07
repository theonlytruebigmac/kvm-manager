use crate::models::operation::{OperationKind, TargetIdentity};
use crate::models::storage::{
    StoragePool, StoragePoolConfig, Volume, VolumeConfig, VolumeEncryptionInfo,
};
use crate::services::storage_service::StorageService;
use crate::state::app_state::AppState;
use crate::utils::error::{AppError, SafeFailure};
use tauri::State;

fn storage_target(kind: &str, stable_id: String) -> TargetIdentity {
    TargetIdentity {
        resource_kind: kind.to_string(),
        stable_id,
        display_name: None,
    }
}

fn volume_target(pool_id: &str, volume_name: &str) -> TargetIdentity {
    storage_target("volume", format!("{pool_id}/{volume_name}"))
}

/// Get all storage pools (active and inactive)
#[tauri::command]
pub async fn get_storage_pools(
    state: State<'_, AppState>,
) -> Result<Vec<StoragePool>, SafeFailure> {
    tracing::debug!("get_storage_pools command called");

    let operation = state
        .resolve_operation(OperationKind::Query, None)
        .map_err(SafeFailure::from)?;
    StorageService::list_storage_pools(&operation.connection).map_err(SafeFailure::from)
}

/// Get all volumes in a storage pool
#[tauri::command]
pub async fn get_volumes(
    state: State<'_, AppState>,
    pool_id: String,
) -> Result<Vec<Volume>, SafeFailure> {
    tracing::debug!("get_volumes command called for pool: {}", pool_id);

    let operation = state
        .resolve_operation(
            OperationKind::Query,
            Some(storage_target("storage_pool", pool_id.clone())),
        )
        .map_err(SafeFailure::from)?;
    StorageService::list_volumes(&operation.connection, &pool_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Create a new volume in a storage pool
#[tauri::command]
pub async fn create_volume(
    state: State<'_, AppState>,
    pool_id: String,
    config: VolumeConfig,
) -> Result<String, SafeFailure> {
    tracing::info!(
        "create_volume command called for volume: {} in pool: {}",
        config.name,
        pool_id
    );

    let operation = state
        .resolve_operation(
            OperationKind::Mutation,
            Some(volume_target(&pool_id, &config.name)),
        )
        .map_err(SafeFailure::from)?;
    StorageService::create_volume(&operation.connection, &pool_id, config)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Delete a volume from a storage pool
#[tauri::command]
pub async fn delete_volume(
    state: State<'_, AppState>,
    pool_id: String,
    volume_name: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "delete_volume command called for volume: {} in pool: {}",
        volume_name,
        pool_id
    );

    let stable_id = format!("{pool_id}/{volume_name}");
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "delete_volume",
        "volume",
        &stable_id,
        Some(volume_name.clone()),
        "delete",
    )
    .map_err(SafeFailure::from)?;
    let operation = state
        .resolve_operation(
            OperationKind::Mutation,
            Some(volume_target(&pool_id, &volume_name)),
        )
        .map_err(SafeFailure::from)?;
    StorageService::delete_volume(&operation.connection, &pool_id, &volume_name)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Create a new storage pool
#[tauri::command]
pub async fn create_storage_pool(
    state: State<'_, AppState>,
    config: StoragePoolConfig,
) -> Result<String, SafeFailure> {
    tracing::info!(
        "create_storage_pool command called for pool: {}",
        config.name
    );

    let operation = state
        .resolve_operation(
            OperationKind::Mutation,
            Some(storage_target("storage_pool", config.name.clone())),
        )
        .map_err(SafeFailure::from)?;
    StorageService::create_storage_pool(&operation.connection, config)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Resize a volume
#[tauri::command]
pub async fn resize_volume(
    state: State<'_, AppState>,
    pool_id: String,
    volume_name: String,
    new_capacity_gb: u64,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "resize_volume command called for volume: {} in pool: {} to {}GB",
        volume_name,
        pool_id,
        new_capacity_gb
    );

    let operation = state
        .resolve_operation(
            OperationKind::Mutation,
            Some(volume_target(&pool_id, &volume_name)),
        )
        .map_err(SafeFailure::from)?;
    StorageService::resize_volume(
        &operation.connection,
        &pool_id,
        &volume_name,
        new_capacity_gb,
    )
    .map_err(SafeFailure::from)
    .map_err(|failure| failure.with_context(&operation.context))
}

/// Upload a file to create/replace a volume
#[tauri::command]
pub async fn upload_volume(
    state: State<'_, AppState>,
    pool_id: String,
    volume_name: String,
    source_path: String,
    format: Option<String>,
) -> Result<Volume, SafeFailure> {
    tracing::info!("upload_volume command called for requested volume");

    let operation = state
        .resolve_host_local_operation(Some(volume_target(&pool_id, &volume_name)))
        .map_err(SafeFailure::from)?;
    StorageService::upload_volume(
        &operation.connection,
        &pool_id,
        &volume_name,
        &source_path,
        format.as_deref(),
    )
    .map_err(SafeFailure::from)
    .map_err(|failure| failure.with_context(&operation.context))
}

/// Copy a desktop-local ISO into a connection-owned pool through a libvirt stream. The source is
/// opened read-only and is never removed or modified.
#[tauri::command]
pub async fn import_iso_to_pool(
    state: State<'_, AppState>,
    pool_id: String,
    volume_name: String,
    source_path: String,
    confirmed: bool,
) -> Result<Volume, SafeFailure> {
    if !confirmed {
        return Err(SafeFailure::from(AppError::InvalidConfig(
            "ISO import requires explicit confirmation".to_string(),
        )));
    }
    if !source_path.to_ascii_lowercase().ends_with(".iso") {
        return Err(SafeFailure::from(AppError::InvalidConfig(
            "The selected source must be an ISO image".to_string(),
        )));
    }
    let operation = state
        .resolve_operation(
            OperationKind::Mutation,
            Some(volume_target(&pool_id, &volume_name)),
        )
        .map_err(SafeFailure::from)?;
    StorageService::upload_volume(
        &operation.connection,
        &pool_id,
        &volume_name,
        &source_path,
        Some("raw"),
    )
    .map_err(SafeFailure::from)
    .map_err(|failure| failure.with_context(&operation.context))
}

/// Download a volume to a local file
#[tauri::command]
pub async fn download_volume(
    state: State<'_, AppState>,
    pool_id: String,
    volume_name: String,
    dest_path: String,
) -> Result<u64, SafeFailure> {
    tracing::info!("download_volume command called for requested volume");

    let operation = state
        .resolve_host_local_operation(Some(volume_target(&pool_id, &volume_name)))
        .map_err(SafeFailure::from)?;
    StorageService::download_volume(&operation.connection, &pool_id, &volume_name, &dest_path)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Get the file path of a volume
#[tauri::command]
pub async fn get_volume_path(
    state: State<'_, AppState>,
    pool_id: String,
    volume_name: String,
) -> Result<String, SafeFailure> {
    tracing::debug!(
        "get_volume_path command called for volume: {} in pool: {}",
        volume_name,
        pool_id
    );

    let operation = state
        .resolve_operation(
            OperationKind::Query,
            Some(volume_target(&pool_id, &volume_name)),
        )
        .map_err(SafeFailure::from)?;
    StorageService::get_volume_path(&operation.connection, &pool_id, &volume_name)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Get encryption info for a volume
#[tauri::command]
pub async fn get_volume_encryption_info(
    state: State<'_, AppState>,
    pool_id: String,
    volume_name: String,
) -> Result<VolumeEncryptionInfo, SafeFailure> {
    tracing::debug!(
        "get_volume_encryption_info command called for volume: {} in pool: {}",
        volume_name,
        pool_id
    );

    let operation = state
        .resolve_operation(
            OperationKind::Query,
            Some(volume_target(&pool_id, &volume_name)),
        )
        .map_err(SafeFailure::from)?;
    StorageService::get_volume_encryption_info(&operation.connection, &pool_id, &volume_name)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Get metadata from OVA/OVF file
#[tauri::command]
pub async fn get_ova_metadata(
    source_path: String,
) -> Result<crate::services::ova_service::OvfMetadata, SafeFailure> {
    tracing::info!("get_ova_metadata command called");

    crate::services::ova_service::OvaService::get_ova_metadata(&source_path)
        .map_err(SafeFailure::from)
}

/// Import OVA/OVF file and convert disks
#[tauri::command]
pub async fn import_ova(
    config: crate::services::ova_service::OvaImportConfig,
) -> Result<String, SafeFailure> {
    tracing::info!("import_ova command called");

    let result_path =
        crate::services::ova_service::OvaService::import_ova(config).map_err(SafeFailure::from)?;

    Ok(result_path.to_string_lossy().to_string())
}
