use serde::{Deserialize, Serialize};

/// PCI Device Type enumeration
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PciDeviceType {
    Gpu,
    Audio,
    UsbController,
    NetworkAdapter,
    StorageController,
    Other,
}

/// PCI Device information
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PciDevice {
    /// Device address (e.g., "0000:01:00.0")
    pub address: String,
    /// Vendor name
    pub vendor: String,
    /// Vendor ID (hex)
    pub vendor_id: String,
    /// Device name/model
    pub device_name: String,
    /// Device ID (hex)
    pub device_id: String,
    /// Device class (e.g., "VGA compatible controller")
    pub device_class: String,
    /// Device type
    pub device_type: PciDeviceType,
    /// IOMMU group number
    pub iommu_group: Option<u32>,
    /// Whether the device is currently in use (attached to a VM or host driver)
    pub in_use: bool,
    /// Name of VM this device is attached to (if any)
    pub attached_to_vm: Option<String>,
    /// Kernel driver in use
    pub driver: Option<String>,
    /// Whether this device can be safely passed through
    pub passthrough_safe: bool,
}

/// IOMMU Group information
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IommuGroup {
    /// Group number
    pub group_id: u32,
    /// List of PCI device addresses in this group
    pub devices: Vec<String>,
    /// Whether all devices in this group can be passed through together
    pub passthrough_safe: bool,
}

/// IOMMU Status
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IommuStatus {
    /// Whether IOMMU is enabled on the host
    pub enabled: bool,
    /// IOMMU type (e.g., "Intel VT-d", "AMD-Vi")
    pub iommu_type: Option<String>,
    /// Warning message if IOMMU is not properly configured
    pub warning: Option<String>,
}

/// PCI device configuration for VM
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PciPassthroughConfig {
    /// PCI device address
    pub address: String,
    /// Whether to use managed mode (libvirt handles driver unbinding)
    #[serde(default = "default_managed")]
    pub managed: bool,
}

/// SR-IOV Physical Function (PF) information
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SriovPf {
    /// PCI address of the Physical Function
    pub address: String,
    /// Device name
    pub device_name: String,
    /// Vendor name
    pub vendor: String,
    /// Network interface name (e.g., enp1s0f0)
    pub interface_name: Option<String>,
    /// Maximum number of VFs supported
    pub max_vfs: u32,
    /// Number of VFs currently enabled
    pub num_vfs: u32,
    /// Total VFs available (num_vfs - in_use)
    pub available_vfs: u32,
    /// Driver in use
    pub driver: Option<String>,
    /// Link speed (e.g., "10 Gb/s")
    pub link_speed: Option<String>,
}

/// SR-IOV Virtual Function (VF) information
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SriovVf {
    /// PCI address of the Virtual Function
    pub address: String,
    /// VF index (0, 1, 2, etc.)
    pub vf_index: u32,
    /// Parent PF address
    pub parent_pf: String,
    /// MAC address (if configured)
    pub mac_address: Option<String>,
    /// VLAN ID (if configured)
    pub vlan_id: Option<u16>,
    /// Whether this VF is attached to a VM
    pub in_use: bool,
    /// VM name this VF is attached to
    pub attached_to_vm: Option<String>,
    /// IOMMU group
    pub iommu_group: Option<u32>,
}

/// SR-IOV configuration for a VF
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SriovVfConfig {
    /// PF interface name (e.g., enp1s0f0)
    pub pf_interface: String,
    /// VF index to configure
    pub vf_index: u32,
    /// Optional MAC address
    pub mac_address: Option<String>,
    /// Optional VLAN ID
    pub vlan_id: Option<u16>,
    /// Whether to enable spoof check
    #[serde(default = "default_true")]
    pub spoof_check: bool,
    /// Whether to enable trust mode
    #[serde(default)]
    pub trust: bool,
}

fn default_true() -> bool {
    true
}

#[allow(dead_code)]
fn default_managed() -> bool {
    true
}
