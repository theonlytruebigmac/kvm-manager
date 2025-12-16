use tauri::{AppHandle, State, Emitter};
use serde::Serialize;
use crate::models::vm::{VM, VmState, VmStats};
use crate::services::vm_service::{VmService, KernelBootSettings, HugepagesSettings, HugepageInfo, UsbRedirectionInfo, EvdevDevice, CpuModelConfig};
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

/// Hibernate (managed save) a VM - saves state to disk and stops
#[tauri::command]
pub async fn hibernate_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), String> {
    tracing::info!("hibernate_vm command called for VM: {}", vm_id);

    // Get VM info before state change
    let vm = VmService::get_vm(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())?;
    let old_state = vm.state.clone();
    let vm_name = vm.name.clone();

    VmService::hibernate_vm(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())?;

    // Emit event - VM is now stopped but with saved state
    let _ = app.emit("vm-state-changed", VmStateChangedPayload {
        vm_id: vm_id.clone(),
        vm_name,
        old_state,
        new_state: VmState::Stopped,
        timestamp: chrono::Utc::now().timestamp_millis(),
    });

    Ok(())
}

/// Check if VM has a managed save (hibernated state)
#[tauri::command]
pub async fn has_managed_save(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<bool, String> {
    tracing::debug!("has_managed_save command called for VM: {}", vm_id);

    VmService::has_managed_save(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())
}

/// Remove managed save (discard hibernated state)
#[tauri::command]
pub async fn remove_managed_save(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), String> {
    tracing::info!("remove_managed_save command called for VM: {}", vm_id);

    VmService::remove_managed_save(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())
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
    delete_snapshots: bool,
) -> Result<(), String> {
    tracing::info!("delete_vm command called for VM: {} (delete_disks: {}, delete_snapshots: {})", vm_id, delete_disks, delete_snapshots);

    // Get VM info before deletion
    let vm = VmService::get_vm(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())?;
    let vm_name = vm.name.clone();

    // Delete snapshots if requested
    if delete_snapshots {
        tracing::info!("Deleting all snapshots for VM: {}", vm_id);
        use crate::services::snapshot_service::SnapshotService;

        match SnapshotService::list_snapshots(&state.libvirt, &vm_id) {
            Ok(snapshots) => {
                for snapshot in snapshots {
                    if let Err(e) = SnapshotService::delete_snapshot(&state.libvirt, &vm_id, &snapshot.name) {
                        tracing::warn!("Failed to delete snapshot {}: {}", snapshot.name, e);
                    } else {
                        tracing::info!("Deleted snapshot: {}", snapshot.name);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to list snapshots for deletion: {}", e);
            }
        }
    }

    VmService::delete_vm(&state.libvirt, &vm_id, delete_disks)
        .map_err(|e| e.to_string())?;

    // Emit event (use a custom event for deletion)
    let _ = app.emit("vm-deleted", serde_json::json!({
        "vmId": vm_id,
        "vmName": vm_name,
        "deleteDisks": delete_disks,
        "deleteSnapshots": delete_snapshots,
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

/// Export VM configuration to XML
#[tauri::command]
pub async fn export_vm(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<String, String> {
    tracing::info!("export_vm command called for VM: {}", vm_id);

    VmService::export_vm(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())
}

/// Import VM from XML configuration
#[tauri::command]
pub async fn import_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    xml: String,
) -> Result<String, String> {
    tracing::info!("import_vm command called");

    let vm_id = VmService::import_vm(&state.libvirt, &xml)
        .map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("vm-imported", serde_json::json!({
        "vmId": vm_id,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(vm_id)
}

/// Attach a disk to a VM
#[tauri::command]
pub async fn attach_disk(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    disk_path: String,
    device_target: String,
    bus_type: String,
) -> Result<(), String> {
    tracing::info!("attach_disk command called for VM: {}", vm_id);

    VmService::attach_disk(&state.libvirt, &vm_id, &disk_path, &device_target, &bus_type)
        .map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("vm-disk-attached", serde_json::json!({
        "vmId": vm_id,
        "diskPath": disk_path,
        "deviceTarget": device_target,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(())
}

/// Detach a disk from a VM
#[tauri::command]
pub async fn detach_disk(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    device_target: String,
) -> Result<(), String> {
    tracing::info!("detach_disk command called for VM: {}", vm_id);

    VmService::detach_disk(&state.libvirt, &vm_id, &device_target)
        .map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("vm-disk-detached", serde_json::json!({
        "vmId": vm_id,
        "deviceTarget": device_target,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(())
}

/// Update disk I/O settings (cache, io mode, discard, throttling)
#[tauri::command]
pub async fn update_disk_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    device_target: String,
    cache: Option<String>,
    io: Option<String>,
    discard: Option<String>,
    detect_zeroes: Option<String>,
    read_iops_sec: Option<u64>,
    write_iops_sec: Option<u64>,
    read_bytes_sec: Option<u64>,
    write_bytes_sec: Option<u64>,
) -> Result<(), String> {
    tracing::info!("update_disk_settings command called for VM: {}, disk: {}", vm_id, device_target);

    VmService::update_disk_settings(
        &state.libvirt,
        &vm_id,
        &device_target,
        cache.clone(),
        io.clone(),
        discard.clone(),
        detect_zeroes.clone(),
        read_iops_sec,
        write_iops_sec,
        read_bytes_sec,
        write_bytes_sec,
    )
    .map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("vm-disk-settings-updated", serde_json::json!({
        "vmId": vm_id,
        "deviceTarget": device_target,
        "cache": cache,
        "io": io,
        "discard": discard,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(())
}

/// Get direct kernel boot settings for a VM
#[tauri::command]
pub async fn get_kernel_boot_settings(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<KernelBootSettings, String> {
    tracing::info!("get_kernel_boot_settings command called for VM: {}", vm_id);
    VmService::get_kernel_boot_settings(&state.libvirt, &vm_id).map_err(|e| e.to_string())
}

/// Update direct kernel boot settings for a VM (VM must be shut off)
#[tauri::command]
pub async fn set_kernel_boot_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    enabled: bool,
    kernel_path: Option<String>,
    initrd_path: Option<String>,
    kernel_args: Option<String>,
    dtb_path: Option<String>,
) -> Result<(), String> {
    tracing::info!("set_kernel_boot_settings command called for VM: {}", vm_id);

    let settings = KernelBootSettings {
        enabled,
        kernel_path: kernel_path.clone(),
        initrd_path: initrd_path.clone(),
        kernel_args: kernel_args.clone(),
        dtb_path: dtb_path.clone(),
    };

    VmService::set_kernel_boot_settings(&state.libvirt, &vm_id, settings).map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("vm-kernel-boot-updated", serde_json::json!({
        "vmId": vm_id,
        "enabled": enabled,
        "kernelPath": kernel_path,
        "initrdPath": initrd_path,
        "kernelArgs": kernel_args,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(())
}

/// Get hugepages memory backing settings for a VM
#[tauri::command]
pub async fn get_hugepages_settings(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<HugepagesSettings, String> {
    tracing::info!("get_hugepages_settings command called for VM: {}", vm_id);
    VmService::get_hugepages_settings(&state.libvirt, &vm_id).map_err(|e| e.to_string())
}

/// Set hugepages memory backing for a VM (VM must be shut off)
#[tauri::command]
pub async fn set_hugepages(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    enabled: bool,
    size: Option<u64>,  // Size in KiB
) -> Result<(), String> {
    tracing::info!("set_hugepages command called for VM: {} enabled={} size={:?}", vm_id, enabled, size);

    VmService::set_hugepages(&state.libvirt, &vm_id, enabled, size).map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("vm-hugepages-updated", serde_json::json!({
        "vmId": vm_id,
        "enabled": enabled,
        "size": size,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(())
}

/// Get available hugepage sizes on the host system
#[tauri::command]
pub async fn get_host_hugepage_info() -> Result<Vec<HugepageInfo>, String> {
    tracing::info!("get_host_hugepage_info command called");
    VmService::get_host_hugepage_info().map_err(|e| e.to_string())
}

/// Update network interface bandwidth/QoS settings
#[tauri::command]
pub async fn update_interface_bandwidth(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    mac_address: String,
    inbound_average: Option<u64>,
    inbound_peak: Option<u64>,
    inbound_burst: Option<u64>,
    outbound_average: Option<u64>,
    outbound_peak: Option<u64>,
    outbound_burst: Option<u64>,
) -> Result<(), String> {
    tracing::info!("update_interface_bandwidth command called for VM: {}, interface: {}", vm_id, mac_address);

    VmService::update_interface_bandwidth(
        &state.libvirt,
        &vm_id,
        &mac_address,
        inbound_average,
        inbound_peak,
        inbound_burst,
        outbound_average,
        outbound_peak,
        outbound_burst,
    )
    .map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("vm-interface-bandwidth-updated", serde_json::json!({
        "vmId": vm_id,
        "macAddress": mac_address,
        "inboundAverage": inbound_average,
        "outboundAverage": outbound_average,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(())
}

/// Set the link state (up/down) for a network interface
#[tauri::command]
pub async fn set_interface_link_state(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    mac_address: String,
    link_up: bool,
) -> Result<(), String> {
    tracing::info!("set_interface_link_state command called for VM: {}, interface: {}, up: {}", vm_id, mac_address, link_up);

    VmService::set_interface_link_state(
        &state.libvirt,
        &vm_id,
        &mac_address,
        link_up,
    )
    .map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("vm-interface-link-state-changed", serde_json::json!({
        "vmId": vm_id,
        "macAddress": mac_address,
        "linkUp": link_up,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(())
}

/// Get the link state for a network interface
#[tauri::command]
pub async fn get_interface_link_state(
    state: State<'_, AppState>,
    vm_id: String,
    mac_address: String,
) -> Result<bool, String> {
    tracing::info!("get_interface_link_state command called for VM: {}, interface: {}", vm_id, mac_address);

    VmService::get_interface_link_state(
        &state.libvirt,
        &vm_id,
        &mac_address,
    )
    .map_err(|e| e.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchOperationResult {
    vm_id: String,
    vm_name: String,
    success: bool,
    error: Option<String>,
}

/// Start multiple VMs in batch
#[tauri::command]
pub async fn batch_start_vms(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_ids: Vec<String>,
) -> Result<Vec<BatchOperationResult>, String> {
    tracing::info!("batch_start_vms command called for {} VMs", vm_ids.len());

    let mut results = Vec::new();

    for vm_id in vm_ids {
        let vm = match VmService::get_vm(&state.libvirt, &vm_id) {
            Ok(v) => v,
            Err(e) => {
                results.push(BatchOperationResult {
                    vm_id: vm_id.clone(),
                    vm_name: "Unknown".to_string(),
                    success: false,
                    error: Some(e.to_string()),
                });
                continue;
            }
        };

        let old_state = vm.state.clone();
        let vm_name = vm.name.clone();

        let result = match VmService::start_vm(&state.libvirt, &vm_id) {
            Ok(_) => {
                let _ = app.emit("vm-state-changed", VmStateChangedPayload {
                    vm_id: vm_id.clone(),
                    vm_name: vm_name.clone(),
                    old_state,
                    new_state: VmState::Running,
                    timestamp: chrono::Utc::now().timestamp_millis(),
                });

                BatchOperationResult {
                    vm_id: vm_id.clone(),
                    vm_name,
                    success: true,
                    error: None,
                }
            }
            Err(e) => BatchOperationResult {
                vm_id: vm_id.clone(),
                vm_name,
                success: false,
                error: Some(e.to_string()),
            },
        };

        results.push(result);
    }

    Ok(results)
}

/// Stop multiple VMs in batch
#[tauri::command]
pub async fn batch_stop_vms(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_ids: Vec<String>,
    force: bool,
) -> Result<Vec<BatchOperationResult>, String> {
    tracing::info!("batch_stop_vms command called for {} VMs (force: {})", vm_ids.len(), force);

    let mut results = Vec::new();

    for vm_id in vm_ids {
        let vm = match VmService::get_vm(&state.libvirt, &vm_id) {
            Ok(v) => v,
            Err(e) => {
                results.push(BatchOperationResult {
                    vm_id: vm_id.clone(),
                    vm_name: "Unknown".to_string(),
                    success: false,
                    error: Some(e.to_string()),
                });
                continue;
            }
        };

        let old_state = vm.state.clone();
        let vm_name = vm.name.clone();

        let result = if force {
            match VmService::force_stop_vm(&state.libvirt, &vm_id) {
                Ok(_) => {
                    let _ = app.emit("vm-state-changed", VmStateChangedPayload {
                        vm_id: vm_id.clone(),
                        vm_name: vm_name.clone(),
                        old_state,
                        new_state: VmState::Stopped,
                        timestamp: chrono::Utc::now().timestamp_millis(),
                    });

                    BatchOperationResult {
                        vm_id: vm_id.clone(),
                        vm_name,
                        success: true,
                        error: None,
                    }
                }
                Err(e) => BatchOperationResult {
                    vm_id: vm_id.clone(),
                    vm_name,
                    success: false,
                    error: Some(e.to_string()),
                },
            }
        } else {
            match VmService::stop_vm(&state.libvirt, &vm_id) {
                Ok(_) => {
                    let _ = app.emit("vm-state-changed", VmStateChangedPayload {
                        vm_id: vm_id.clone(),
                        vm_name: vm_name.clone(),
                        old_state,
                        new_state: VmState::Stopped,
                        timestamp: chrono::Utc::now().timestamp_millis(),
                    });

                    BatchOperationResult {
                        vm_id: vm_id.clone(),
                        vm_name,
                        success: true,
                        error: None,
                    }
                }
                Err(e) => BatchOperationResult {
                    vm_id: vm_id.clone(),
                    vm_name,
                    success: false,
                    error: Some(e.to_string()),
                },
            }
        };

        results.push(result);
    }

    Ok(results)
}

/// Reboot multiple VMs in batch
#[tauri::command]
pub async fn batch_reboot_vms(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_ids: Vec<String>,
) -> Result<Vec<BatchOperationResult>, String> {
    tracing::info!("batch_reboot_vms command called for {} VMs", vm_ids.len());

    let mut results = Vec::new();

    for vm_id in vm_ids {
        let vm = match VmService::get_vm(&state.libvirt, &vm_id) {
            Ok(v) => v,
            Err(e) => {
                results.push(BatchOperationResult {
                    vm_id: vm_id.clone(),
                    vm_name: "Unknown".to_string(),
                    success: false,
                    error: Some(e.to_string()),
                });
                continue;
            }
        };

        let vm_name = vm.name.clone();

        let result = match VmService::reboot_vm(&state.libvirt, &vm_id) {
            Ok(_) => {
                let _ = app.emit("vm-rebooted", serde_json::json!({
                    "vmId": vm_id.clone(),
                    "vmName": vm_name.clone(),
                    "timestamp": chrono::Utc::now().timestamp_millis(),
                }));

                BatchOperationResult {
                    vm_id: vm_id.clone(),
                    vm_name,
                    success: true,
                    error: None,
                }
            }
            Err(e) => BatchOperationResult {
                vm_id: vm_id.clone(),
                vm_name,
                success: false,
                error: Some(e.to_string()),
            },
        };

        results.push(result);
    }

    Ok(results)
}

/// Mount guest agent ISO to a VM
/// Mount an ISO file to a VM's CD-ROM drive
#[tauri::command]
pub async fn mount_iso(
    state: State<'_, AppState>,
    vm_id: String,
    iso_path: String,
) -> Result<(), String> {
    tracing::info!("mount_iso command called for VM: {} with ISO: {}", vm_id, iso_path);

    VmService::mount_cd_iso(&state.libvirt, &vm_id, &iso_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mount_guest_agent_iso(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), String> {
    tracing::info!("mount_guest_agent_iso command called for VM: {}", vm_id);

    // First, try to add the KVM Manager agent channel if it doesn't exist
    // This is a best-effort attempt - the VM might need to be stopped first
    match VmService::attach_channel(&state.libvirt, &vm_id, "kvmmanager-agent") {
        Ok(_) => tracing::info!("Added KVM Manager agent channel to VM {}", vm_id),
        Err(e) => tracing::debug!("Could not add agent channel (may already exist): {}", e),
    }

    // Mount the ISO
    VmService::mount_cd_iso(&state.libvirt, &vm_id, "/var/lib/libvirt/images/kvmmanager-guest-agent.iso")
        .map_err(|e| e.to_string())
}

/// Unmount (eject) ISO from a VM
#[tauri::command]
pub async fn eject_cdrom(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), String> {
    tracing::info!("eject_cdrom command called for VM: {}", vm_id);

    VmService::eject_cd(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())
}

/// Update VM boot order
#[tauri::command]
pub async fn update_vm_boot_order(
    state: State<'_, AppState>,
    vm_id: String,
    boot_order: Vec<String>,
) -> Result<(), String> {
    tracing::info!("update_vm_boot_order command called for VM: {} with order: {:?}", vm_id, boot_order);

    VmService::update_boot_order(&state.libvirt, &vm_id, boot_order)
        .map_err(|e| e.to_string())
}

/// Rename a VM
#[tauri::command]
pub async fn rename_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    new_name: String,
) -> Result<(), String> {
    tracing::info!("rename_vm command called for VM: {} to new name: {}", vm_id, new_name);

    VmService::rename_vm(&state.libvirt, &vm_id, &new_name)
        .map_err(|e| e.to_string())?;

    // Emit event
    let _ = app.emit("vm-renamed", serde_json::json!({
        "vmId": vm_id,
        "newName": new_name,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(())
}

/// Get VM autostart status
#[tauri::command]
pub async fn get_vm_autostart(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<bool, String> {
    tracing::debug!("get_vm_autostart command called for VM: {}", vm_id);

    VmService::get_vm_autostart(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())
}

/// Set VM autostart status
#[tauri::command]
pub async fn set_vm_autostart(
    state: State<'_, AppState>,
    vm_id: String,
    enable: bool,
) -> Result<(), String> {
    tracing::info!("set_vm_autostart command called for VM: {} to {}", vm_id, enable);

    VmService::set_vm_autostart(&state.libvirt, &vm_id, enable)
        .map_err(|e| e.to_string())
}

/// Attach a network interface to a VM
#[tauri::command]
pub async fn attach_interface(
    state: State<'_, AppState>,
    vm_id: String,
    network: String,
    model: String,
    mac_address: Option<String>,
) -> Result<String, String> {
    tracing::info!("attach_interface command called for VM: {} on network: {}", vm_id, network);

    VmService::attach_interface(&state.libvirt, &vm_id, &network, &model, mac_address.as_deref())
        .map_err(|e| e.to_string())
}

/// Detach a network interface from a VM by MAC address
#[tauri::command]
pub async fn detach_interface(
    state: State<'_, AppState>,
    vm_id: String,
    mac_address: String,
) -> Result<(), String> {
    tracing::info!("detach_interface command called for VM: {} MAC: {}", vm_id, mac_address);

    VmService::detach_interface(&state.libvirt, &vm_id, &mac_address)
        .map_err(|e| e.to_string())
}

/// Attach a sound device to a VM
#[tauri::command]
pub async fn attach_sound(
    state: State<'_, AppState>,
    vm_id: String,
    model: String,
) -> Result<(), String> {
    tracing::info!("attach_sound command called for VM: {} model: {}", vm_id, model);

    VmService::attach_sound(&state.libvirt, &vm_id, &model)
        .map_err(|e| e.to_string())
}

/// Detach a sound device from a VM
#[tauri::command]
pub async fn detach_sound(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), String> {
    tracing::info!("detach_sound command called for VM: {}", vm_id);

    VmService::detach_sound(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())
}

/// Attach an input device to a VM
#[tauri::command]
pub async fn attach_input(
    state: State<'_, AppState>,
    vm_id: String,
    device_type: String,
    bus: String,
) -> Result<(), String> {
    tracing::info!("attach_input command called for VM: {} type: {} bus: {}", vm_id, device_type, bus);

    VmService::attach_input(&state.libvirt, &vm_id, &device_type, &bus)
        .map_err(|e| e.to_string())
}

/// Attach an RNG device to a VM
#[tauri::command]
pub async fn attach_rng(
    state: State<'_, AppState>,
    vm_id: String,
    backend: String,
) -> Result<(), String> {
    tracing::info!("attach_rng command called for VM: {} backend: {}", vm_id, backend);

    VmService::attach_rng(&state.libvirt, &vm_id, &backend)
        .map_err(|e| e.to_string())
}

/// Attach a watchdog device to a VM
#[tauri::command]
pub async fn attach_watchdog(
    state: State<'_, AppState>,
    vm_id: String,
    model: String,
    action: String,
) -> Result<(), String> {
    tracing::info!("attach_watchdog command called for VM: {} model: {} action: {}", vm_id, model, action);

    VmService::attach_watchdog(&state.libvirt, &vm_id, &model, &action)
        .map_err(|e| e.to_string())
}

/// Attach a channel device to a VM (QEMU Guest Agent or Spice)
#[tauri::command]
pub async fn attach_channel(
    state: State<'_, AppState>,
    vm_id: String,
    channel_type: String,
) -> Result<(), String> {
    tracing::info!("attach_channel command called for VM: {} type: {}", vm_id, channel_type);

    VmService::attach_channel(&state.libvirt, &vm_id, &channel_type)
        .map_err(|e| e.to_string())
}

/// Attach a filesystem share to a VM
#[tauri::command]
pub async fn attach_filesystem(
    state: State<'_, AppState>,
    vm_id: String,
    source_path: String,
    target_mount: String,
    fs_type: String,
    readonly: bool,
) -> Result<(), String> {
    tracing::info!("attach_filesystem command called for VM: {} path: {} -> {}",
        vm_id, source_path, target_mount);

    VmService::attach_filesystem(&state.libvirt, &vm_id, &source_path, &target_mount, &fs_type, readonly)
        .map_err(|e| e.to_string())
}

/// Attach a graphics device to a VM (VNC or Spice)
#[tauri::command]
pub async fn attach_graphics(
    state: State<'_, AppState>,
    vm_id: String,
    graphics_type: String,
    listen_address: Option<String>,
    port: Option<i32>,
) -> Result<(), String> {
    tracing::info!("attach_graphics command called for VM: {} type: {}", vm_id, graphics_type);

    VmService::attach_graphics(
        &state.libvirt,
        &vm_id,
        &graphics_type,
        listen_address.as_deref(),
        port,
    )
    .map_err(|e| e.to_string())
}

/// Attach a video device to a VM
#[tauri::command]
pub async fn attach_video(
    state: State<'_, AppState>,
    vm_id: String,
    model: String,
    vram: Option<u32>,
    heads: Option<u32>,
    acceleration_3d: Option<bool>,
) -> Result<(), String> {
    tracing::info!("attach_video command called for VM: {} model: {}", vm_id, model);

    VmService::attach_video(
        &state.libvirt,
        &vm_id,
        &model,
        vram,
        heads,
        acceleration_3d.unwrap_or(false),
    )
    .map_err(|e| e.to_string())
}

/// Set VM vCPU count
#[tauri::command]
pub async fn set_vm_vcpus(
    state: State<'_, AppState>,
    vm_id: String,
    vcpus: u32,
) -> Result<(), String> {
    tracing::info!("set_vm_vcpus command called for VM: {} vcpus: {}", vm_id, vcpus);

    VmService::set_vcpus(&state.libvirt, &vm_id, vcpus)
        .map_err(|e| e.to_string())
}

/// Set VM memory allocation
#[tauri::command]
pub async fn set_vm_memory(
    state: State<'_, AppState>,
    vm_id: String,
    memory_mb: u64,
) -> Result<(), String> {
    tracing::info!("set_vm_memory command called for VM: {} memory_mb: {}", vm_id, memory_mb);

    VmService::set_memory(&state.libvirt, &vm_id, memory_mb)
        .map_err(|e| e.to_string())
}

/// Set VM CPU topology (sockets, cores, threads)
#[tauri::command]
pub async fn set_vm_cpu_topology(
    state: State<'_, AppState>,
    vm_id: String,
    sockets: u32,
    cores: u32,
    threads: u32,
) -> Result<(), String> {
    tracing::info!("set_vm_cpu_topology command called for VM: {} topology: {}s/{}c/{}t",
        vm_id, sockets, cores, threads);

    VmService::set_cpu_topology(&state.libvirt, &vm_id, sockets, cores, threads)
        .map_err(|e| e.to_string())
}

/// Get CPU model configuration for a VM
#[tauri::command]
pub async fn get_cpu_model(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<CpuModelConfig, String> {
    tracing::info!("get_cpu_model command called for VM: {}", vm_id);

    VmService::get_cpu_model(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())
}

/// Set CPU model for a VM
#[tauri::command]
pub async fn set_cpu_model(
    state: State<'_, AppState>,
    vm_id: String,
    mode: String,
    model: Option<String>,
) -> Result<(), String> {
    tracing::info!("set_cpu_model command called for VM: {} mode: {} model: {:?}", vm_id, mode, model);

    VmService::set_cpu_model(&state.libvirt, &vm_id, &mode, model.as_deref())
        .map_err(|e| e.to_string())
}

/// Get available CPU models from libvirt
#[tauri::command]
pub async fn get_available_cpu_models(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    tracing::info!("get_available_cpu_models command called");

    VmService::get_available_cpu_models(&state.libvirt)
        .map_err(|e| e.to_string())
}

/// Get CPU pinning configuration for a VM
#[tauri::command]
pub async fn get_cpu_pinning(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<Vec<(u32, Vec<u32>)>, String> {
    tracing::info!("get_cpu_pinning command called for VM: {}", vm_id);

    VmService::get_cpu_pinning(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())
}

/// Set CPU pinning for a specific vCPU
#[tauri::command]
pub async fn set_cpu_pin(
    state: State<'_, AppState>,
    vm_id: String,
    vcpu: u32,
    host_cpus: Vec<u32>,
) -> Result<(), String> {
    tracing::info!("set_cpu_pin command called for VM: {} vCPU: {} -> CPUs: {:?}", vm_id, vcpu, host_cpus);

    VmService::set_cpu_pin(&state.libvirt, &vm_id, vcpu, host_cpus)
        .map_err(|e| e.to_string())
}

/// Clear CPU pinning for a specific vCPU
#[tauri::command]
pub async fn clear_cpu_pin(
    state: State<'_, AppState>,
    vm_id: String,
    vcpu: u32,
) -> Result<(), String> {
    tracing::info!("clear_cpu_pin command called for VM: {} vCPU: {}", vm_id, vcpu);

    VmService::clear_cpu_pin(&state.libvirt, &vm_id, vcpu)
        .map_err(|e| e.to_string())
}

/// Attach a serial port to a VM
#[tauri::command]
pub async fn attach_serial(
    state: State<'_, AppState>,
    vm_id: String,
    port_type: String,
    target_port: u32,
) -> Result<(), String> {
    tracing::info!("attach_serial command called for VM: {} type: {} port: {}", vm_id, port_type, target_port);

    VmService::attach_serial(&state.libvirt, &vm_id, &port_type, target_port)
        .map_err(|e| e.to_string())
}

/// Attach a console device to a VM
#[tauri::command]
pub async fn attach_console(
    state: State<'_, AppState>,
    vm_id: String,
    target_port: u32,
    target_type: String,
) -> Result<(), String> {
    tracing::info!("attach_console command called for VM: {} type: {} port: {}", vm_id, target_type, target_port);

    VmService::attach_console(&state.libvirt, &vm_id, target_port, &target_type)
        .map_err(|e| e.to_string())
}

/// Attach a TPM device to a VM
#[tauri::command]
pub async fn attach_tpm(
    state: State<'_, AppState>,
    vm_id: String,
    model: String,
    version: String,
) -> Result<(), String> {
    tracing::info!("attach_tpm command called for VM: {} model: {} version: {}", vm_id, model, version);

    VmService::attach_tpm(&state.libvirt, &vm_id, &model, &version)
        .map_err(|e| e.to_string())
}

/// Attach a USB controller to a VM
#[tauri::command]
pub async fn attach_usb_controller(
    state: State<'_, AppState>,
    vm_id: String,
    model: String,
) -> Result<(), String> {
    tracing::info!("attach_usb_controller command called for VM: {} model: {}", vm_id, model);

    VmService::attach_usb_controller(&state.libvirt, &vm_id, &model)
        .map_err(|e| e.to_string())
}

/// Get USB redirection configuration for a VM
#[tauri::command]
pub async fn get_usb_redirection(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<UsbRedirectionInfo, String> {
    tracing::info!("get_usb_redirection command called for VM: {}", vm_id);
    VmService::get_usb_redirection(&state.libvirt, &vm_id).map_err(|e| e.to_string())
}

/// Attach USB redirection channels to a VM for SPICE USB passthrough
#[tauri::command]
pub async fn attach_usb_redirection(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    count: u32,
) -> Result<(), String> {
    tracing::info!("attach_usb_redirection command called for VM: {} count: {}", vm_id, count);

    VmService::attach_usb_redirection(&state.libvirt, &vm_id, count).map_err(|e| e.to_string())?;

    let _ = app.emit("vm-usb-redirection-updated", serde_json::json!({
        "vmId": vm_id,
        "count": count,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(())
}

/// Remove all USB redirection channels from a VM
#[tauri::command]
pub async fn remove_usb_redirection(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), String> {
    tracing::info!("remove_usb_redirection command called for VM: {}", vm_id);

    VmService::remove_usb_redirection(&state.libvirt, &vm_id).map_err(|e| e.to_string())?;

    let _ = app.emit("vm-usb-redirection-updated", serde_json::json!({
        "vmId": vm_id,
        "count": 0,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(())
}

/// Attach a SCSI controller to a VM
#[tauri::command]
pub async fn attach_scsi_controller(
    state: State<'_, AppState>,
    vm_id: String,
    model: String,
) -> Result<(), String> {
    tracing::info!("attach_scsi_controller command called for VM: {} model: {}", vm_id, model);

    VmService::attach_scsi_controller(&state.libvirt, &vm_id, &model)
        .map_err(|e| e.to_string())
}

/// Attach a panic notifier device to a VM
/// Notifies the host when the guest kernel panics
#[tauri::command]
pub async fn attach_panic_notifier(
    state: State<'_, AppState>,
    vm_id: String,
    model: String,
) -> Result<(), String> {
    tracing::info!("attach_panic_notifier command called for VM: {} model: {}", vm_id, model);

    VmService::attach_panic_notifier(&state.libvirt, &vm_id, &model)
        .map_err(|e| e.to_string())
}

/// Attach a VirtIO VSOCK device to a VM
/// Allows fast guest-host communication
#[tauri::command]
pub async fn attach_vsock(
    state: State<'_, AppState>,
    vm_id: String,
    cid: u32,
) -> Result<(), String> {
    tracing::info!("attach_vsock command called for VM: {} CID: {}", vm_id, cid);

    VmService::attach_vsock(&state.libvirt, &vm_id, cid)
        .map_err(|e| e.to_string())
}

/// Attach a parallel port device to a VM
#[tauri::command]
pub async fn attach_parallel(
    state: State<'_, AppState>,
    vm_id: String,
    target_port: u32,
) -> Result<(), String> {
    tracing::info!("attach_parallel command called for VM: {} port: {}", vm_id, target_port);

    VmService::attach_parallel(&state.libvirt, &vm_id, target_port)
        .map_err(|e| e.to_string())
}

/// Attach a smartcard reader device to a VM
#[tauri::command]
pub async fn attach_smartcard(
    state: State<'_, AppState>,
    vm_id: String,
    mode: String,
) -> Result<(), String> {
    tracing::info!("attach_smartcard command called for VM: {} mode: {}", vm_id, mode);

    VmService::attach_smartcard(&state.libvirt, &vm_id, &mode)
        .map_err(|e| e.to_string())
}

/// Get host NUMA topology information
#[tauri::command]
pub async fn get_host_numa_topology(
    state: State<'_, AppState>,
) -> Result<Vec<crate::models::vm::HostNumaNode>, String> {
    tracing::info!("get_host_numa_topology command called");

    VmService::get_host_numa_topology(&state.libvirt)
        .map_err(|e| e.to_string())
}

/// Get VM NUMA configuration
#[tauri::command]
pub async fn get_vm_numa_config(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<Option<crate::models::vm::VmNumaConfig>, String> {
    tracing::info!("get_vm_numa_config command called for VM: {}", vm_id);

    VmService::get_vm_numa_config(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())
}

/// Set VM NUMA configuration
#[tauri::command]
pub async fn set_vm_numa_config(
    state: State<'_, AppState>,
    vm_id: String,
    config: crate::models::vm::VmNumaConfig,
) -> Result<(), String> {
    tracing::info!("set_vm_numa_config command called for VM: {}", vm_id);

    VmService::set_vm_numa_config(&state.libvirt, &vm_id, config)
        .map_err(|e| e.to_string())
}

/// Clear VM NUMA configuration
#[tauri::command]
pub async fn clear_vm_numa_config(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), String> {
    tracing::info!("clear_vm_numa_config command called for VM: {}", vm_id);

    VmService::clear_vm_numa_config(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())
}

/// Migrate a VM to another host
#[tauri::command]
pub async fn migrate_vm(
    state: State<'_, AppState>,
    vm_id: String,
    dest_uri: String,
    live: bool,
    unsafe_migration: bool,
) -> Result<(), String> {
    tracing::info!("migrate_vm command called for VM: {} to {}", vm_id, dest_uri);

    VmService::migrate_vm(&state.libvirt, &vm_id, &dest_uri, live, unsafe_migration)
        .map_err(|e| e.to_string())
}

/// Get migration info for a VM
#[tauri::command]
pub async fn get_migration_info(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<crate::services::vm_service::MigrationInfo, String> {
    tracing::info!("get_migration_info command called for VM: {}", vm_id);

    VmService::get_migration_info(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())
}

/// List available evdev input devices on the host
#[tauri::command]
pub async fn list_evdev_devices() -> Result<Vec<EvdevDevice>, String> {
    tracing::info!("list_evdev_devices command called");
    VmService::list_evdev_devices().map_err(|e| e.to_string())
}

/// Attach an evdev input device to a VM for low-latency passthrough
#[tauri::command]
pub async fn attach_evdev(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    device_path: String,
    grab_all: bool,
) -> Result<(), String> {
    tracing::info!("attach_evdev command called for VM: {} device: {}", vm_id, device_path);

    VmService::attach_evdev(&state.libvirt, &vm_id, &device_path, grab_all)
        .map_err(|e| e.to_string())?;

    let _ = app.emit("vm-evdev-updated", serde_json::json!({
        "vmId": vm_id,
        "devicePath": device_path,
        "action": "attached",
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(())
}

/// Get list of evdev devices attached to a VM
#[tauri::command]
pub async fn get_vm_evdev_devices(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<Vec<String>, String> {
    tracing::info!("get_vm_evdev_devices command called for VM: {}", vm_id);
    VmService::get_vm_evdev_devices(&state.libvirt, &vm_id).map_err(|e| e.to_string())
}

/// Remove an evdev device from a VM
#[tauri::command]
pub async fn detach_evdev(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    device_path: String,
) -> Result<(), String> {
    tracing::info!("detach_evdev command called for VM: {} device: {}", vm_id, device_path);

    VmService::detach_evdev(&state.libvirt, &vm_id, &device_path)
        .map_err(|e| e.to_string())?;

    let _ = app.emit("vm-evdev-updated", serde_json::json!({
        "vmId": vm_id,
        "devicePath": device_path,
        "action": "detached",
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));

    Ok(())
}
