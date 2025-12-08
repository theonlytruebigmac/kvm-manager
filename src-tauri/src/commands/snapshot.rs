use tauri::{AppHandle, Emitter, State};
use crate::models::snapshot::{Snapshot, SnapshotConfig};
use crate::services::snapshot_service::SnapshotService;
use crate::state::app_state::AppState;

/// Get all snapshots for a VM
#[tauri::command]
pub async fn get_snapshots(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<Vec<Snapshot>, String> {
    tracing::debug!("get_snapshots command called for VM: {}", vm_id);
    SnapshotService::list_snapshots(&state.libvirt, &vm_id)
        .map_err(|e| e.to_string())
}

/// Create a new snapshot
#[tauri::command]
pub async fn create_snapshot(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    config: SnapshotConfig,
) -> Result<String, String> {
    tracing::debug!("create_snapshot command called for VM: {}", vm_id);

    let snapshot_name = SnapshotService::create_snapshot(&state.libvirt, &vm_id, config)
        .map_err(|e| e.to_string())?;

    // Emit event for real-time updates
    let _ = app.emit("snapshot-created", serde_json::json!({
        "vmId": vm_id,
        "snapshotName": snapshot_name,
        "timestamp": chrono::Utc::now().timestamp()
    }));

    Ok(snapshot_name)
}

/// Delete a snapshot
#[tauri::command]
pub async fn delete_snapshot(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    snapshot_name: String,
) -> Result<(), String> {
    tracing::debug!("delete_snapshot command called: {} from VM {}", snapshot_name, vm_id);

    SnapshotService::delete_snapshot(&state.libvirt, &vm_id, &snapshot_name)
        .map_err(|e| e.to_string())?;

    // Emit event for real-time updates
    let _ = app.emit("snapshot-deleted", serde_json::json!({
        "vmId": vm_id,
        "snapshotName": snapshot_name,
        "timestamp": chrono::Utc::now().timestamp()
    }));

    Ok(())
}

/// Revert VM to a snapshot
#[tauri::command]
pub async fn revert_snapshot(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    snapshot_name: String,
) -> Result<(), String> {
    tracing::debug!("revert_snapshot command called: {} for VM {}", snapshot_name, vm_id);

    SnapshotService::revert_snapshot(&state.libvirt, &vm_id, &snapshot_name)
        .map_err(|e| e.to_string())?;

    // Emit event for real-time updates
    let _ = app.emit("snapshot-reverted", serde_json::json!({
        "vmId": vm_id,
        "snapshotName": snapshot_name,
        "timestamp": chrono::Utc::now().timestamp()
    }));

    Ok(())
}
