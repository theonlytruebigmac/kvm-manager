use tauri::State;
use crate::state::app_state::AppState;
use crate::services::serial_console_service::{SerialConsoleService, SerialConsoleInfo};

/// Get serial console information for a VM
#[tauri::command]
pub async fn get_serial_console_info(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<SerialConsoleInfo, String> {
    tracing::info!("get_serial_console_info command called for VM: {}", vm_id);

    SerialConsoleService::get_serial_console_info(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())
}

/// Open a serial console connection to a VM
#[tauri::command]
pub async fn open_serial_console(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<SerialConsoleInfo, String> {
    tracing::info!("open_serial_console command called for VM: {}", vm_id);

    // First get the console info
    let info = SerialConsoleService::get_serial_console_info(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())?;

    if !info.active {
        return Err("VM is not running".to_string());
    }

    // Open the connection with vm_name for virsh console
    state.serial_console.open_connection(&vm_id, &info.vm_name, &info.pty_path)
        .map_err(|e| e.to_string())?;

    Ok(info)
}

/// Close a serial console connection
#[tauri::command]
pub async fn close_serial_console(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), String> {
    tracing::info!("close_serial_console command called for VM: {}", vm_id);

    state.serial_console.close_connection(&vm_id)
        .map_err(|e| e.to_string())
}

/// Read output from serial console
#[tauri::command]
pub async fn read_serial_console(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<String, String> {
    state.serial_console.read_output(&vm_id)
        .map_err(|e| e.to_string())
}

/// Write input to serial console
#[tauri::command]
pub async fn write_serial_console(
    state: State<'_, AppState>,
    vm_id: String,
    input: String,
) -> Result<(), String> {
    tracing::debug!("write_serial_console: writing {} bytes to VM {}", input.len(), vm_id);

    state.serial_console.write_input(&vm_id, &input)
        .map_err(|e| e.to_string())
}

/// Check if serial console is connected for a VM
#[tauri::command]
pub async fn is_serial_console_connected(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<bool, String> {
    Ok(state.serial_console.is_connected(&vm_id))
}
