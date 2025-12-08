use tauri::{AppHandle, State, Emitter};
use serde::Serialize;
use crate::models::vm::{VM, VmState, VmStats};
use crate::services::vm_service::VmService;
use crate::state::app_state::AppState;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct VmStateChangedPayload {
    vm_id: String,
    vm_name: String,
    old_state: VmState,
    new_state: VmState,
    timestamp: i64,
}

/// Get all VMs (active and inactive)
#[tauri::command]
pub async fn get_vms(state: State<'_, AppState>) -> Result<Vec<VM>, String> {
    tracing::debug!("get_vms command called");

    VmService::list_vms(&state.libvirt)
        .map_err(|e| e.to_string())
}

/// Get a single VM by ID
#[tauri::command]
pub async fn get_vm(state: State<'_, AppState>, vm_id: String) -> Result<VM, String> {
    tracing::debug!("get_vm command called for VM: {}", vm_id);

    VmService::get_vm(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())
}

/// Start a VM
#[tauri::command]
pub async fn start_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), String> {
    tracing::info!("start_vm command called for VM: {}", vm_id);

    // Get VM info before state change
    let vm = VmService::get_vm(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())?;
    let old_state = vm.state.clone();
    let vm_name = vm.name.clone();

    VmService::start_vm(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("vm-state-changed", VmStateChangedPayload {
        vm_id: vm_id.clone(),
        vm_name,
        old_state,
        new_state: VmState::Running,
        timestamp: chrono::Utc::now().timestamp_millis(),
    });

    Ok(())
}

/// Stop a VM
#[tauri::command]
pub async fn stop_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), String> {
    tracing::info!("stop_vm command called for VM: {}", vm_id);

    // Get VM info before state change
    let vm = VmService::get_vm(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())?;
    let old_state = vm.state.clone();
    let vm_name = vm.name.clone();

    VmService::stop_vm(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("vm-state-changed", VmStateChangedPayload {
        vm_id: vm_id.clone(),
        vm_name,
        old_state,
        new_state: VmState::Stopped,
        timestamp: chrono::Utc::now().timestamp_millis(),
    });

    Ok(())
}

/// Force stop a VM (immediate power off)
#[tauri::command]
pub async fn force_stop_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), String> {
    tracing::info!("force_stop_vm command called for VM: {}", vm_id);

    // Get VM info before state change
    let vm = VmService::get_vm(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())?;
    let old_state = vm.state.clone();
    let vm_name = vm.name.clone();

    VmService::force_stop_vm(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("vm-state-changed", VmStateChangedPayload {
        vm_id: vm_id.clone(),
        vm_name,
        old_state,
        new_state: VmState::Stopped,
        timestamp: chrono::Utc::now().timestamp_millis(),
    });

    Ok(())
}

/// Pause a VM
#[tauri::command]
pub async fn pause_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), String> {
    tracing::info!("pause_vm command called for VM: {}", vm_id);

    // Get VM info before state change
    let vm = VmService::get_vm(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())?;
    let old_state = vm.state.clone();
    let vm_name = vm.name.clone();

    VmService::pause_vm(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("vm-state-changed", VmStateChangedPayload {
        vm_id: vm_id.clone(),
        vm_name,
        old_state,
        new_state: VmState::Paused,
        timestamp: chrono::Utc::now().timestamp_millis(),
    });

    Ok(())
}

/// Resume a paused VM
#[tauri::command]
pub async fn resume_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), String> {
    tracing::info!("resume_vm command called for VM: {}", vm_id);

    // Get VM info before state change
    let vm = VmService::get_vm(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())?;
    let old_state = vm.state.clone();
    let vm_name = vm.name.clone();

    VmService::resume_vm(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("vm-state-changed", VmStateChangedPayload {
        vm_id: vm_id.clone(),
        vm_name,
        old_state,
        new_state: VmState::Running,
        timestamp: chrono::Utc::now().timestamp_millis(),
    });

    Ok(())
}

/// Reboot a VM
#[tauri::command]
pub async fn reboot_vm(state: State<'_, AppState>, vm_id: String) -> Result<(), String> {
    tracing::info!("reboot_vm command called for VM: {}", vm_id);

    VmService::reboot_vm(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())
}

/// Delete a VM
#[tauri::command]
pub async fn delete_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    delete_disks: bool,
) -> Result<(), String> {
    tracing::info!("delete_vm command called for VM: {} (delete_disks: {})", vm_id, delete_disks);

    // Get VM info before deletion
    let vm = VmService::get_vm(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())?;
    let vm_name = vm.name.clone();

    VmService::delete_vm(&state.libvirt, &vm_id, delete_disks)
        .map_err(|e| e.to_string())?;

    // Emit event (use a custom event for deletion)
    let _ = app.emit("vm-deleted", serde_json::json!({
        "vmId": vm_id,
        "vmName": vm_name,
        "deleteDisks": delete_disks,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(())
}

/// Clone an existing VM
#[tauri::command]
pub async fn clone_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    source_vm_id: String,
    new_name: String,
) -> Result<String, String> {
    tracing::info!("clone_vm command called: source_vm_id={}, new_name={}", source_vm_id, new_name);

    let cloned_vm_id = VmService::clone_vm(&state.libvirt, &source_vm_id, &new_name)
        .map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("vm-cloned", serde_json::json!({
        "sourceVmId": source_vm_id,
        "clonedVmId": cloned_vm_id,
        "newName": new_name,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(cloned_vm_id)
}

/// Create a new VM
#[tauri::command]
pub async fn create_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    config: crate::models::vm::VmConfig,
) -> Result<String, String> {
    tracing::info!("create_vm command called for VM: {}", config.name);

    let vm_id = VmService::create_vm(&state.libvirt, config)
        .map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("vm-created", serde_json::json!({
        "vmId": vm_id,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(vm_id)
}

/// Get VM performance statistics
#[tauri::command]
pub async fn get_vm_stats(state: State<'_, AppState>, vm_id: String) -> Result<VmStats, String> {
    tracing::debug!("get_vm_stats command called for VM: {}", vm_id);

    VmService::get_vm_stats(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())
}

/// Add tags to a VM
#[tauri::command]
pub async fn add_vm_tags(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    tags: Vec<String>,
) -> Result<(), String> {
    tracing::info!("add_vm_tags command called for VM: {}", vm_id);

    VmService::add_vm_tags(&state.libvirt, &vm_id, tags.clone())
        .map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("vm-tags-updated", serde_json::json!({
        "vmId": vm_id,
        "tags": tags,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(())
}

/// Remove tags from a VM
#[tauri::command]
pub async fn remove_vm_tags(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    tags: Vec<String>,
) -> Result<(), String> {
    tracing::info!("remove_vm_tags command called for VM: {}", vm_id);

    VmService::remove_vm_tags(&state.libvirt, &vm_id, tags.clone())
        .map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("vm-tags-updated", serde_json::json!({
        "vmId": vm_id,
        "removedTags": tags,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(())
}
