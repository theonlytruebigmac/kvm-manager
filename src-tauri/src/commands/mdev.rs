use crate::models::mdev::{MdevDevice, MdevStatus, MdevType};
use crate::services::mdev_service::MdevService;
use crate::utils::error::SafeFailure;
use crate::AppState;
use tauri::State;

fn require_host_device(state: &AppState) -> Result<(), Box<SafeFailure>> {
    state
        .resolve_host_local_operation(None)
        .map(|_| ())
        .map_err(|error| Box::new(SafeFailure::from(error)))
}

/// Check MDEV support status on the host
#[tauri::command]
pub async fn check_mdev_status(state: State<'_, AppState>) -> Result<MdevStatus, SafeFailure> {
    tracing::info!("check_mdev_status command called");
    require_host_device(&state).map_err(|failure| *failure)?;

    Ok(MdevService::check_mdev_status())
}

/// List all available MDEV types
#[tauri::command]
pub async fn list_mdev_types(state: State<'_, AppState>) -> Result<Vec<MdevType>, SafeFailure> {
    tracing::info!("list_mdev_types command called");
    require_host_device(&state).map_err(|failure| *failure)?;

    MdevService::list_mdev_types().map_err(SafeFailure::from)
}

/// List all active MDEV instances
#[tauri::command]
pub async fn list_mdev_devices(state: State<'_, AppState>) -> Result<Vec<MdevDevice>, SafeFailure> {
    tracing::info!("list_mdev_devices command called");
    let operation = state
        .resolve_host_local_operation(None)
        .map_err(SafeFailure::from)?;

    MdevService::list_mdev_devices(&operation.connection).map_err(SafeFailure::from)
}

/// Attach an MDEV device to a VM
#[tauri::command]
pub async fn attach_mdev(
    state: State<'_, AppState>,
    vm_id: String,
    mdev_uuid: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "attach_mdev command called for VM: {} mdev: {}",
        vm_id,
        mdev_uuid
    );
    let operation = state
        .resolve_host_local_operation(None)
        .map_err(SafeFailure::from)?;

    let stable_id = format!("{vm_id}/{mdev_uuid}");
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "attach_mdev",
        "mediated_device",
        &stable_id,
        None,
        "attach",
    )
    .map_err(SafeFailure::from)?;
    MdevService::attach_mdev(&operation.connection, &vm_id, &mdev_uuid)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Detach an MDEV device from a VM
#[tauri::command]
pub async fn detach_mdev(
    state: State<'_, AppState>,
    vm_id: String,
    mdev_uuid: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "detach_mdev command called for VM: {} mdev: {}",
        vm_id,
        mdev_uuid
    );
    let operation = state
        .resolve_host_local_operation(None)
        .map_err(SafeFailure::from)?;

    let stable_id = format!("{vm_id}/{mdev_uuid}");
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "detach_mdev",
        "mediated_device",
        &stable_id,
        None,
        "detach",
    )
    .map_err(SafeFailure::from)?;
    MdevService::detach_mdev(&operation.connection, &vm_id, &mdev_uuid)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Create a new MDEV instance
#[tauri::command]
pub async fn create_mdev(
    state: State<'_, AppState>,
    parent_device: String,
    mdev_type: String,
    confirmation_token: String,
) -> Result<String, SafeFailure> {
    tracing::info!(
        "create_mdev command called: type {} on {}",
        mdev_type,
        parent_device
    );
    require_host_device(&state).map_err(|failure| *failure)?;

    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "create_mdev",
        "host_device",
        &parent_device,
        None,
        &format!("create;type={mdev_type}"),
    )
    .map_err(SafeFailure::from)?;
    MdevService::create_mdev(&parent_device, &mdev_type).map_err(SafeFailure::from)
}

/// Remove an MDEV instance
#[tauri::command]
pub async fn remove_mdev(
    state: State<'_, AppState>,
    mdev_uuid: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!("remove_mdev command called: {}", mdev_uuid);
    require_host_device(&state).map_err(|failure| *failure)?;

    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "remove_mdev",
        "mediated_device",
        &mdev_uuid,
        None,
        "remove",
    )
    .map_err(SafeFailure::from)?;
    MdevService::remove_mdev(&mdev_uuid).map_err(SafeFailure::from)
}
