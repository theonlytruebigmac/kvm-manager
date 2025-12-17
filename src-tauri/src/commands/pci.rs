use tauri::{AppHandle, Emitter, State};
use crate::models::pci::{PciDevice, IommuGroup, IommuStatus};
use crate::services::pci_service::{PciService, VfioStatus};
use crate::state::app_state::AppState;

/// List all PCI devices on the host
#[tauri::command]
pub async fn list_pci_devices(state: State<'_, AppState>) -> Result<Vec<PciDevice>, String> {
    tracing::debug!("list_pci_devices command called");

    PciService::list_pci_devices(&state.libvirt)
        .map_err(|e| e.to_string())
}

/// Check IOMMU status on the host
#[tauri::command]
pub async fn check_iommu_status() -> Result<IommuStatus, String> {
    tracing::debug!("check_iommu_status command called");

    PciService::check_iommu_status()
        .map_err(|e| e.to_string())
}

/// Get IOMMU groups
#[tauri::command]
pub async fn get_iommu_groups() -> Result<Vec<IommuGroup>, String> {
    tracing::debug!("get_iommu_groups command called");

    PciService::get_iommu_groups()
        .map_err(|e| e.to_string())
}

/// Attach a PCI device to a VM
#[tauri::command]
pub async fn attach_pci_device(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    pci_address: String,
    managed: bool,
) -> Result<(), String> {
    tracing::info!("attach_pci_device command called for VM: {} device: {}", vm_id, pci_address);

    PciService::attach_pci_device(&state.libvirt, &vm_id, &pci_address, managed)
        .map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("vm-pci-device-attached", serde_json::json!({
        "vmId": vm_id,
        "pciAddress": pci_address,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(())
}

/// Detach a PCI device from a VM
#[tauri::command]
pub async fn detach_pci_device(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    pci_address: String,
) -> Result<(), String> {
    tracing::info!("detach_pci_device command called for VM: {} device: {}", vm_id, pci_address);

    PciService::detach_pci_device(&state.libvirt, &vm_id, &pci_address)
        .map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("vm-pci-device-detached", serde_json::json!({
        "vmId": vm_id,
        "pciAddress": pci_address,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(())
}

/// Get VFIO binding status for a PCI device
#[tauri::command]
pub async fn get_vfio_status(pci_address: String) -> Result<VfioStatus, String> {
    tracing::debug!("get_vfio_status command called for device: {}", pci_address);

    PciService::get_vfio_status(&pci_address)
        .map_err(|e| e.to_string())
}

/// Bind a PCI device to the vfio-pci driver for passthrough
#[tauri::command]
pub async fn bind_to_vfio(
    app: AppHandle,
    pci_address: String,
) -> Result<(), String> {
    tracing::info!("bind_to_vfio command called for device: {}", pci_address);

    PciService::bind_to_vfio(&pci_address)
        .map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("pci-device-vfio-bound", serde_json::json!({
        "pciAddress": pci_address,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(())
}

/// Unbind a PCI device from vfio-pci and restore original driver
#[tauri::command]
pub async fn unbind_from_vfio(
    app: AppHandle,
    pci_address: String,
) -> Result<(), String> {
    tracing::info!("unbind_from_vfio command called for device: {}", pci_address);

    PciService::unbind_from_vfio(&pci_address)
        .map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("pci-device-vfio-unbound", serde_json::json!({
        "pciAddress": pci_address,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(())
}
