use serde::{Deserialize, Serialize};

/// VM State enumeration matching libvirt domain states
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VmState {
    Running,
    Stopped,
    Paused,
    Suspended,
}

/// Network Interface configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterface {
    pub name: String,
    pub mac_address: String,
    pub network: String,
}

/// Virtual Machine model
/// Matches the contract defined in .agents/integration/tauri-commands.md
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VM {
    pub id: String,
    pub name: String,
    pub state: VmState,
    pub cpu_count: u32,
    pub memory_mb: u64,
    pub disk_size_gb: u64,
    pub network_interfaces: Vec<NetworkInterface>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// VM Configuration for creating new VMs
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VmConfig {
    pub name: String,
    pub cpu_count: u32,
    pub memory_mb: u64,
    pub disk_size_gb: u64,
    #[serde(default = "default_os_type")]
    pub os_type: String,
    #[serde(default)]
    pub iso_path: Option<String>,
    #[serde(default = "default_network")]
    pub network: String,
    #[serde(default = "default_disk_format")]
    pub disk_format: String,
    #[serde(default)]
    pub boot_menu: bool,
}

fn default_os_type() -> String {
    "linux".to_string()
}

fn default_network() -> String {
    "default".to_string()
}

fn default_disk_format() -> String {
    "qcow2".to_string()
}

/// VM Performance Statistics
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VmStats {
    pub vm_id: String,
    pub cpu_usage_percent: f64,
    pub memory_used_mb: u64,
    pub memory_available_mb: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub timestamp: i64,
}
