use crate::models::operation::TargetIdentity;
use crate::services::serial_console_service::{SerialConsoleInfo, SerialConsoleService};
use crate::state::app_state::AppState;
use crate::utils::error::{AppError, SafeFailure};
use tauri::State;

fn console_target(vm_id: &str) -> TargetIdentity {
    TargetIdentity {
        resource_kind: "vm".to_string(),
        stable_id: vm_id.to_string(),
        display_name: None,
    }
}

/// Get serial console information for a VM
#[tauri::command]
pub async fn get_serial_console_info(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<SerialConsoleInfo, SafeFailure> {
    tracing::info!("get_serial_console_info command called for VM: {}", vm_id);

    let operation = state
        .resolve_console_operation(Some(console_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    SerialConsoleService::get_serial_console_info(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Open a serial console connection to a VM
#[tauri::command]
pub async fn open_serial_console(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<SerialConsoleInfo, SafeFailure> {
    tracing::info!("open_serial_console command called for VM: {}", vm_id);

    let operation = state
        .resolve_console_operation(Some(console_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    // First get the console info through the selected local-console connection.
    let info = SerialConsoleService::get_serial_console_info(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    if !info.active {
        return Err(SafeFailure::from(AppError::InvalidVmState(
            "serial console is inactive".to_string(),
        ))
        .with_context(&operation.context));
    }

    let connection_uri = operation
        .connection
        .get_uri()
        .map_err(|error| SafeFailure::from(AppError::LibvirtError(error.to_string())))
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Open the connection with vm_name for virsh console
    state
        .serial_console
        .open_connection(&vm_id, &info.vm_name, &info.pty_path, &connection_uri)
        .map_err(|error| SafeFailure::from(AppError::Other(error.to_string())))
        .map_err(|failure| failure.with_context(&operation.context))?;

    Ok(info)
}

/// Close a serial console connection
#[tauri::command]
pub async fn close_serial_console(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), SafeFailure> {
    tracing::info!("close_serial_console command called for VM: {}", vm_id);

    state
        .serial_console
        .close_connection(&vm_id)
        .map_err(|error| SafeFailure::from(AppError::Other(error.to_string())))
}

/// Read output from serial console
#[tauri::command]
pub async fn read_serial_console(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<String, SafeFailure> {
    state
        .serial_console
        .read_output(&vm_id)
        .map_err(|error| SafeFailure::from(AppError::Other(error.to_string())))
}

/// Write input to serial console
#[tauri::command]
pub async fn write_serial_console(
    state: State<'_, AppState>,
    vm_id: String,
    input: String,
) -> Result<(), SafeFailure> {
    tracing::debug!(
        "write_serial_console: writing {} bytes to VM {}",
        input.len(),
        vm_id
    );

    state
        .serial_console
        .write_input(&vm_id, &input)
        .map_err(|error| SafeFailure::from(AppError::Other(error.to_string())))
}

/// Check if serial console is connected for a VM
#[tauri::command]
pub async fn is_serial_console_connected(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<bool, SafeFailure> {
    Ok(state.serial_console.is_connected(&vm_id))
}
