//! Guest Agent Commands
//!
//! Tauri commands for interacting with guest agents running inside VMs.

use crate::state::app_state::AppState;
use crate::services::guest_agent_service::{
    AgentInfo, SystemInfo, NetworkInfo, DiskUsageInfo, ExecCommandResult,
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
    timeout_seconds: Option<u64>,
    state: State<'_, AppState>,
) -> Result<ExecCommandResult, String> {
    let timeout = timeout_seconds.unwrap_or(30);

    state
        .guest_agent
        .exec_command(&vm_name, &command, args, timeout)
        .await
        .map_err(|e| format!("Failed to execute command: {}", e))
}

/// Read a file from the guest
#[tauri::command]
pub async fn read_guest_file(
    vm_name: String,
    path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    state
        .guest_agent
        .read_file(&vm_name, &path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))
}

/// Write a file to the guest
#[tauri::command]
pub async fn write_guest_file(
    vm_name: String,
    path: String,
    content: String,
    create_dirs: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let create = create_dirs.unwrap_or(false);

    state
        .guest_agent
        .write_file(&vm_name, &path, &content, create)
        .await
        .map_err(|e| format!("Failed to write file: {}", e))
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
