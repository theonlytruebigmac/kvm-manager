use tauri::State;
use crate::AppState;
use crate::models::mdev::{MdevType, MdevDevice, MdevStatus};
use crate::services::mdev_service::MdevService;

/// Check MDEV support status on the host
#[tauri::command]
pub async fn check_mdev_status() -> Result<MdevStatus, String> {
    tracing::info!("check_mdev_status command called");

    Ok(MdevService::check_mdev_status())
}

/// List all available MDEV types
#[tauri::command]
pub async fn list_mdev_types() -> Result<Vec<MdevType>, String> {
    tracing::info!("list_mdev_types command called");

    MdevService::list_mdev_types()
        .map_err(|e| e.to_string())
}

/// List all active MDEV instances
#[tauri::command]
pub async fn list_mdev_devices(
    state: State<'_, AppState>,
) -> Result<Vec<MdevDevice>, String> {
    tracing::info!("list_mdev_devices command called");

    MdevService::list_mdev_devices(&state.libvirt)
        .map_err(|e| e.to_string())
}

/// Attach an MDEV device to a VM
#[tauri::command]
pub async fn attach_mdev(
    state: State<'_, AppState>,
    vm_id: String,
    mdev_uuid: String,
) -> Result<(), String> {
    tracing::info!("attach_mdev command called for VM: {} mdev: {}", vm_id, mdev_uuid);

    MdevService::attach_mdev(&state.libvirt, &vm_id, &mdev_uuid)
        .map_err(|e| e.to_string())
}

/// Detach an MDEV device from a VM
#[tauri::command]
pub async fn detach_mdev(
    state: State<'_, AppState>,
    vm_id: String,
    mdev_uuid: String,
) -> Result<(), String> {
    tracing::info!("detach_mdev command called for VM: {} mdev: {}", vm_id, mdev_uuid);

    MdevService::detach_mdev(&state.libvirt, &vm_id, &mdev_uuid)
        .map_err(|e| e.to_string())
}

/// Create a new MDEV instance
#[tauri::command]
pub async fn create_mdev(
    parent_device: String,
    mdev_type: String,
) -> Result<String, String> {
    tracing::info!("create_mdev command called: type {} on {}", mdev_type, parent_device);

    MdevService::create_mdev(&parent_device, &mdev_type)
        .map_err(|e| e.to_string())
}

/// Remove an MDEV instance
#[tauri::command]
pub async fn remove_mdev(
    mdev_uuid: String,
) -> Result<(), String> {
    tracing::info!("remove_mdev command called: {}", mdev_uuid);

    MdevService::remove_mdev(&mdev_uuid)
        .map_err(|e| e.to_string())
}
