use tauri::{AppHandle, Emitter, State};
use crate::models::pci::{SriovPf, SriovVf, SriovVfConfig};
use crate::services::sriov_service::SriovService;
use crate::state::app_state::AppState;

/// List all SR-IOV capable Physical Functions
#[tauri::command]
pub async fn list_sriov_pfs() -> Result<Vec<SriovPf>, String> {
    tracing::debug!("list_sriov_pfs command called");
    SriovService::list_sriov_pfs().map_err(|e| e.to_string())
}

/// List Virtual Functions for a Physical Function
#[tauri::command]
pub async fn list_sriov_vfs(
    state: State<'_, AppState>,
    pf_address: String,
) -> Result<Vec<SriovVf>, String> {
    tracing::debug!("list_sriov_vfs command called for PF: {}", pf_address);
    SriovService::list_vfs(&pf_address, &state.libvirt).map_err(|e| e.to_string())
}

/// Enable VFs on a Physical Function
#[tauri::command]
pub async fn enable_sriov_vfs(
    app: AppHandle,
    pf_address: String,
    num_vfs: u32,
) -> Result<(), String> {
    tracing::info!("enable_sriov_vfs command called: PF={}, num_vfs={}", pf_address, num_vfs);

    SriovService::enable_vfs(&pf_address, num_vfs).map_err(|e| e.to_string())?;

    let _ = app.emit("sriov-vfs-enabled", serde_json::json!({
        "pfAddress": pf_address,
        "numVfs": num_vfs,
    }));

    Ok(())
}

/// Configure a Virtual Function
#[tauri::command]
pub async fn configure_sriov_vf(
    app: AppHandle,
    config: SriovVfConfig,
) -> Result<(), String> {
    tracing::info!("configure_sriov_vf command called: interface={}, vf={}", config.pf_interface, config.vf_index);

    SriovService::configure_vf(&config).map_err(|e| e.to_string())?;

    let _ = app.emit("sriov-vf-configured", serde_json::json!({
        "pfInterface": config.pf_interface,
        "vfIndex": config.vf_index,
    }));

    Ok(())
}

/// Attach a VF to a VM
#[tauri::command]
pub async fn attach_sriov_vf(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    vf_address: String,
) -> Result<(), String> {
    tracing::info!("attach_sriov_vf command called: VM={}, VF={}", vm_id, vf_address);

    SriovService::attach_vf_to_vm(&state.libvirt, &vm_id, &vf_address)
        .map_err(|e| e.to_string())?;

    let _ = app.emit("sriov-vf-attached", serde_json::json!({
        "vmId": vm_id,
        "vfAddress": vf_address,
    }));

    Ok(())
}

/// Detach a VF from a VM
#[tauri::command]
pub async fn detach_sriov_vf(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    vf_address: String,
) -> Result<(), String> {
    tracing::info!("detach_sriov_vf command called: VM={}, VF={}", vm_id, vf_address);

    SriovService::detach_vf_from_vm(&state.libvirt, &vm_id, &vf_address)
        .map_err(|e| e.to_string())?;

    let _ = app.emit("sriov-vf-detached", serde_json::json!({
        "vmId": vm_id,
        "vfAddress": vf_address,
    }));

    Ok(())
}
