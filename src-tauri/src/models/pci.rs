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

#[allow(dead_code)]
fn default_managed() -> bool {
    true
}
