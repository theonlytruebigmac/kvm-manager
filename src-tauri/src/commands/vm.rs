use crate::models::operation::{OperationKind, TargetIdentity};
use crate::models::vm::{VmState, VmStats, VM};
use crate::services::vm_service::{
    CpuModelConfig, EvdevDevice, HugepageInfo, HugepagesSettings, KernelBootSettings,
    UsbRedirectionInfo, VmService,
};
use crate::state::app_state::AppState;
use crate::utils::error::SafeFailure;
use crate::utils::xml::validate_document_root;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct VmStateChangedPayload {
    vm_id: String,
    vm_name: String,
    old_state: VmState,
    new_state: VmState,
    timestamp: i64,
}

fn vm_target(vm_id: &str) -> TargetIdentity {
    TargetIdentity {
        resource_kind: "vm".to_string(),
        stable_id: vm_id.to_string(),
        display_name: None,
    }
}

/// Get all VMs (active and inactive)
#[tauri::command]
pub async fn get_vms(state: State<'_, AppState>) -> Result<Vec<VM>, SafeFailure> {
    tracing::debug!("get_vms command called");

    let operation = state
        .resolve_operation(OperationKind::Query, None)
        .map_err(SafeFailure::from)?;
    VmService::list_vms(&operation.connection)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Get a single VM by ID
#[tauri::command]
pub async fn get_vm(state: State<'_, AppState>, vm_id: String) -> Result<VM, SafeFailure> {
    tracing::debug!("get_vm command called for VM: {}", vm_id);

    let operation = state
        .resolve_operation(OperationKind::Query, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::get_vm(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Start a VM
#[tauri::command]
pub async fn start_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), SafeFailure> {
    tracing::info!("start_vm command called for VM: {}", vm_id);

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    // Get VM info before state change from the same captured connection.
    let vm = VmService::get_vm(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;
    let old_state = vm.state.clone();
    let vm_name = vm.name.clone();

    VmService::start_vm(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-state-changed",
        VmStateChangedPayload {
            vm_id: vm_id.clone(),
            vm_name,
            old_state,
            new_state: VmState::Running,
            timestamp: chrono::Utc::now().timestamp_millis(),
        },
    );

    Ok(())
}

/// Stop a VM
#[tauri::command]
pub async fn stop_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), SafeFailure> {
    tracing::info!("stop_vm command called for VM: {}", vm_id);

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    // Get VM info before state change from the same captured connection.
    let vm = VmService::get_vm(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;
    let old_state = vm.state.clone();
    let vm_name = vm.name.clone();

    VmService::stop_vm(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-state-changed",
        VmStateChangedPayload {
            vm_id: vm_id.clone(),
            vm_name,
            old_state,
            new_state: VmState::Stopped,
            timestamp: chrono::Utc::now().timestamp_millis(),
        },
    );

    Ok(())
}

/// Force stop a VM (immediate power off)
#[tauri::command]
pub async fn force_stop_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!("force_stop_vm command called for VM: {}", vm_id);

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    // Get VM info before state change from the same captured connection.
    let vm = VmService::get_vm(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;
    let old_state = vm.state.clone();
    let vm_name = vm.name.clone();
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "force_stop_vm",
        "vm",
        &vm_id,
        None,
        "force-stop",
    )
    .map_err(SafeFailure::from)?;

    VmService::force_stop_vm(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-state-changed",
        VmStateChangedPayload {
            vm_id: vm_id.clone(),
            vm_name,
            old_state,
            new_state: VmState::Stopped,
            timestamp: chrono::Utc::now().timestamp_millis(),
        },
    );

    Ok(())
}

/// Pause a VM
#[tauri::command]
pub async fn pause_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), SafeFailure> {
    tracing::info!("pause_vm command called for VM: {}", vm_id);

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    // Get VM info before state change from the same captured connection.
    let vm = VmService::get_vm(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;
    let old_state = vm.state.clone();
    let vm_name = vm.name.clone();

    VmService::pause_vm(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-state-changed",
        VmStateChangedPayload {
            vm_id: vm_id.clone(),
            vm_name,
            old_state,
            new_state: VmState::Paused,
            timestamp: chrono::Utc::now().timestamp_millis(),
        },
    );

    Ok(())
}

/// Resume a paused VM
#[tauri::command]
pub async fn resume_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), SafeFailure> {
    tracing::info!("resume_vm command called for VM: {}", vm_id);

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    // Get VM info before state change from the same captured connection.
    let vm = VmService::get_vm(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;
    let old_state = vm.state.clone();
    let vm_name = vm.name.clone();

    VmService::resume_vm(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-state-changed",
        VmStateChangedPayload {
            vm_id: vm_id.clone(),
            vm_name,
            old_state,
            new_state: VmState::Running,
            timestamp: chrono::Utc::now().timestamp_millis(),
        },
    );

    Ok(())
}

/// Hibernate (managed save) a VM - saves state to disk and stops
#[tauri::command]
pub async fn hibernate_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), SafeFailure> {
    tracing::info!("hibernate_vm command called for VM: {}", vm_id);

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    // Get VM info before state change from the same captured connection.
    let vm = VmService::get_vm(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;
    let old_state = vm.state.clone();
    let vm_name = vm.name.clone();

    VmService::hibernate_vm(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event - VM is now stopped but with saved state
    let _ = app.emit(
        "vm-state-changed",
        VmStateChangedPayload {
            vm_id: vm_id.clone(),
            vm_name,
            old_state,
            new_state: VmState::Stopped,
            timestamp: chrono::Utc::now().timestamp_millis(),
        },
    );

    Ok(())
}

/// Check if VM has a managed save (hibernated state)
#[tauri::command]
pub async fn has_managed_save(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<bool, SafeFailure> {
    tracing::debug!("has_managed_save command called for VM: {}", vm_id);

    let operation = state
        .resolve_operation(OperationKind::Query, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::has_managed_save(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Remove managed save (discard hibernated state)
#[tauri::command]
pub async fn remove_managed_save(
    state: State<'_, AppState>,
    vm_id: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!("remove_managed_save command called for VM: {}", vm_id);

    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "remove_managed_save",
        "vm",
        &vm_id,
        None,
        "discard-managed-save",
    )
    .map_err(SafeFailure::from)?;
    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::remove_managed_save(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Reboot a VM
#[tauri::command]
pub async fn reboot_vm(state: State<'_, AppState>, vm_id: String) -> Result<(), SafeFailure> {
    tracing::info!("reboot_vm command called for VM: {}", vm_id);

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::reboot_vm(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Reset a running VM and inject a boot key while attached installation media is loading.
#[tauri::command]
pub async fn restart_to_install_media(
    state: State<'_, AppState>,
    vm_id: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!("restart_to_install_media command called for VM: {}", vm_id);

    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "restart_to_install_media",
        "vm",
        &vm_id,
        None,
        "immediate-reset-and-send-boot-key",
    )
    .map_err(SafeFailure::from)?;

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::restart_to_install_media(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Delete a VM
#[tauri::command]
pub async fn delete_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    delete_disks: bool,
    delete_snapshots: bool,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "delete_vm command called for VM: {} (delete_disks: {}, delete_snapshots: {})",
        vm_id,
        delete_disks,
        delete_snapshots
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    // Get VM info before deletion so the confirmation is tied to the immutable target label.
    let vm = VmService::get_vm(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;
    let vm_name = vm.name.clone();
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "delete_vm",
        "vm",
        &vm_id,
        None,
        &format!("delete-disks={delete_disks};delete-snapshots={delete_snapshots}"),
    )
    .map_err(SafeFailure::from)?;

    // Delete snapshots if requested
    if delete_snapshots {
        tracing::info!("Deleting all snapshots for VM: {}", vm_id);
        use crate::services::snapshot_service::SnapshotService;

        match SnapshotService::list_snapshots(&operation.connection, &vm_id) {
            Ok(snapshots) => {
                for snapshot in snapshots {
                    if SnapshotService::delete_snapshot(
                        &operation.connection,
                        &vm_id,
                        &snapshot.name,
                    )
                    .is_err()
                    {
                        tracing::warn!("A VM snapshot could not be deleted");
                    } else {
                        tracing::info!("Deleted snapshot: {}", snapshot.name);
                    }
                }
            }
            Err(_) => {
                tracing::warn!("VM snapshots could not be listed for deletion");
            }
        }
    }

    VmService::delete_vm(&operation.connection, &vm_id, delete_disks)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event (use a custom event for deletion)
    let _ = app.emit(
        "vm-deleted",
        serde_json::json!({
            "vmId": vm_id,
            "vmName": vm_name,
            "deleteDisks": delete_disks,
            "deleteSnapshots": delete_snapshots,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

    Ok(())
}

/// Clone an existing VM
#[tauri::command]
pub async fn clone_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    source_vm_id: String,
    new_name: String,
) -> Result<String, SafeFailure> {
    tracing::info!(
        "clone_vm command called: source_vm_id={}, new_name={}",
        source_vm_id,
        new_name
    );

    let operation = state
        .resolve_host_local_operation(Some(vm_target(&source_vm_id)))
        .map_err(SafeFailure::from)?;
    let cloned_vm_id = VmService::clone_vm(&operation.connection, &source_vm_id, &new_name)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-cloned",
        serde_json::json!({
            "sourceVmId": source_vm_id,
            "clonedVmId": cloned_vm_id,
            "newName": new_name,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

    Ok(cloned_vm_id)
}

/// Clone an existing VM with advanced options (disk cloning, snapshots)
#[tauri::command]
pub async fn clone_vm_with_options(
    app: AppHandle,
    state: State<'_, AppState>,
    source_vm_id: String,
    config: crate::models::vm::CloneConfig,
) -> Result<String, SafeFailure> {
    tracing::info!(
        "clone_vm_with_options command called: source_vm_id={}, config={:?}",
        source_vm_id,
        config
    );

    let operation = state
        .resolve_host_local_operation(Some(vm_target(&source_vm_id)))
        .map_err(SafeFailure::from)?;
    let cloned_vm_id =
        VmService::clone_vm_with_options(&operation.connection, &source_vm_id, &config)
            .map_err(SafeFailure::from)
            .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-cloned",
        serde_json::json!({
            "sourceVmId": source_vm_id,
            "clonedVmId": cloned_vm_id,
            "newName": config.new_name,
            "cloneDisks": config.clone_disks,
            "cloneSnapshots": config.clone_snapshots,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

    Ok(cloned_vm_id)
}

/// Create a new VM
#[tauri::command]
pub async fn create_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    config: crate::models::vm::VmConfig,
) -> Result<String, SafeFailure> {
    tracing::info!("create_vm command called for VM: {}", config.name);

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&config.name)))
        .map_err(SafeFailure::from)?;
    let vm_id = VmService::create_vm(&operation.connection, &operation.context, config)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-created",
        serde_json::json!({
            "vmId": vm_id,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

    Ok(vm_id)
}

/// Get VM performance statistics
#[tauri::command]
pub async fn get_vm_stats(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<VmStats, SafeFailure> {
    tracing::debug!("get_vm_stats command called for VM: {}", vm_id);

    let operation = state
        .resolve_operation(OperationKind::Query, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::get_vm_stats(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Add tags to a VM
#[tauri::command]
pub async fn add_vm_tags(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    tags: Vec<String>,
) -> Result<(), SafeFailure> {
    tracing::info!("add_vm_tags command called for VM: {}", vm_id);

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::add_vm_tags(&operation.connection, &vm_id, tags.clone())
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-tags-updated",
        serde_json::json!({
            "vmId": vm_id,
            "tags": tags,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

    Ok(())
}

/// Remove tags from a VM
#[tauri::command]
pub async fn remove_vm_tags(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    tags: Vec<String>,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!("remove_vm_tags command called for VM: {}", vm_id);

    let mut reviewed_tags = tags.clone();
    reviewed_tags.sort();
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "remove_vm_tags",
        "vm",
        &vm_id,
        None,
        &format!("remove-tags={}", reviewed_tags.join(",")),
    )
    .map_err(SafeFailure::from)?;

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::remove_vm_tags(&operation.connection, &vm_id, tags.clone())
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-tags-updated",
        serde_json::json!({
            "vmId": vm_id,
            "removedTags": tags,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

    Ok(())
}

/// Export VM configuration to XML
#[tauri::command]
pub async fn export_vm(state: State<'_, AppState>, vm_id: String) -> Result<String, SafeFailure> {
    tracing::info!("export_vm command called for VM: {}", vm_id);

    let operation = state
        .resolve_operation(OperationKind::Query, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::export_vm(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Import VM from XML configuration
#[tauri::command]
pub async fn import_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    xml: String,
) -> Result<String, SafeFailure> {
    tracing::info!("import_vm command called");

    validate_document_root(&xml, "domain").map_err(SafeFailure::from)?;

    let operation = state
        .resolve_operation(OperationKind::Mutation, None)
        .map_err(SafeFailure::from)?;
    let vm_id = VmService::import_vm(&operation.connection, &xml)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-imported",
        serde_json::json!({
            "vmId": vm_id,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

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
) -> Result<(), SafeFailure> {
    tracing::info!("attach_disk command called for VM: {}", vm_id);

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::attach_disk(
        &operation.connection,
        &vm_id,
        &disk_path,
        &device_target,
        &bus_type,
    )
    .map_err(SafeFailure::from)
    .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-disk-attached",
        serde_json::json!({
            "vmId": vm_id,
            "diskPath": disk_path,
            "deviceTarget": device_target,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

    Ok(())
}

/// Detach a disk from a VM
#[tauri::command]
pub async fn detach_disk(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    device_target: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!("detach_disk command called for VM: {}", vm_id);

    let stable_id = format!("{vm_id}/{device_target}");
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "detach_disk",
        "disk",
        &stable_id,
        None,
        "detach",
    )
    .map_err(SafeFailure::from)?;
    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::detach_disk(&operation.connection, &vm_id, &device_target)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-disk-detached",
        serde_json::json!({
            "vmId": vm_id,
            "deviceTarget": device_target,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

    Ok(())
}

/// Update disk I/O settings (cache, io mode, discard, throttling)
#[tauri::command]
#[allow(clippy::too_many_arguments)] // Tauri IPC preserves the existing field-level client contract.
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
) -> Result<(), SafeFailure> {
    tracing::info!(
        "update_disk_settings command called for VM: {}, disk: {}",
        vm_id,
        device_target
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::update_disk_settings(
        &operation.connection,
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
    .map_err(SafeFailure::from)
    .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-disk-settings-updated",
        serde_json::json!({
            "vmId": vm_id,
            "deviceTarget": device_target,
            "cache": cache,
            "io": io,
            "discard": discard,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

    Ok(())
}

/// Get direct kernel boot settings for a VM
#[tauri::command]
pub async fn get_kernel_boot_settings(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<KernelBootSettings, SafeFailure> {
    tracing::info!("get_kernel_boot_settings command called for VM: {}", vm_id);
    let operation = state
        .resolve_operation(OperationKind::Query, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::get_kernel_boot_settings(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Update direct kernel boot settings for a VM (VM must be shut off)
#[tauri::command]
#[allow(clippy::too_many_arguments)] // Tauri IPC preserves the existing field-level client contract.
pub async fn set_kernel_boot_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    enabled: bool,
    kernel_path: Option<String>,
    initrd_path: Option<String>,
    kernel_args: Option<String>,
    dtb_path: Option<String>,
) -> Result<(), SafeFailure> {
    tracing::info!("set_kernel_boot_settings command called for VM: {}", vm_id);

    let settings = KernelBootSettings {
        enabled,
        kernel_path: kernel_path.clone(),
        initrd_path: initrd_path.clone(),
        kernel_args: kernel_args.clone(),
        dtb_path: dtb_path.clone(),
    };

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::set_kernel_boot_settings(&operation.connection, &vm_id, settings)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-kernel-boot-updated",
        serde_json::json!({
            "vmId": vm_id,
            "enabled": enabled,
            "kernelPath": kernel_path,
            "initrdPath": initrd_path,
            "kernelArgs": kernel_args,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

    Ok(())
}

/// Get hugepages memory backing settings for a VM
#[tauri::command]
pub async fn get_hugepages_settings(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<HugepagesSettings, SafeFailure> {
    tracing::info!("get_hugepages_settings command called for VM: {}", vm_id);
    let operation = state
        .resolve_operation(OperationKind::Query, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::get_hugepages_settings(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Set hugepages memory backing for a VM (VM must be shut off)
#[tauri::command]
pub async fn set_hugepages(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    enabled: bool,
    size: Option<u64>, // Size in KiB
) -> Result<(), SafeFailure> {
    tracing::info!(
        "set_hugepages command called for VM: {} enabled={} size={:?}",
        vm_id,
        enabled,
        size
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::set_hugepages(&operation.connection, &vm_id, enabled, size)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-hugepages-updated",
        serde_json::json!({
            "vmId": vm_id,
            "enabled": enabled,
            "size": size,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

    Ok(())
}

/// Get available hugepage sizes on the host system
#[tauri::command]
pub async fn get_host_hugepage_info() -> Result<Vec<HugepageInfo>, SafeFailure> {
    tracing::info!("get_host_hugepage_info command called");
    VmService::get_host_hugepage_info().map_err(SafeFailure::from)
}

/// Update network interface bandwidth/QoS settings
#[tauri::command]
#[allow(clippy::too_many_arguments)] // Tauri IPC preserves the existing field-level client contract.
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
) -> Result<(), SafeFailure> {
    tracing::info!(
        "update_interface_bandwidth command called for VM: {}, interface: {}",
        vm_id,
        mac_address
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::update_interface_bandwidth(
        &operation.connection,
        &vm_id,
        &mac_address,
        inbound_average,
        inbound_peak,
        inbound_burst,
        outbound_average,
        outbound_peak,
        outbound_burst,
    )
    .map_err(SafeFailure::from)
    .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-interface-bandwidth-updated",
        serde_json::json!({
            "vmId": vm_id,
            "macAddress": mac_address,
            "inboundAverage": inbound_average,
            "outboundAverage": outbound_average,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

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
) -> Result<(), SafeFailure> {
    tracing::info!(
        "set_interface_link_state command called for VM: {}, interface: {}, up: {}",
        vm_id,
        mac_address,
        link_up
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::set_interface_link_state(&operation.connection, &vm_id, &mac_address, link_up)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-interface-link-state-changed",
        serde_json::json!({
            "vmId": vm_id,
            "macAddress": mac_address,
            "linkUp": link_up,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

    Ok(())
}

/// Get the link state for a network interface
#[tauri::command]
pub async fn get_interface_link_state(
    state: State<'_, AppState>,
    vm_id: String,
    mac_address: String,
) -> Result<bool, SafeFailure> {
    tracing::info!(
        "get_interface_link_state command called for VM: {}, interface: {}",
        vm_id,
        mac_address
    );

    let operation = state
        .resolve_operation(OperationKind::Query, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::get_interface_link_state(&operation.connection, &vm_id, &mac_address)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchOperationResult {
    vm_id: String,
    vm_name: String,
    success: bool,
    error: Option<SafeFailure>,
}

/// Start multiple VMs in batch
#[tauri::command]
pub async fn batch_start_vms(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_ids: Vec<String>,
) -> Result<Vec<BatchOperationResult>, SafeFailure> {
    tracing::info!("batch_start_vms command called for {} VMs", vm_ids.len());

    let operation = state
        .resolve_operation(OperationKind::Mutation, None)
        .map_err(SafeFailure::from)?;
    let mut results = Vec::new();

    for vm_id in vm_ids {
        let vm = match VmService::get_vm(&operation.connection, &vm_id) {
            Ok(v) => v,
            Err(e) => {
                results.push(BatchOperationResult {
                    vm_id: vm_id.clone(),
                    vm_name: "Unknown".to_string(),
                    success: false,
                    error: Some(SafeFailure::from(e).with_context(&operation.context)),
                });
                continue;
            }
        };

        let old_state = vm.state.clone();
        let vm_name = vm.name.clone();

        let result = match VmService::start_vm(&operation.connection, &vm_id) {
            Ok(_) => {
                let _ = app.emit(
                    "vm-state-changed",
                    VmStateChangedPayload {
                        vm_id: vm_id.clone(),
                        vm_name: vm_name.clone(),
                        old_state,
                        new_state: VmState::Running,
                        timestamp: chrono::Utc::now().timestamp_millis(),
                    },
                );

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
                error: Some(SafeFailure::from(e).with_context(&operation.context)),
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
) -> Result<Vec<BatchOperationResult>, SafeFailure> {
    tracing::info!(
        "batch_stop_vms command called for {} VMs (force: {})",
        vm_ids.len(),
        force
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, None)
        .map_err(SafeFailure::from)?;
    let mut results = Vec::new();

    for vm_id in vm_ids {
        let vm = match VmService::get_vm(&operation.connection, &vm_id) {
            Ok(v) => v,
            Err(e) => {
                results.push(BatchOperationResult {
                    vm_id: vm_id.clone(),
                    vm_name: "Unknown".to_string(),
                    success: false,
                    error: Some(SafeFailure::from(e).with_context(&operation.context)),
                });
                continue;
            }
        };

        let old_state = vm.state.clone();
        let vm_name = vm.name.clone();

        let result = if force {
            match VmService::force_stop_vm(&operation.connection, &vm_id) {
                Ok(_) => {
                    let _ = app.emit(
                        "vm-state-changed",
                        VmStateChangedPayload {
                            vm_id: vm_id.clone(),
                            vm_name: vm_name.clone(),
                            old_state,
                            new_state: VmState::Stopped,
                            timestamp: chrono::Utc::now().timestamp_millis(),
                        },
                    );

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
                    error: Some(SafeFailure::from(e).with_context(&operation.context)),
                },
            }
        } else {
            match VmService::stop_vm(&operation.connection, &vm_id) {
                Ok(_) => {
                    let _ = app.emit(
                        "vm-state-changed",
                        VmStateChangedPayload {
                            vm_id: vm_id.clone(),
                            vm_name: vm_name.clone(),
                            old_state,
                            new_state: VmState::Stopped,
                            timestamp: chrono::Utc::now().timestamp_millis(),
                        },
                    );

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
                    error: Some(SafeFailure::from(e).with_context(&operation.context)),
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
) -> Result<Vec<BatchOperationResult>, SafeFailure> {
    tracing::info!("batch_reboot_vms command called for {} VMs", vm_ids.len());

    let operation = state
        .resolve_operation(OperationKind::Mutation, None)
        .map_err(SafeFailure::from)?;
    let mut results = Vec::new();

    for vm_id in vm_ids {
        let vm = match VmService::get_vm(&operation.connection, &vm_id) {
            Ok(v) => v,
            Err(e) => {
                results.push(BatchOperationResult {
                    vm_id: vm_id.clone(),
                    vm_name: "Unknown".to_string(),
                    success: false,
                    error: Some(SafeFailure::from(e).with_context(&operation.context)),
                });
                continue;
            }
        };

        let vm_name = vm.name.clone();

        let result = match VmService::reboot_vm(&operation.connection, &vm_id) {
            Ok(_) => {
                let _ = app.emit(
                    "vm-rebooted",
                    serde_json::json!({
                        "vmId": vm_id.clone(),
                        "vmName": vm_name.clone(),
                        "timestamp": chrono::Utc::now().timestamp_millis(),
                    }),
                );

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
                error: Some(SafeFailure::from(e).with_context(&operation.context)),
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
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!("mount_iso command called for VM: {}", vm_id);

    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "mount_iso",
        "vm",
        &vm_id,
        None,
        "mount-iso",
    )
    .map_err(SafeFailure::from)?;

    let operation = state
        .resolve_host_local_operation(Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::mount_cd_iso(&operation.connection, &vm_id, &iso_path)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

#[tauri::command]
pub async fn mount_guest_agent_iso(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), SafeFailure> {
    tracing::info!("mount_guest_agent_iso command called for VM: {}", vm_id);

    let operation = state
        .resolve_host_local_operation(Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    // First, try to add the KVM Manager agent channel if it doesn't exist
    // This is a best-effort attempt - the VM might need to be stopped first
    match VmService::attach_channel(&operation.connection, &vm_id, "kvmmanager-agent") {
        Ok(_) => tracing::info!("Added KVM Manager agent channel to VM {}", vm_id),
        Err(_) => tracing::debug!("VM agent channel was not added"),
    }

    // Mount the ISO
    VmService::mount_cd_iso(
        &operation.connection,
        &vm_id,
        "/var/lib/libvirt/images/kvmmanager-guest-agent.iso",
    )
    .map_err(SafeFailure::from)
    .map_err(|failure| failure.with_context(&operation.context))
}

/// Unmount (eject) ISO from a VM
#[tauri::command]
pub async fn eject_cdrom(
    state: State<'_, AppState>,
    vm_id: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!("eject_cdrom command called for VM: {}", vm_id);

    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "eject_cdrom",
        "vm",
        &vm_id,
        None,
        "eject-cdrom",
    )
    .map_err(SafeFailure::from)?;
    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::eject_cd(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Update VM boot order
#[tauri::command]
pub async fn update_vm_boot_order(
    state: State<'_, AppState>,
    vm_id: String,
    boot_order: Vec<String>,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "update_vm_boot_order command called for VM: {} with order: {:?}",
        vm_id,
        boot_order
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::update_boot_order(&operation.connection, &vm_id, boot_order)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Rename a VM
#[tauri::command]
pub async fn rename_vm(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    new_name: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "rename_vm command called for VM: {} to new name: {}",
        vm_id,
        new_name
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::rename_vm(&operation.connection, &vm_id, &new_name)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event
    let _ = app.emit(
        "vm-renamed",
        serde_json::json!({
            "vmId": vm_id,
            "newName": new_name,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

    Ok(())
}

/// Get VM autostart status
#[tauri::command]
pub async fn get_vm_autostart(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<bool, SafeFailure> {
    tracing::debug!("get_vm_autostart command called for VM: {}", vm_id);

    let operation = state
        .resolve_operation(OperationKind::Query, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::get_vm_autostart(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Set VM autostart status
#[tauri::command]
pub async fn set_vm_autostart(
    state: State<'_, AppState>,
    vm_id: String,
    enable: bool,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "set_vm_autostart command called for VM: {} to {}",
        vm_id,
        enable
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::set_vm_autostart(&operation.connection, &vm_id, enable)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Attach a network interface to a VM
#[tauri::command]
pub async fn attach_interface(
    state: State<'_, AppState>,
    vm_id: String,
    network: String,
    model: String,
    mac_address: Option<String>,
) -> Result<String, SafeFailure> {
    tracing::info!(
        "attach_interface command called for VM: {} on network: {}",
        vm_id,
        network
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::attach_interface(
        &operation.connection,
        &vm_id,
        &network,
        &model,
        mac_address.as_deref(),
    )
    .map_err(SafeFailure::from)
    .map_err(|failure| failure.with_context(&operation.context))
}

/// Attach a network interface with advanced options (macvtap, bridge, OVS)
#[tauri::command]
#[allow(clippy::too_many_arguments)] // Tauri IPC preserves the existing field-level client contract.
pub async fn attach_interface_advanced(
    state: State<'_, AppState>,
    vm_id: String,
    interface_type: String,
    source: String,
    model: String,
    mac_address: Option<String>,
    source_mode: Option<String>,
    vlan_id: Option<u16>,
    portgroup: Option<String>,
    mtu: Option<u32>,
) -> Result<String, SafeFailure> {
    tracing::info!(
        "attach_interface_advanced command called for VM: {} type: {} source: {}",
        vm_id,
        interface_type,
        source
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::attach_interface_advanced(
        &operation.connection,
        &vm_id,
        &interface_type,
        &source,
        &model,
        mac_address.as_deref(),
        source_mode.as_deref(),
        vlan_id,
        portgroup.as_deref(),
        mtu,
    )
    .map_err(SafeFailure::from)
    .map_err(|failure| failure.with_context(&operation.context))
}

/// List host network interfaces available for macvtap/direct attachment
#[tauri::command]
pub async fn list_host_interfaces(
) -> Result<Vec<crate::models::vm::HostNetworkInterface>, SafeFailure> {
    tracing::debug!("list_host_interfaces command called");
    VmService::list_host_interfaces().map_err(SafeFailure::from)
}

/// Detach a network interface from a VM by MAC address
#[tauri::command]
pub async fn detach_interface(
    state: State<'_, AppState>,
    vm_id: String,
    mac_address: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "detach_interface command called for VM: {} MAC: {}",
        vm_id,
        mac_address
    );

    let stable_id = format!("{vm_id}/{mac_address}");
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "detach_interface",
        "interface",
        &stable_id,
        None,
        "detach",
    )
    .map_err(SafeFailure::from)?;
    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::detach_interface(&operation.connection, &vm_id, &mac_address)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Attach a sound device to a VM
#[tauri::command]
pub async fn attach_sound(
    state: State<'_, AppState>,
    vm_id: String,
    model: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "attach_sound command called for VM: {} model: {}",
        vm_id,
        model
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::attach_sound(&operation.connection, &vm_id, &model)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Detach a sound device from a VM
#[tauri::command]
pub async fn detach_sound(
    state: State<'_, AppState>,
    vm_id: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!("detach_sound command called for VM: {}", vm_id);

    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "detach_sound",
        "sound_device",
        &vm_id,
        None,
        "detach",
    )
    .map_err(SafeFailure::from)?;
    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::detach_sound(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Attach an input device to a VM
#[tauri::command]
pub async fn attach_input(
    state: State<'_, AppState>,
    vm_id: String,
    device_type: String,
    bus: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "attach_input command called for VM: {} type: {} bus: {}",
        vm_id,
        device_type,
        bus
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::attach_input(&operation.connection, &vm_id, &device_type, &bus)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Attach an RNG device to a VM
#[tauri::command]
pub async fn attach_rng(
    state: State<'_, AppState>,
    vm_id: String,
    backend: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "attach_rng command called for VM: {} backend: {}",
        vm_id,
        backend
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::attach_rng(&operation.connection, &vm_id, &backend)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Attach a watchdog device to a VM
#[tauri::command]
pub async fn attach_watchdog(
    state: State<'_, AppState>,
    vm_id: String,
    model: String,
    action: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "attach_watchdog command called for VM: {} model: {} action: {}",
        vm_id,
        model,
        action
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::attach_watchdog(&operation.connection, &vm_id, &model, &action)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Attach a channel device to a VM (QEMU Guest Agent or Spice)
#[tauri::command]
pub async fn attach_channel(
    state: State<'_, AppState>,
    vm_id: String,
    channel_type: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "attach_channel command called for VM: {} type: {}",
        vm_id,
        channel_type
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::attach_channel(&operation.connection, &vm_id, &channel_type)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
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
) -> Result<(), SafeFailure> {
    tracing::info!("attach_filesystem command called for VM: {}", vm_id);

    let operation = state
        .resolve_host_local_operation(Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::attach_filesystem(
        &operation.connection,
        &vm_id,
        &source_path,
        &target_mount,
        &fs_type,
        readonly,
    )
    .map_err(SafeFailure::from)
    .map_err(|failure| failure.with_context(&operation.context))
}

/// Attach a graphics device to a VM (VNC or Spice)
#[tauri::command]
pub async fn attach_graphics(
    state: State<'_, AppState>,
    vm_id: String,
    graphics_type: String,
    listen_address: Option<String>,
    port: Option<i32>,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "attach_graphics command called for VM: {} type: {}",
        vm_id,
        graphics_type
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::attach_graphics(
        &operation.connection,
        &vm_id,
        &graphics_type,
        listen_address.as_deref(),
        port,
    )
    .map_err(SafeFailure::from)
    .map_err(|failure| failure.with_context(&operation.context))
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
) -> Result<(), SafeFailure> {
    tracing::info!(
        "attach_video command called for VM: {} model: {}",
        vm_id,
        model
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::attach_video(
        &operation.connection,
        &vm_id,
        &model,
        vram,
        heads,
        acceleration_3d.unwrap_or(false),
    )
    .map_err(SafeFailure::from)
    .map_err(|failure| failure.with_context(&operation.context))
}

/// Set VM vCPU count
#[tauri::command]
pub async fn set_vm_vcpus(
    state: State<'_, AppState>,
    vm_id: String,
    vcpus: u32,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "set_vm_vcpus command called for VM: {} vcpus: {}",
        vm_id,
        vcpus
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::set_vcpus(&operation.connection, &vm_id, vcpus)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Set VM memory allocation
#[tauri::command]
pub async fn set_vm_memory(
    state: State<'_, AppState>,
    vm_id: String,
    memory_mb: u64,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "set_vm_memory command called for VM: {} memory_mb: {}",
        vm_id,
        memory_mb
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::set_memory(&operation.connection, &vm_id, memory_mb)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Set VM CPU topology (sockets, cores, threads)
#[tauri::command]
pub async fn set_vm_cpu_topology(
    state: State<'_, AppState>,
    vm_id: String,
    sockets: u32,
    cores: u32,
    threads: u32,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "set_vm_cpu_topology command called for VM: {} topology: {}s/{}c/{}t",
        vm_id,
        sockets,
        cores,
        threads
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::set_cpu_topology(&operation.connection, &vm_id, sockets, cores, threads)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Get CPU model configuration for a VM
#[tauri::command]
pub async fn get_cpu_model(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<CpuModelConfig, SafeFailure> {
    tracing::info!("get_cpu_model command called for VM: {}", vm_id);

    let operation = state
        .resolve_operation(OperationKind::Query, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::get_cpu_model(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Set CPU model for a VM
#[tauri::command]
pub async fn set_cpu_model(
    state: State<'_, AppState>,
    vm_id: String,
    mode: String,
    model: Option<String>,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "set_cpu_model command called for VM: {} mode: {} model: {:?}",
        vm_id,
        mode,
        model
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::set_cpu_model(&operation.connection, &vm_id, &mode, model.as_deref())
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Get available CPU models from libvirt
#[tauri::command]
pub async fn get_available_cpu_models(
    state: State<'_, AppState>,
) -> Result<Vec<String>, SafeFailure> {
    tracing::info!("get_available_cpu_models command called");

    let operation = state
        .resolve_host_local_operation(None)
        .map_err(SafeFailure::from)?;
    VmService::get_available_cpu_models(&operation.connection).map_err(SafeFailure::from)
}

/// Get CPU pinning configuration for a VM
#[tauri::command]
pub async fn get_cpu_pinning(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<Vec<(u32, Vec<u32>)>, SafeFailure> {
    tracing::info!("get_cpu_pinning command called for VM: {}", vm_id);

    let operation = state
        .resolve_operation(OperationKind::Query, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::get_cpu_pinning(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Set CPU pinning for a specific vCPU
#[tauri::command]
pub async fn set_cpu_pin(
    state: State<'_, AppState>,
    vm_id: String,
    vcpu: u32,
    host_cpus: Vec<u32>,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "set_cpu_pin command called for VM: {} vCPU: {} -> CPUs: {:?}",
        vm_id,
        vcpu,
        host_cpus
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::set_cpu_pin(&operation.connection, &vm_id, vcpu, host_cpus)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Clear CPU pinning for a specific vCPU
#[tauri::command]
pub async fn clear_cpu_pin(
    state: State<'_, AppState>,
    vm_id: String,
    vcpu: u32,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "clear_cpu_pin command called for VM: {} vCPU: {}",
        vm_id,
        vcpu
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::clear_cpu_pin(&operation.connection, &vm_id, vcpu)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Attach a serial port to a VM
#[tauri::command]
pub async fn attach_serial(
    state: State<'_, AppState>,
    vm_id: String,
    port_type: String,
    target_port: u32,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "attach_serial command called for VM: {} type: {} port: {}",
        vm_id,
        port_type,
        target_port
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::attach_serial(&operation.connection, &vm_id, &port_type, target_port)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Attach a console device to a VM
#[tauri::command]
pub async fn attach_console(
    state: State<'_, AppState>,
    vm_id: String,
    target_port: u32,
    target_type: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "attach_console command called for VM: {} type: {} port: {}",
        vm_id,
        target_type,
        target_port
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::attach_console(&operation.connection, &vm_id, target_port, &target_type)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Attach a TPM device to a VM
#[tauri::command]
pub async fn attach_tpm(
    state: State<'_, AppState>,
    vm_id: String,
    model: String,
    version: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "attach_tpm command called for VM: {} model: {} version: {}",
        vm_id,
        model,
        version
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::attach_tpm(&operation.connection, &vm_id, &model, &version)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Attach a USB controller to a VM
#[tauri::command]
pub async fn attach_usb_controller(
    state: State<'_, AppState>,
    vm_id: String,
    model: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "attach_usb_controller command called for VM: {} model: {}",
        vm_id,
        model
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::attach_usb_controller(&operation.connection, &vm_id, &model)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Get USB redirection configuration for a VM
#[tauri::command]
pub async fn get_usb_redirection(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<UsbRedirectionInfo, SafeFailure> {
    tracing::info!("get_usb_redirection command called for VM: {}", vm_id);
    let operation = state
        .resolve_operation(OperationKind::Query, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::get_usb_redirection(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Attach USB redirection channels to a VM for SPICE USB passthrough
#[tauri::command]
pub async fn attach_usb_redirection(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    count: u32,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "attach_usb_redirection command called for VM: {} count: {}",
        vm_id,
        count
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::attach_usb_redirection(&operation.connection, &vm_id, count)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    let _ = app.emit(
        "vm-usb-redirection-updated",
        serde_json::json!({
            "vmId": vm_id,
            "count": count,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

    Ok(())
}

/// Remove all USB redirection channels from a VM
#[tauri::command]
pub async fn remove_usb_redirection(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!("remove_usb_redirection command called for VM: {}", vm_id);

    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "remove_usb_redirection",
        "vm",
        &vm_id,
        None,
        "remove-usb-redirection",
    )
    .map_err(SafeFailure::from)?;
    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::remove_usb_redirection(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    let _ = app.emit(
        "vm-usb-redirection-updated",
        serde_json::json!({
            "vmId": vm_id,
            "count": 0,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

    Ok(())
}

/// Attach a SCSI controller to a VM
#[tauri::command]
pub async fn attach_scsi_controller(
    state: State<'_, AppState>,
    vm_id: String,
    model: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "attach_scsi_controller command called for VM: {} model: {}",
        vm_id,
        model
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::attach_scsi_controller(&operation.connection, &vm_id, &model)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Attach a panic notifier device to a VM
/// Notifies the host when the guest kernel panics
#[tauri::command]
pub async fn attach_panic_notifier(
    state: State<'_, AppState>,
    vm_id: String,
    model: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "attach_panic_notifier command called for VM: {} model: {}",
        vm_id,
        model
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::attach_panic_notifier(&operation.connection, &vm_id, &model)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Attach a VirtIO VSOCK device to a VM
/// Allows fast guest-host communication
#[tauri::command]
pub async fn attach_vsock(
    state: State<'_, AppState>,
    vm_id: String,
    cid: u32,
) -> Result<(), SafeFailure> {
    tracing::info!("attach_vsock command called for VM: {} CID: {}", vm_id, cid);

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::attach_vsock(&operation.connection, &vm_id, cid)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Attach a parallel port device to a VM
#[tauri::command]
pub async fn attach_parallel(
    state: State<'_, AppState>,
    vm_id: String,
    target_port: u32,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "attach_parallel command called for VM: {} port: {}",
        vm_id,
        target_port
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::attach_parallel(&operation.connection, &vm_id, target_port)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Attach a smartcard reader device to a VM
#[tauri::command]
pub async fn attach_smartcard(
    state: State<'_, AppState>,
    vm_id: String,
    mode: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "attach_smartcard command called for VM: {} mode: {}",
        vm_id,
        mode
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::attach_smartcard(&operation.connection, &vm_id, &mode)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Get host NUMA topology information
#[tauri::command]
pub async fn get_host_numa_topology(
    state: State<'_, AppState>,
) -> Result<Vec<crate::models::vm::HostNumaNode>, SafeFailure> {
    tracing::info!("get_host_numa_topology command called");

    let operation = state
        .resolve_host_local_operation(None)
        .map_err(SafeFailure::from)?;
    VmService::get_host_numa_topology(&operation.connection).map_err(SafeFailure::from)
}

/// Get VM NUMA configuration
#[tauri::command]
pub async fn get_vm_numa_config(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<Option<crate::models::vm::VmNumaConfig>, SafeFailure> {
    tracing::info!("get_vm_numa_config command called for VM: {}", vm_id);

    let operation = state
        .resolve_operation(OperationKind::Query, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::get_vm_numa_config(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Set VM NUMA configuration
#[tauri::command]
pub async fn set_vm_numa_config(
    state: State<'_, AppState>,
    vm_id: String,
    config: crate::models::vm::VmNumaConfig,
) -> Result<(), SafeFailure> {
    tracing::info!("set_vm_numa_config command called for VM: {}", vm_id);

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::set_vm_numa_config(&operation.connection, &vm_id, config)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Clear VM NUMA configuration
#[tauri::command]
pub async fn clear_vm_numa_config(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), SafeFailure> {
    tracing::info!("clear_vm_numa_config command called for VM: {}", vm_id);

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::clear_vm_numa_config(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Migrate a VM to another host
#[tauri::command]
pub async fn migrate_vm(
    state: State<'_, AppState>,
    vm_id: String,
    dest_uri: String,
    live: bool,
    unsafe_migration: bool,
) -> Result<(), SafeFailure> {
    tracing::info!("migrate_vm command called");

    let operation = state
        .resolve_operation(OperationKind::Migration, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::migrate_vm(
        &operation.connection,
        &vm_id,
        &dest_uri,
        live,
        unsafe_migration,
    )
    .map_err(SafeFailure::from)
    .map_err(|failure| failure.with_context(&operation.context))
}

/// Get migration info for a VM
#[tauri::command]
pub async fn get_migration_info(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<crate::services::vm_service::MigrationInfo, SafeFailure> {
    tracing::info!("get_migration_info command called for VM: {}", vm_id);

    let operation = state
        .resolve_operation(OperationKind::Query, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::get_migration_info(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// List available evdev input devices on the host
#[tauri::command]
pub async fn list_evdev_devices(
    state: State<'_, AppState>,
) -> Result<Vec<EvdevDevice>, SafeFailure> {
    tracing::info!("list_evdev_devices command called");
    state
        .resolve_host_local_operation(None)
        .map_err(SafeFailure::from)?;
    VmService::list_evdev_devices().map_err(SafeFailure::from)
}

/// Attach an evdev input device to a VM for low-latency passthrough
#[tauri::command]
pub async fn attach_evdev(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    device_path: String,
    grab_all: bool,
) -> Result<(), SafeFailure> {
    tracing::info!("attach_evdev command called for VM: {}", vm_id);
    let operation = state
        .resolve_host_local_operation(Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;

    VmService::attach_evdev(&operation.connection, &vm_id, &device_path, grab_all)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    let _ = app.emit(
        "vm-evdev-updated",
        serde_json::json!({
            "vmId": vm_id,
            "devicePath": device_path,
            "action": "attached",
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

    Ok(())
}

/// Get list of evdev devices attached to a VM
#[tauri::command]
pub async fn get_vm_evdev_devices(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<Vec<String>, SafeFailure> {
    tracing::info!("get_vm_evdev_devices command called for VM: {}", vm_id);
    let operation = state
        .resolve_host_local_operation(Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::get_vm_evdev_devices(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Remove an evdev device from a VM
#[tauri::command]
pub async fn detach_evdev(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    device_path: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!("detach_evdev command called for VM: {}", vm_id);
    let operation = state
        .resolve_host_local_operation(Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;

    let stable_id = format!("{vm_id}/{device_path}");
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "detach_evdev",
        "evdev_device",
        &stable_id,
        None,
        "detach",
    )
    .map_err(SafeFailure::from)?;
    VmService::detach_evdev(&operation.connection, &vm_id, &device_path)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    let _ = app.emit(
        "vm-evdev-updated",
        serde_json::json!({
            "vmId": vm_id,
            "devicePath": device_path,
            "action": "detached",
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );

    Ok(())
}

/// Check if a VM can be migrated (check for blockers like PCI passthrough, local disks)
#[tauri::command]
pub async fn check_migration_compatibility(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(bool, Vec<String>), SafeFailure> {
    tracing::debug!(
        "check_migration_compatibility command called for VM: {}",
        vm_id
    );
    let operation = state
        .resolve_operation(OperationKind::Query, Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    VmService::check_migration_compatibility(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Get list of available migration targets (saved remote connections)
#[tauri::command]
pub async fn get_migration_targets(
    state: State<'_, AppState>,
) -> Result<Vec<(String, String)>, SafeFailure> {
    tracing::debug!("get_migration_targets command called");
    VmService::get_migration_targets(&state.connections).map_err(SafeFailure::from)
}
