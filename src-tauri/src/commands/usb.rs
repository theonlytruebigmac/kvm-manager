use tauri::{AppHandle, Emitter, State};
use crate::models::usb::UsbDevice;
use crate::services::usb_service;
use crate::state::app_state::AppState;

/// Get list of USB devices connected to the host
#[tauri::command]
pub async fn list_usb_devices(state: State<'_, AppState>) -> Result<Vec<UsbDevice>, String> {
    tracing::debug!("list_usb_devices command called");
    usb_service::list_usb_devices(&state.libvirt)
}

/// Attach a USB device to a VM by vendor and product ID
#[tauri::command]
pub async fn attach_usb_device(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    vendor_id: String,
    product_id: String,
) -> Result<(), String> {
    tracing::info!("attach_usb_device command called for VM: {} device: {}:{}", vm_id, vendor_id, product_id);

    usb_service::attach_usb_device(&state.libvirt, &vm_id, &vendor_id, &product_id)?;

    // Emit event
    let _ = app.emit("vm-usb-device-attached", serde_json::json!({
        "vmId": vm_id,
        "vendorId": vendor_id,
        "productId": product_id,
    }));

    Ok(())
}

/// Detach a USB device from a VM
#[tauri::command]
pub async fn detach_usb_device(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    vendor_id: String,
    product_id: String,
) -> Result<(), String> {
    tracing::info!("detach_usb_device command called for VM: {} device: {}:{}", vm_id, vendor_id, product_id);

    usb_service::detach_usb_device(&state.libvirt, &vm_id, &vendor_id, &product_id)?;

    // Emit event
    let _ = app.emit("vm-usb-device-detached", serde_json::json!({
        "vmId": vm_id,
        "vendorId": vendor_id,
        "productId": product_id,
    }));

    Ok(())
}

/// Get list of USB devices attached to a VM
#[tauri::command]
pub async fn get_vm_usb_devices(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<Vec<UsbDevice>, String> {
    tracing::debug!("get_vm_usb_devices command called for VM: {}", vm_id);
    usb_service::get_vm_usb_devices(&state.libvirt, &vm_id)
}
