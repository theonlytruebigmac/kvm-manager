use crate::models::pci::{IommuGroup, IommuStatus, PciDevice};
use crate::services::pci_service::{PciService, VfioStatus};
use crate::state::app_state::AppState;
use crate::utils::error::SafeFailure;
use tauri::{AppHandle, Emitter, State};

fn require_host_device(state: &AppState) -> Result<(), Box<SafeFailure>> {
    state
        .resolve_host_local_operation(None)
        .map(|_| ())
        .map_err(|error| Box::new(SafeFailure::from(error)))
}

/// List all PCI devices on the host
#[tauri::command]
pub async fn list_pci_devices(state: State<'_, AppState>) -> Result<Vec<PciDevice>, SafeFailure> {
    tracing::debug!("list_pci_devices command called");
    let operation = state
        .resolve_host_local_operation(None)
        .map_err(SafeFailure::from)?;

    PciService::list_pci_devices(&operation.connection).map_err(SafeFailure::from)
}

/// Check IOMMU status on the host
#[tauri::command]
pub async fn check_iommu_status(state: State<'_, AppState>) -> Result<IommuStatus, SafeFailure> {
    tracing::debug!("check_iommu_status command called");
    require_host_device(&state).map_err(|failure| *failure)?;

    PciService::check_iommu_status().map_err(SafeFailure::from)
}

/// Get IOMMU groups
#[tauri::command]
pub async fn get_iommu_groups(state: State<'_, AppState>) -> Result<Vec<IommuGroup>, SafeFailure> {
    tracing::debug!("get_iommu_groups command called");
    require_host_device(&state).map_err(|failure| *failure)?;

    PciService::get_iommu_groups().map_err(SafeFailure::from)
}

/// Attach a PCI device to a VM
#[tauri::command]
pub async fn attach_pci_device(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    pci_address: String,
    managed: bool,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "attach_pci_device command called for VM: {} device: {}",
        vm_id,
        pci_address
    );
    let operation = state
        .resolve_host_local_operation(None)
        .map_err(SafeFailure::from)?;

    let stable_id = format!("{vm_id}/{pci_address}");
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "attach_pci_device",
        "pci_device",
        &stable_id,
        None,
        &format!("attach;managed={managed}"),
    )
    .map_err(SafeFailure::from)?;
    PciService::attach_pci_device(&operation.connection, &vm_id, &pci_address, managed)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-pci-device-attached",
        serde_json::json!({
            "vmId": vm_id,
            "pciAddress": pci_address,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

    Ok(())
}

/// Detach a PCI device from a VM
#[tauri::command]
pub async fn detach_pci_device(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    pci_address: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "detach_pci_device command called for VM: {} device: {}",
        vm_id,
        pci_address
    );
    let operation = state
        .resolve_host_local_operation(None)
        .map_err(SafeFailure::from)?;

    let stable_id = format!("{vm_id}/{pci_address}");
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "detach_pci_device",
        "pci_device",
        &stable_id,
        None,
        "detach",
    )
    .map_err(SafeFailure::from)?;
    PciService::detach_pci_device(&operation.connection, &vm_id, &pci_address)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-pci-device-detached",
        serde_json::json!({
            "vmId": vm_id,
            "pciAddress": pci_address,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

    Ok(())
}

/// Get VFIO binding status for a PCI device
#[tauri::command]
pub async fn get_vfio_status(
    state: State<'_, AppState>,
    pci_address: String,
) -> Result<VfioStatus, SafeFailure> {
    tracing::debug!("get_vfio_status command called");
    require_host_device(&state).map_err(|failure| *failure)?;

    PciService::get_vfio_status(&pci_address).map_err(SafeFailure::from)
}

/// Bind a PCI device to the vfio-pci driver for passthrough
#[tauri::command]
pub async fn bind_to_vfio(
    app: AppHandle,
    state: State<'_, AppState>,
    pci_address: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!("bind_to_vfio command called");
    require_host_device(&state).map_err(|failure| *failure)?;

    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "bind_to_vfio",
        "host_device",
        &pci_address,
        None,
        "bind-vfio",
    )
    .map_err(SafeFailure::from)?;
    PciService::bind_to_vfio(&pci_address).map_err(SafeFailure::from)?;

    // Emit event
    let _ = app.emit(
        "pci-device-vfio-bound",
        serde_json::json!({
            "pciAddress": pci_address,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

    Ok(())
}

/// Unbind a PCI device from vfio-pci and restore original driver
#[tauri::command]
pub async fn unbind_from_vfio(
    app: AppHandle,
    state: State<'_, AppState>,
    pci_address: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "unbind_from_vfio command called for device: {}",
        pci_address
    );
    require_host_device(&state).map_err(|failure| *failure)?;

    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "unbind_from_vfio",
        "host_device",
        &pci_address,
        None,
        "unbind-vfio",
    )
    .map_err(SafeFailure::from)?;
    PciService::unbind_from_vfio(&pci_address).map_err(SafeFailure::from)?;

    // Emit event
    let _ = app.emit(
        "pci-device-vfio-unbound",
        serde_json::json!({
            "pciAddress": pci_address,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

    Ok(())
}
