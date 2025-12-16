//! Guest Agent Commands
//!
//! Tauri commands for interacting with QEMU Guest Agent running inside VMs.
//! Uses virsh qemu-agent-command functionality.

use crate::state::app_state::AppState;
use crate::services::guest_agent_service::{
    AgentInfo, SystemInfo, NetworkInfo, DiskUsageInfo, ExecCommandResult,
    GuestCpuStats, GuestDiskStats, GuestUser, GuestTimezone, GuestFullInfo,
};
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct GuestAgentStatus {
    pub available: bool,
    pub agent_info: Option<AgentInfo>,
}

/// Check if guest agent is available for a VM
#[tauri::command]
pub async fn check_guest_agent_status(
    vm_name: String,
    state: State<'_, AppState>,
) -> Result<GuestAgentStatus, String> {
    let available = state
        .guest_agent
        .is_agent_available(&vm_name)
        .await;

    if !available {
        return Ok(GuestAgentStatus {
            available: false,
            agent_info: None,
        });
    }

    // Get agent info if available
    let agent_info = state
        .guest_agent
        .get_agent_info(&vm_name)
        .await
        .ok();

    Ok(GuestAgentStatus {
        available: true,
        agent_info,
    })
}

/// Get guest system information
#[tauri::command]
pub async fn get_guest_system_info(
    vm_name: String,
    state: State<'_, AppState>,
) -> Result<SystemInfo, String> {
    state
        .guest_agent
        .get_system_info(&vm_name)
        .await
        .map_err(|e| format!("Failed to get guest system info: {}", e))
}

/// Get guest network information
#[tauri::command]
pub async fn get_guest_network_info(
    vm_name: String,
    state: State<'_, AppState>,
) -> Result<NetworkInfo, String> {
    state
        .guest_agent
        .get_network_info(&vm_name)
        .await
        .map_err(|e| format!("Failed to get guest network info: {}", e))
}

/// Get guest disk usage
#[tauri::command]
pub async fn get_guest_disk_usage(
    vm_name: String,
    state: State<'_, AppState>,
) -> Result<DiskUsageInfo, String> {
    state
        .guest_agent
        .get_disk_usage(&vm_name)
        .await
        .map_err(|e| format!("Failed to get guest disk usage: {}", e))
}

/// Execute a command in the guest
#[tauri::command]
pub async fn execute_guest_command(
    vm_name: String,
    command: String,
    args: Vec<String>,
    _timeout_seconds: Option<u64>,
    state: State<'_, AppState>,
) -> Result<ExecCommandResult, String> {
    state
        .guest_agent
        .exec_command(&vm_name, &command, args)
        .await
        .map_err(|e| format!("Failed to execute command: {}", e))
}

/// Read a file from the guest (using guest-exec with cat)
#[tauri::command]
pub async fn read_guest_file(
    vm_name: String,
    path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let result = state
        .guest_agent
        .exec_command(&vm_name, "cat", vec![path])
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    if result.exit_code != 0 {
        return Err(format!("Failed to read file: {}", result.stderr));
    }
    Ok(result.stdout)
}

/// Write a file to the guest (using guest-file-open/write/close)
#[tauri::command]
pub async fn write_guest_file(
    vm_name: String,
    path: String,
    content: String,
    _create_dirs: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Use echo with redirect via guest-exec
    let result = state
        .guest_agent
        .exec_command(&vm_name, "bash", vec![
            "-c".to_string(),
            format!("echo '{}' > {}", content.replace("'", "'\\''"), path)
        ])
        .await
        .map_err(|e| format!("Failed to write file: {}", e))?;

    if result.exit_code != 0 {
        return Err(format!("Failed to write file: {}", result.stderr));
    }
    Ok(())
}

/// Shutdown guest via agent (more graceful than ACPI)
#[tauri::command]
pub async fn guest_agent_shutdown(
    vm_name: String,
    force: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let force_shutdown = force.unwrap_or(false);

    state
        .guest_agent
        .shutdown(&vm_name, force_shutdown)
        .await
        .map_err(|e| format!("Failed to shutdown guest: {}", e))
}

/// Reboot guest via agent
#[tauri::command]
pub async fn guest_agent_reboot(
    vm_name: String,
    force: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let force_reboot = force.unwrap_or(false);

    state
        .guest_agent
        .reboot(&vm_name, force_reboot)
        .await
        .map_err(|e| format!("Failed to reboot guest: {}", e))
}

/// Get CPU statistics from guest
#[tauri::command]
pub async fn get_guest_cpu_stats(
    vm_name: String,
    state: State<'_, AppState>,
) -> Result<GuestCpuStats, String> {
    state
        .guest_agent
        .get_cpu_stats(&vm_name)
        .await
        .map_err(|e| format!("Failed to get guest CPU stats: {}", e))
}

/// Get disk I/O statistics from guest
#[tauri::command]
pub async fn get_guest_disk_stats(
    vm_name: String,
    state: State<'_, AppState>,
) -> Result<GuestDiskStats, String> {
    state
        .guest_agent
        .get_disk_stats(&vm_name)
        .await
        .map_err(|e| format!("Failed to get guest disk stats: {}", e))
}

/// Get logged-in users from guest
#[tauri::command]
pub async fn get_guest_users(
    vm_name: String,
    state: State<'_, AppState>,
) -> Result<Vec<GuestUser>, String> {
    state
        .guest_agent
        .get_users(&vm_name)
        .await
        .map_err(|e| format!("Failed to get guest users: {}", e))
}

/// Get timezone from guest
#[tauri::command]
pub async fn get_guest_timezone(
    vm_name: String,
    state: State<'_, AppState>,
) -> Result<GuestTimezone, String> {
    state
        .guest_agent
        .get_timezone(&vm_name)
        .await
        .map_err(|e| format!("Failed to get guest timezone: {}", e))
}

/// Get full guest information (combines multiple queries)
#[tauri::command]
pub async fn get_guest_full_info(
    vm_name: String,
    state: State<'_, AppState>,
) -> Result<GuestFullInfo, String> {
    state
        .guest_agent
        .get_full_info(&vm_name)
        .await
        .map_err(|e| format!("Failed to get full guest info: {}", e))
}
