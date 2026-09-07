use crate::models::operation::{OperationKind, TargetIdentity};
use crate::models::snapshot::{Snapshot, SnapshotConfig};
use crate::services::snapshot_service::SnapshotService;
use crate::state::app_state::AppState;
use crate::utils::error::SafeFailure;
use tauri::{AppHandle, Emitter, State};

fn snapshot_target(vm_id: &str, snapshot_name: Option<&str>) -> TargetIdentity {
    TargetIdentity {
        resource_kind: "snapshot".to_string(),
        stable_id: snapshot_name
            .map(|name| format!("{vm_id}/{name}"))
            .unwrap_or_else(|| vm_id.to_string()),
        display_name: snapshot_name.map(str::to_string),
    }
}

/// Get all snapshots for a VM
#[tauri::command]
pub async fn get_snapshots(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<Vec<Snapshot>, SafeFailure> {
    tracing::debug!("get_snapshots command called for VM: {}", vm_id);
    let operation = state
        .resolve_operation(OperationKind::Query, Some(snapshot_target(&vm_id, None)))
        .map_err(SafeFailure::from)?;
    SnapshotService::list_snapshots(&operation.connection, &vm_id)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Create a new snapshot
#[tauri::command]
pub async fn create_snapshot(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    config: SnapshotConfig,
) -> Result<String, SafeFailure> {
    tracing::debug!("create_snapshot command called for VM: {}", vm_id);

    let operation = state
        .resolve_operation(
            OperationKind::Mutation,
            Some(snapshot_target(&vm_id, Some(&config.name))),
        )
        .map_err(SafeFailure::from)?;
    let snapshot_name = SnapshotService::create_snapshot(&operation.connection, &vm_id, config)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event for real-time updates
    let _ = app.emit(
        "snapshot-created",
        serde_json::json!({
            "vmId": vm_id,
            "snapshotName": snapshot_name,
            "timestamp": chrono::Utc::now().timestamp()
        }),
    );

    Ok(snapshot_name)
}

/// Delete a snapshot
#[tauri::command]
pub async fn delete_snapshot(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    snapshot_name: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::debug!(
        "delete_snapshot command called: {} from VM {}",
        snapshot_name,
        vm_id
    );

    let stable_id = format!("{vm_id}/{snapshot_name}");
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "delete_snapshot",
        "snapshot",
        &stable_id,
        Some(snapshot_name.clone()),
        "delete",
    )
    .map_err(SafeFailure::from)?;
    let operation = state
        .resolve_operation(
            OperationKind::Mutation,
            Some(snapshot_target(&vm_id, Some(&snapshot_name))),
        )
        .map_err(SafeFailure::from)?;
    SnapshotService::delete_snapshot(&operation.connection, &vm_id, &snapshot_name)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event for real-time updates
    let _ = app.emit(
        "snapshot-deleted",
        serde_json::json!({
            "vmId": vm_id,
            "snapshotName": snapshot_name,
            "timestamp": chrono::Utc::now().timestamp()
        }),
    );

    Ok(())
}

/// Revert VM to a snapshot
#[tauri::command]
pub async fn revert_snapshot(
    app: AppHandle,
    state: State<'_, AppState>,
    vm_id: String,
    snapshot_name: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::debug!(
        "revert_snapshot command called: {} for VM {}",
        snapshot_name,
        vm_id
    );

    let stable_id = format!("{vm_id}/{snapshot_name}");
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "revert_snapshot",
        "snapshot",
        &stable_id,
        Some(snapshot_name.clone()),
        "revert",
    )
    .map_err(SafeFailure::from)?;
    let operation = state
        .resolve_operation(
            OperationKind::Mutation,
            Some(snapshot_target(&vm_id, Some(&snapshot_name))),
        )
        .map_err(SafeFailure::from)?;
    SnapshotService::revert_snapshot(&operation.connection, &vm_id, &snapshot_name)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))?;

    // Emit event for real-time updates
    let _ = app.emit(
        "snapshot-reverted",
        serde_json::json!({
            "vmId": vm_id,
            "snapshotName": snapshot_name,
            "timestamp": chrono::Utc::now().timestamp()
        }),
    );

    Ok(())
}
