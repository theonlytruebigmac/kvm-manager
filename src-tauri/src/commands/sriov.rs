use crate::models::pci::{SriovPf, SriovVf, SriovVfConfig};
use crate::services::sriov_service::SriovService;
use crate::state::app_state::AppState;
use crate::utils::error::SafeFailure;
use tauri::{AppHandle, Emitter, State};

fn require_host_device(state: &AppState) -> Result<(), Box<SafeFailure>> {
    state
        .resolve_host_local_operation(None)
        .map(|_| ())
        .map_err(|error| Box::new(SafeFailure::from(error)))
}

/// List all SR-IOV capable Physical Functions
#[tauri::command]
pub async fn list_sriov_pfs(state: State<'_, AppState>) -> Result<Vec<SriovPf>, SafeFailure> {
    tracing::debug!("list_sriov_pfs command called");
    require_host_device(&state).map_err(|failure| *failure)?;
    SriovService::list_sriov_pfs().map_err(SafeFailure::from)
}

/// List Virtual Functions for a Physical Function
#[tauri::command]
pub async fn list_sriov_vfs(
    state: State<'_, AppState>,
    pf_address: String,
) -> Result<Vec<SriovVf>, SafeFailure> {
    tracing::debug!("list_sriov_vfs command called");
    let operation = state
        .resolve_host_local_operation(None)
        .map_err(SafeFailure::from)?;
    SriovService::list_vfs(&pf_address, &operation.connection)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Enable VFs on a Physical Function
#[tauri::command]
pub async fn enable_sriov_vfs(
    app: AppHandle,
    state: State<'_, AppState>,
    pf_address: String,
    num_vfs: u32,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "enable_sriov_vfs command called: PF={}, num_vfs={}",
        pf_address,
        num_vfs
    );
    require_host_device(&state).map_err(|failure| *failure)?;

    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "enable_sriov_vfs",
        "host_device",
        &pf_address,
        None,
        &format!("enable-vfs={num_vfs}"),
    )
    .map_err(SafeFailure::from)?;
    SriovService::enable_vfs(&pf_address, num_vfs).map_err(SafeFailure::from)?;

    let _ = app.emit(
        "sriov-vfs-enabled",
        serde_json::json!({
            "pfAddress": pf_address,
            "numVfs": num_vfs,
        }),
    );

    Ok(())
}

/// Configure a Virtual Function
#[tauri::command]
pub async fn configure_sriov_vf(
    app: AppHandle,
    state: State<'_, AppState>,
    config: SriovVfConfig,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "configure_sriov_vf command called: interface={}, vf={}",
        config.pf_interface,
        config.vf_index
    );
    require_host_device(&state).map_err(|failure| *failure)?;

    let stable_id = format!("{}/{}", config.pf_interface, config.vf_index);
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "configure_sriov_vf",
        "host_device",
        &stable_id,
        None,
        "configure",
    )
    .map_err(SafeFailure::from)?;
    SriovService::configure_vf(&config).map_err(SafeFailure::from)?;

    let _ = app.emit(
        "sriov-vf-configured",
        serde_json::json!({
            "pfInterface": config.pf_interface,
            "vfIndex": config.vf_index,
        }),
    );

    Ok(())
}

/// Attach a VF to a VM
#[tauri::command]
pub async fn attach_sriov_vf(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    vf_address: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "attach_sriov_vf command called: VM={}, VF={}",
        vm_id,
        vf_address
    );
    let operation = state
        .resolve_host_local_operation(None)
        .map_err(SafeFailure::from)?;

    let stable_id = format!("{vm_id}/{vf_address}");
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "attach_sriov_vf",
        "sriov_vf",
        &stable_id,
        None,
        "attach",
    )
    .map_err(SafeFailure::from)?;
    SriovService::attach_vf_to_vm(&operation.connection, &vm_id, &vf_address)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    let _ = app.emit(
        "sriov-vf-attached",
        serde_json::json!({
            "vmId": vm_id,
            "vfAddress": vf_address,
        }),
    );

    Ok(())
}

/// Detach a VF from a VM
#[tauri::command]
pub async fn detach_sriov_vf(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    vf_address: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "detach_sriov_vf command called: VM={}, VF={}",
        vm_id,
        vf_address
    );
    let operation = state
        .resolve_host_local_operation(None)
        .map_err(SafeFailure::from)?;

    let stable_id = format!("{vm_id}/{vf_address}");
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "detach_sriov_vf",
        "sriov_vf",
        &stable_id,
        None,
        "detach",
    )
    .map_err(SafeFailure::from)?;
    SriovService::detach_vf_from_vm(&operation.connection, &vm_id, &vf_address)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    let _ = app.emit(
        "sriov-vf-detached",
        serde_json::json!({
            "vmId": vm_id,
            "vfAddress": vf_address,
        }),
    );

    Ok(())
}
