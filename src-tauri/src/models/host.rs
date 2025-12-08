use serde::Serialize;

/// Host Information
/// Matches the contract defined in .agents/integration/tauri-commands.md
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HostInfo {
    pub hostname: String,
    pub cpu_model: String,
    pub cpu_count: u32,
    pub cpu_threads: u32,
    pub memory_total_mb: u64,
    pub memory_free_mb: u64,
    pub libvirt_version: String,
    pub qemu_version: String,
    pub hypervisor: String,
    pub active_vms: u32,
    pub total_vms: u32,
}

/// Connection Status
#[derive(Serialize, Clone, Debug)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub uri: String,
    pub error: Option<String>,
}

/// VNC Connection Information
#[derive(Serialize, Clone, Debug)]
pub struct VncInfo {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub websocket_port: Option<u16>,
}
