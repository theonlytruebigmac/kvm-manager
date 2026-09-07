use crate::models::host::{
    ConnectionStatus, GuestCapabilityReview, HostInfo, ReadinessRepairResult, VmCreationReadiness,
    VncInfo,
};
use crate::models::operation::{ConnectionScope, MutationOutcome, OperationKind, TargetIdentity};
use crate::models::vm::VmConfig;
use crate::services::distribution_profile_service::DistributionProfileService;
use crate::services::host_readiness_service::HostReadinessService;
use crate::services::readiness_repair_service::ReadinessRepairService;
use crate::state::app_state::AppState;
use crate::utils::error::{AppError, SafeFailure, SafeFailureCode};
use tauri::State;

fn vm_target(vm_id: &str) -> TargetIdentity {
    TargetIdentity {
        resource_kind: "vm".to_string(),
        stable_id: vm_id.to_string(),
        display_name: None,
    }
}

fn integration_failure(summary: &'static str) -> SafeFailure {
    SafeFailure::new(SafeFailureCode::Integration, summary)
}

/// Return a non-destructive assessment owned by the captured active connection.
#[tauri::command]
pub async fn get_host_readiness(
    state: State<'_, AppState>,
) -> Result<VmCreationReadiness, SafeFailure> {
    tracing::debug!("get_host_readiness command called");
    get_readiness(&state, None)
}

#[tauri::command]
pub async fn get_vm_creation_readiness(
    state: State<'_, AppState>,
    requested_disk_bytes: Option<u64>,
) -> Result<VmCreationReadiness, SafeFailure> {
    get_readiness(&state, requested_disk_bytes)
}

fn get_readiness(
    state: &State<'_, AppState>,
    requested_disk_bytes: Option<u64>,
) -> Result<VmCreationReadiness, SafeFailure> {
    let operation = state
        .resolve_operation(OperationKind::Query, None)
        .map_err(SafeFailure::from)?;
    HostReadinessService::connection_report(
        &operation.connection,
        &operation.context,
        requested_disk_bytes,
        None,
    )
    .map_err(|error| SafeFailure::from(error).with_context(&operation.context))
}

#[tauri::command]
pub async fn preflight_vm_creation(
    state: State<'_, AppState>,
    config: VmConfig,
) -> Result<GuestCapabilityReview, SafeFailure> {
    let operation = state
        .resolve_operation(OperationKind::Query, None)
        .map_err(SafeFailure::from)?;
    HostReadinessService::preflight(&operation.connection, &operation.context, &config)
        .map_err(|error| SafeFailure::from(error).with_context(&operation.context))
}

#[tauri::command]
pub async fn execute_readiness_repair(
    state: State<'_, AppState>,
    action_id: String,
    confirmation_token: String,
) -> Result<ReadinessRepairResult, SafeFailure> {
    if !ReadinessRepairService::is_known_action(&action_id) {
        return Err(SafeFailure::from(AppError::InvalidConfig(
            "The requested readiness repair is not recognized".to_string(),
        )));
    }
    let operation = state
        .resolve_operation(
            OperationKind::Mutation,
            Some(TargetIdentity {
                resource_kind: "readiness_action".to_string(),
                stable_id: action_id.clone(),
                display_name: None,
            }),
        )
        .map_err(SafeFailure::from)?;
    if operation.context.connection_scope != ConnectionScope::LocalSystem {
        return Err(SafeFailure::from(AppError::InvalidConfig(
            "Automated readiness repair is available only for a local system connection"
                .to_string(),
        ))
        .with_context(&operation.context));
    }
    let profile = DistributionProfileService::detect_local();
    if !ReadinessRepairService::is_allowed(
        &operation.context.connection_scope,
        &action_id,
        &profile,
    ) {
        return Err(SafeFailure::from(AppError::InvalidConfig(
            "This readiness repair is not available for the detected distribution".to_string(),
        ))
        .with_context(&operation.context));
    }
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "execute_readiness_repair",
        "readiness_action",
        &action_id,
        None,
        "execute",
    )
    .map_err(SafeFailure::from)?;

    let repair_action_id = action_id.clone();
    let repair_connection_id = operation.context.connection_id.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        ReadinessRepairService::execute(&repair_action_id, &repair_connection_id, &profile)
    })
    .await
    .map_err(|_| {
        SafeFailure::from(AppError::Other(
            "The readiness repair could not be completed".to_string(),
        ))
    })?
    .map_err(SafeFailure::from)
    .map_err(|failure| failure.with_context(&operation.context))?;
    let outcome = if matches!(
        result.outcome,
        crate::models::host::ReadinessRepairOutcome::Applied
    ) {
        MutationOutcome::Applied
    } else {
        MutationOutcome::Rejected
    };
    crate::utils::diagnostics::operation_finished(&operation.context, outcome, None);
    Ok(result)
}

/// Get host information from libvirt
#[tauri::command]
pub async fn get_host_info(state: State<'_, AppState>) -> Result<HostInfo, SafeFailure> {
    use crate::services::vm_service::VmService;

    tracing::debug!("get_host_info command called");

    let operation = state
        .resolve_operation(OperationKind::Query, None)
        .map_err(SafeFailure::from)?;
    let conn = &operation.connection;

    let hostname = conn
        .get_hostname()
        .map_err(|_| integration_failure("The host information could not be read."))?;

    let libvirt_version_number = conn
        .get_lib_version()
        .map_err(|_| integration_failure("The host information could not be read."))?;
    let libvirt_version = format!(
        "{}.{}.{}",
        libvirt_version_number / 1_000_000,
        (libvirt_version_number % 1_000_000) / 1_000,
        libvirt_version_number % 1_000
    );

    // Get host CPU info
    let node_info = conn
        .get_node_info()
        .map_err(|_| integration_failure("The host information could not be read."))?;

    let cpu_model = node_info.model;
    let cpu_count = node_info.cpus;
    let cpu_threads = node_info.threads;
    let memory_total_mb = node_info.memory / 1024; // Convert from KiB to MiB

    // Get free memory
    let memory_stats = conn
        .get_free_memory()
        .map_err(|_| integration_failure("The host information could not be read."))?;
    let memory_free_mb = memory_stats / (1024 * 1024); // Convert from bytes to MiB

    // Get the connected hypervisor version rather than reporting a fabricated QEMU version.
    let qemu_version = conn
        .get_hyp_version()
        .map(|version| {
            format!(
                "{}.{}.{}",
                version / 1_000_000,
                (version % 1_000_000) / 1_000,
                version % 1_000
            )
        })
        .unwrap_or_else(|_| "Unavailable".to_string());
    let hypervisor = conn
        .get_type()
        .map_err(|_| integration_failure("The host information could not be read."))?;

    // Get VM counts
    let all_vms = VmService::list_vms(conn).map_err(SafeFailure::from)?;
    let total_vms = all_vms.len() as u32;
    let active_vms = all_vms
        .iter()
        .filter(|vm| vm.state == crate::models::vm::VmState::Running)
        .count() as u32;

    Ok(HostInfo {
        hostname,
        cpu_model,
        cpu_count,
        cpu_threads,
        memory_total_mb,
        memory_free_mb,
        libvirt_version,
        qemu_version,
        hypervisor,
        active_vms,
        total_vms,
    })
}

/// Get libvirt connection status
#[tauri::command]
pub async fn get_connection_status(
    state: State<'_, AppState>,
) -> Result<ConnectionStatus, SafeFailure> {
    tracing::debug!("get_connection_status command called");

    let active = state
        .connections
        .get_active_connection_info()
        .map_err(SafeFailure::from)?;
    let Some(active) = active else {
        return Ok(ConnectionStatus {
            connected: false,
            uri: "No active connection".to_string(),
            error: Some("No selected connection is active".to_string()),
        });
    };

    match state.resolve_operation(OperationKind::Query, None) {
        Ok(_) => Ok(ConnectionStatus {
            connected: true,
            uri: active.build_uri(),
            error: None,
        }),
        Err(error) => Ok(ConnectionStatus {
            connected: false,
            uri: active.build_uri(),
            error: Some(SafeFailure::from(error).summary),
        }),
    }
}

/// Get graphics display info for a VM (VNC or SPICE, starts websockify proxy if needed)
#[tauri::command]
pub async fn get_vnc_info(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<VncInfo, SafeFailure> {
    use virt::domain::Domain;
    use virt::sys;

    tracing::debug!("get_vnc_info command called for VM: {}", vm_id);

    let operation = state
        .resolve_console_operation(Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    let conn = &operation.connection;
    let domain = Domain::lookup_by_uuid_string(conn, &vm_id).map_err(|_| {
        SafeFailure::from(AppError::VmNotFound(vm_id.clone())).with_context(&operation.context)
    })?;

    // Check if VM is running
    let info = domain.get_info().map_err(|_| {
        integration_failure("The virtual machine state could not be read.")
            .with_context(&operation.context)
    })?;

    if info.state != sys::VIR_DOMAIN_RUNNING {
        return Err(
            SafeFailure::from(AppError::InvalidVmState(vm_id)).with_context(&operation.context)
        );
    }

    let xml = domain.get_xml_desc(0).map_err(|_| {
        integration_failure("The console configuration could not be read.")
            .with_context(&operation.context)
    })?;

    // Try to find VNC first, then SPICE
    let (graphics_port, graphics_type) = if let Some(start) = xml.find("<graphics type='vnc'") {
        // Parse VNC port
        let graphics_section = &xml[start..];
        if let Some(port_start) = graphics_section.find("port='") {
            let port_section = &graphics_section[port_start + 6..];
            if let Some(port_end) = port_section.find("'") {
                let port_str = &port_section[..port_end];
                let port = port_str.parse::<u16>().map_err(|_| {
                    SafeFailure::new(
                        SafeFailureCode::InvalidInput,
                        "The console configuration is invalid.",
                    )
                    .with_context(&operation.context)
                })?;
                (port, "vnc")
            } else {
                return Err(SafeFailure::new(
                    SafeFailureCode::InvalidInput,
                    "The console configuration is invalid.",
                )
                .with_context(&operation.context));
            }
        } else {
            return Err(SafeFailure::new(
                SafeFailureCode::Unsupported,
                "No graphical console is configured for this virtual machine.",
            )
            .with_context(&operation.context));
        }
    } else if let Some(start) = xml.find("<graphics type='spice'") {
        // Parse SPICE port
        let graphics_section = &xml[start..];
        if let Some(port_start) = graphics_section.find("port='") {
            let port_section = &graphics_section[port_start + 6..];
            if let Some(port_end) = port_section.find("'") {
                let port_str = &port_section[..port_end];
                let port = port_str.parse::<u16>().map_err(|_| {
                    SafeFailure::new(
                        SafeFailureCode::InvalidInput,
                        "The console configuration is invalid.",
                    )
                    .with_context(&operation.context)
                })?;
                (port, "spice")
            } else {
                return Err(SafeFailure::new(
                    SafeFailureCode::InvalidInput,
                    "The console configuration is invalid.",
                )
                .with_context(&operation.context));
            }
        } else {
            return Err(SafeFailure::new(
                SafeFailureCode::Unsupported,
                "No graphical console is configured for this virtual machine.",
            )
            .with_context(&operation.context));
        }
    } else {
        return Err(SafeFailure::new(
            SafeFailureCode::Unsupported,
            "No graphical console is configured for this virtual machine.",
        )
        .with_context(&operation.context));
    };

    // Start WebSocket proxy for this VM (native Rust implementation)
    let ws_port = state
        .ws_proxy
        .start_proxy(&vm_id, graphics_port)
        .await
        .map_err(|_| {
            integration_failure("The console connection could not be established.")
                .with_context(&operation.context)
        })?;

    tracing::info!(
        "{} info for VM {}: port {}, WebSocket port {}",
        graphics_type.to_uppercase(),
        vm_id,
        graphics_port,
        ws_port
    );

    Ok(VncInfo {
        host: "127.0.0.1".to_string(),
        port: ws_port, // Return WebSocket port, not raw graphics port
        password: None,
        websocket_port: Some(ws_port),
        graphics_type: Some(graphics_type.to_string()),
    })
}

/// Launch external VNC viewer for a VM
#[tauri::command]
pub async fn open_vnc_console(
    state: State<'_, AppState>,
    vm_id: String,
) -> Result<(), SafeFailure> {
    use virt::domain::Domain;

    tracing::debug!("open_vnc_console command called for VM: {}", vm_id);

    let operation = state
        .resolve_console_operation(Some(vm_target(&vm_id)))
        .map_err(SafeFailure::from)?;
    let conn = &operation.connection;
    let domain = Domain::lookup_by_uuid_string(conn, &vm_id).map_err(|_| {
        SafeFailure::from(AppError::VmNotFound(vm_id.clone())).with_context(&operation.context)
    })?;

    // Get domain name for virt-viewer
    let name = domain.get_name().map_err(|_| {
        integration_failure("The virtual machine could not be identified.")
            .with_context(&operation.context)
    })?;
    let connection_uri = conn.get_uri().map_err(|_| {
        integration_failure("The selected connection is unavailable.")
            .with_context(&operation.context)
    })?;

    // Try to launch virt-viewer (preferred) or remote-viewer
    let viewers = vec!["virt-viewer", "remote-viewer"];
    for viewer in viewers {
        tracing::info!("Attempting to launch {} for VM: {}", viewer, name);

        match std::process::Command::new(viewer)
            .arg("--connect")
            .arg(&connection_uri)
            .arg(&name)
            .stderr(std::process::Stdio::null()) // Suppress GTK warnings
            .spawn()
        {
            Ok(_) => {
                tracing::info!("Successfully launched {} for VM: {}", viewer, name);
                return Ok(());
            }
            Err(_) => {
                tracing::warn!("Console viewer launch failed");
            }
        }
    }

    Err(SafeFailure::new(
        SafeFailureCode::Unavailable,
        "No supported graphical console viewer is available.",
    )
    .with_context(&operation.context))
}

/// Stop the WebSocket proxy for a VM (called when console window closes)
#[tauri::command]
pub async fn stop_vnc_proxy(state: State<'_, AppState>, vm_id: String) -> Result<(), SafeFailure> {
    tracing::debug!("stop_vnc_proxy command called for VM: {}", vm_id);

    state
        .ws_proxy
        .stop_proxy(&vm_id)
        .await
        .map_err(|_| integration_failure("The console proxy could not be stopped."))
}
