use tauri::State;
use crate::models::host::{HostInfo, ConnectionStatus, VncInfo};
use crate::state::app_state::AppState;

/// Get host information from libvirt
#[tauri::command]
pub async fn get_host_info(state: State<'_, AppState>) -> Result<HostInfo, String> {
    use crate::services::vm_service::VmService;

    tracing::debug!("get_host_info command called");

    let libvirt = &state.libvirt;

    let hostname = libvirt.get_hostname()
        .map_err(|e| e.to_string())?;

    let libvirt_version = libvirt.get_version()
        .map_err(|e| e.to_string())?;

    // Get host CPU info
    let conn = libvirt.get_connection();
    let node_info = conn.get_node_info()
        .map_err(|e| format!("Failed to get node info: {}", e))?;

    let cpu_model = node_info.model;
    let cpu_count = node_info.cpus;
    let cpu_threads = node_info.threads;
    let memory_total_mb = node_info.memory / 1024; // Convert from KiB to MiB

    // Get free memory
    let memory_stats = conn.get_free_memory()
        .map_err(|e| format!("Failed to get free memory: {}", e))?;
    let memory_free_mb = memory_stats / (1024 * 1024); // Convert from bytes to MiB

    // Get hypervisor info
    let qemu_version = "QEMU 8.0+".to_string(); // Placeholder
    let hypervisor = conn.get_type()
        .map_err(|e| format!("Failed to get hypervisor type: {}", e))?;

    // Get VM counts
    let all_vms = VmService::list_vms(&state.libvirt)
        .map_err(|e| e.to_string())?;
    let total_vms = all_vms.len() as u32;
    let active_vms = all_vms.iter()
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
pub async fn get_connection_status(state: State<'_, AppState>) -> Result<ConnectionStatus, String> {
    tracing::debug!("get_connection_status command called");

    let libvirt = &state.libvirt;
    let connected = libvirt.is_alive();

    Ok(ConnectionStatus {
        connected,
        uri: "qemu:///system".to_string(),
        error: if connected { None } else { Some("Connection lost".to_string()) },
    })
}

/// Get graphics display info for a VM (VNC or SPICE, starts websockify proxy if needed)
#[tauri::command]
pub async fn get_vnc_info(state: State<'_, AppState>, vm_id: String) -> Result<VncInfo, String> {
    use virt::domain::Domain;
    use virt::sys;

    tracing::debug!("get_vnc_info command called for VM: {}", vm_id);

    let conn = state.libvirt.get_connection();
    let domain = Domain::lookup_by_uuid_string(conn, &vm_id)
        .map_err(|_| format!("VM not found: {}", vm_id))?;

    // Check if VM is running
    let info = domain.get_info()
        .map_err(|e| format!("Failed to get VM info: {}", e))?;

    if info.state != sys::VIR_DOMAIN_RUNNING {
        return Err("VM is not running".to_string());
    }

    let xml = domain.get_xml_desc(0)
        .map_err(|e| format!("Failed to get VM XML: {}", e))?;

    // Try to find VNC first, then SPICE
    let (graphics_port, graphics_type) = if let Some(start) = xml.find("<graphics type='vnc'") {
        // Parse VNC port
        let graphics_section = &xml[start..];
        if let Some(port_start) = graphics_section.find("port='") {
            let port_section = &graphics_section[port_start + 6..];
            if let Some(port_end) = port_section.find("'") {
                let port_str = &port_section[..port_end];
                let port = port_str.parse::<u16>()
                    .map_err(|_| "Failed to parse VNC port".to_string())?;
                (port, "vnc")
            } else {
                return Err("VNC port not found in configuration".to_string());
            }
        } else {
            return Err("VNC port not configured".to_string());
        }
    } else if let Some(start) = xml.find("<graphics type='spice'") {
        // Parse SPICE port
        let graphics_section = &xml[start..];
        if let Some(port_start) = graphics_section.find("port='") {
            let port_section = &graphics_section[port_start + 6..];
            if let Some(port_end) = port_section.find("'") {
                let port_str = &port_section[..port_end];
                let port = port_str.parse::<u16>()
                    .map_err(|_| "Failed to parse SPICE port".to_string())?;
                (port, "spice")
            } else {
                return Err("SPICE port not found in configuration".to_string());
            }
        } else {
            return Err("SPICE port not configured".to_string());
        }
    } else {
        return Err("No graphics device (VNC or SPICE) configured for this VM".to_string());
    };

    // Start WebSocket proxy for this VM (native Rust implementation)
    let ws_port = state.ws_proxy.start_proxy(&vm_id, graphics_port).await
        .map_err(|e| e.to_string())?;

    tracing::info!("{} info for VM {}: port {}, WebSocket port {}",
        graphics_type.to_uppercase(), vm_id, graphics_port, ws_port);

    Ok(VncInfo {
        host: "127.0.0.1".to_string(),
        port: ws_port,  // Return WebSocket port, not raw graphics port
        password: None,
        websocket_port: Some(ws_port),
        graphics_type: Some(graphics_type.to_string()),
    })
}

/// Launch external VNC viewer for a VM
#[tauri::command]
pub async fn open_vnc_console(state: State<'_, AppState>, vm_id: String) -> Result<(), String> {
    use virt::domain::Domain;

    tracing::debug!("open_vnc_console command called for VM: {}", vm_id);

    let conn = state.libvirt.get_connection();
    let domain = Domain::lookup_by_uuid_string(conn, &vm_id)
        .map_err(|_| format!("VM not found: {}", vm_id))?;

    // Get domain name for virt-viewer
    let name = domain.get_name()
        .map_err(|e| format!("Failed to get VM name: {}", e))?;

    // Try to launch virt-viewer (preferred) or remote-viewer
    let viewers = vec!["virt-viewer", "remote-viewer"];
    let mut last_error = String::new();

    for viewer in viewers {
        tracing::info!("Attempting to launch {} for VM: {}", viewer, name);

        match std::process::Command::new(viewer)
            .arg("--connect")
            .arg("qemu:///system")
            .arg(&name)
            .stderr(std::process::Stdio::null()) // Suppress GTK warnings
            .spawn()
        {
            Ok(_) => {
                tracing::info!("Successfully launched {} for VM: {}", viewer, name);
                return Ok(());
            }
            Err(e) => {
                last_error = format!("{}: {}", viewer, e);
                tracing::warn!("Failed to launch {}: {}", viewer, e);
            }
        }
    }

    Err(format!(
        "No VNC viewer found. Please install virt-viewer or remote-viewer. Last error: {}",
        last_error
    ))
}

/// Stop the WebSocket proxy for a VM (called when console window closes)
#[tauri::command]
pub async fn stop_vnc_proxy(state: State<'_, AppState>, vm_id: String) -> Result<(), String> {
    tracing::debug!("stop_vnc_proxy command called for VM: {}", vm_id);

    state.ws_proxy.stop_proxy(&vm_id).await
        .map_err(|e| e.to_string())
}
