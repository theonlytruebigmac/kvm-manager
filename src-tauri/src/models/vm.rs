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
    #[serde(rename = "mac")]
    pub mac: String,  // Alias for mac_address for frontend compatibility
    pub network: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub model_type: Option<String>,  // NIC model type (virtio, e1000, etc.)
    // QoS / Bandwidth settings (in KB/s for average, peak; KB for burst)
    #[serde(default)]
    pub inbound_average: Option<u64>,
    #[serde(default)]
    pub inbound_peak: Option<u64>,
    #[serde(default)]
    pub inbound_burst: Option<u64>,
    #[serde(default)]
    pub outbound_average: Option<u64>,
    #[serde(default)]
    pub outbound_peak: Option<u64>,
    #[serde(default)]
    pub outbound_burst: Option<u64>,
}

/// Disk Device information
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DiskDevice {
    pub device: String,
    pub path: String,
    pub disk_type: String,
    pub bus: String,
    /// Cache mode: none, writeback, writethrough, directsync, unsafe
    #[serde(default)]
    pub cache: Option<String>,
    /// I/O mode: native, threads, io_uring
    #[serde(default)]
    pub io: Option<String>,
    /// Discard mode: unmap, ignore
    #[serde(default)]
    pub discard: Option<String>,
    /// Detect zeroes: off, on, unmap
    #[serde(default)]
    pub detect_zeroes: Option<String>,
    /// Read IOPS limit
    #[serde(default)]
    pub read_iops_sec: Option<u64>,
    /// Write IOPS limit
    #[serde(default)]
    pub write_iops_sec: Option<u64>,
    /// Read bandwidth limit (bytes/sec)
    #[serde(default)]
    pub read_bytes_sec: Option<u64>,
    /// Write bandwidth limit (bytes/sec)
    #[serde(default)]
    pub write_bytes_sec: Option<u64>,
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
    #[serde(default)]
    pub max_memory_mb: u64,
    pub disk_size_gb: u64,
    pub network_interfaces: Vec<NetworkInterface>,
    #[serde(default)]
    pub disks: Vec<DiskDevice>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_firmware")]
    pub firmware: String,
    #[serde(default)]
    pub tpm_enabled: bool,
    #[serde(default = "default_chipset")]
    pub chipset: String,
    #[serde(default = "default_cpu_sockets")]
    pub cpu_sockets: u32,
    #[serde(default = "default_cpu_cores")]
    pub cpu_cores: u32,
    #[serde(default = "default_cpu_threads")]
    pub cpu_threads: u32,
    #[serde(default)]
    pub cloud_init: Option<super::cloud_init::CloudInitConfig>,
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
    #[serde(default = "default_boot_order")]
    pub boot_order: Vec<String>,
    #[serde(default = "default_firmware")]
    pub firmware: String,
    #[serde(default)]
    pub tpm_enabled: bool,
    #[serde(default = "default_chipset")]
    pub chipset: String,
    #[serde(default = "default_cpu_sockets")]
    pub cpu_sockets: u32,
    #[serde(default = "default_cpu_cores")]
    pub cpu_cores: u32,
    #[serde(default = "default_cpu_threads")]
    pub cpu_threads: u32,
    #[serde(default)]
    pub cloud_init: Option<super::cloud_init::CloudInitConfig>,
    #[serde(default)]
    pub pci_devices: Vec<String>,
    #[serde(default = "default_graphics_type")]
    pub graphics_type: String,
    #[serde(default = "default_video_model")]
    pub video_model: String,
    #[serde(default)]
    pub rng_enabled: bool,
    #[serde(default)]
    pub watchdog_model: Option<String>,
    #[serde(default = "default_watchdog_action")]
    pub watchdog_action: String,
    // Installation type: iso, import (existing disk), network, or manual (no media)
    #[serde(default = "default_installation_type")]
    pub installation_type: String,
    // Path to existing disk image when installation_type is "import"
    #[serde(default)]
    pub existing_disk_path: Option<String>,
    // URL for network installation (HTTP/HTTPS/FTP)
    #[serde(default)]
    pub network_install_url: Option<String>,
    // Direct kernel boot options
    #[serde(default)]
    pub direct_kernel_boot: bool,
    #[serde(default)]
    pub kernel_path: Option<String>,
    #[serde(default)]
    pub initrd_path: Option<String>,
    #[serde(default)]
    pub kernel_args: Option<String>,
    #[serde(default)]
    pub dtb_path: Option<String>,  // Device tree blob for ARM
    // Hugepages memory backing
    #[serde(default)]
    pub hugepages_enabled: bool,
    #[serde(default)]
    pub hugepage_size: Option<u64>,  // Size in KiB (2048 = 2MB, 1048576 = 1GB)
}

fn default_installation_type() -> String {
    "iso".to_string()
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

fn default_boot_order() -> Vec<String> {
    vec!["hd".to_string()]
}

fn default_firmware() -> String {
    "bios".to_string()
}

fn default_chipset() -> String {
    "pc".to_string()
}

fn default_cpu_sockets() -> u32 {
    1
}

fn default_cpu_cores() -> u32 {
    1
}

fn default_cpu_threads() -> u32 {
    1
}

fn default_graphics_type() -> String {
    "vnc".to_string()
}

fn default_video_model() -> String {
    "qxl".to_string()
}

fn default_watchdog_action() -> String {
    "reset".to_string()
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

/// Host NUMA node information
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HostNumaNode {
    pub id: u32,
    pub cpus: Vec<u32>,
    pub memory_mb: u64,
}

/// VM NUMA configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VmNumaConfig {
    /// Memory mode: strict, preferred, interleave
    pub mode: String,
    /// List of host NUMA nodes to use (comma-separated like "0,1")
    pub nodeset: Option<String>,
    /// Guest NUMA cells (for advanced configuration)
    pub cells: Vec<VmNumaCell>,
}

/// Guest NUMA cell configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VmNumaCell {
    pub id: u32,
    pub cpus: String,
    pub memory_mb: u64,
    /// Host NUMA nodes this cell maps to
    pub host_nodes: Option<String>,
}
