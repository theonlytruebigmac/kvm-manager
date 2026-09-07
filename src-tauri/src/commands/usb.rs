use crate::models::usb::UsbDevice;
use crate::services::usb_service;
use crate::state::app_state::AppState;
use crate::utils::error::{AppError, SafeFailure};
use tauri::{AppHandle, Emitter, State};

/// Get list of USB devices connected to the host
#[tauri::command]
pub async fn list_usb_devices(state: State<'_, AppState>) -> Result<Vec<UsbDevice>, SafeFailure> {
    tracing::debug!("list_usb_devices command called");
    let operation = state
        .resolve_host_local_operation(None)
        .map_err(SafeFailure::from)?;
    usb_service::list_usb_devices(&operation.connection)
        .map_err(|error| SafeFailure::from(AppError::Other(error)))
}

/// Attach a USB device to a VM by vendor and product ID
#[tauri::command]
pub async fn attach_usb_device(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    vendor_id: String,
    product_id: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "attach_usb_device command called for VM: {} device: {}:{}",
        vm_id,
        vendor_id,
        product_id
    );
    let operation = state
        .resolve_host_local_operation(None)
        .map_err(SafeFailure::from)?;

    let stable_id = format!("{vm_id}/{vendor_id}:{product_id}");
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "attach_usb_device",
        "usb_device",
        &stable_id,
        None,
        "attach",
    )
    .map_err(SafeFailure::from)?;
    usb_service::attach_usb_device(&operation.connection, &vm_id, &vendor_id, &product_id)
        .map_err(|error| SafeFailure::from(AppError::Other(error)))
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-usb-device-attached",
        serde_json::json!({
            "vmId": vm_id,
            "vendorId": vendor_id,
            "productId": product_id,
        }),
    );

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
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "detach_usb_device command called for VM: {} device: {}:{}",
        vm_id,
        vendor_id,
        product_id
    );
    let operation = state
        .resolve_host_local_operation(None)
        .map_err(SafeFailure::from)?;

    let stable_id = format!("{vm_id}/{vendor_id}:{product_id}");
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "detach_usb_device",
        "usb_device",
        &stable_id,
        None,
        "detach",
    )
    .map_err(SafeFailure::from)?;
    usb_service::detach_usb_device(&operation.connection, &vm_id, &vendor_id, &product_id)
        .map_err(|error| SafeFailure::from(AppError::Other(error)))
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-usb-device-detached",
        serde_json::json!({
            "vmId": vm_id,
            "vendorId": vendor_id,
            "productId": product_id,
        }),
    );

    Ok(())
}

/// Get list of USB devices attached to a VM
#[tauri::command]
pub async fn get_vm_usb_devices(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<Vec<UsbDevice>, SafeFailure> {
    tracing::debug!("get_vm_usb_devices command called for VM: {}", vm_id);
    let operation = state
        .resolve_host_local_operation(None)
        .map_err(SafeFailure::from)?;
    usb_service::get_vm_usb_devices(&operation.connection, &vm_id)
        .map_err(|error| SafeFailure::from(AppError::Other(error)))
        .map_err(|failure| failure.with_context(&operation.context))
}
