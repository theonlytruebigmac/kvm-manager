use serde::{Deserialize, Serialize};

/// MDEV (Mediated Device) type information
/// Represents available vGPU types on the host
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MdevType {
    /// Type name (e.g., "nvidia-63", "i915-GVTg_V5_4")
    pub name: String,
    /// Human-readable description
    pub description: Option<String>,
    /// Available instances that can be created
    pub available_instances: u32,
    /// Maximum instances supported by this type
    pub max_instances: u32,
    /// Device API type (vfio-pci, etc.)
    pub device_api: String,
    /// Parent PCI device address
    pub parent_device: String,
    /// Parent device name/vendor
    pub parent_name: String,
}

/// Active MDEV instance
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MdevDevice {
    /// MDEV UUID
    pub uuid: String,
    /// MDEV type name
    pub mdev_type: String,
    /// Parent PCI device address
    pub parent_device: String,
    /// Whether this MDEV is in use by a VM
    pub in_use: bool,
    /// Name of VM using this MDEV (if any)
    pub used_by_vm: Option<String>,
}

/// MDEV support status on the host
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MdevStatus {
    /// Whether MDEV is supported on this system
    pub supported: bool,
    /// Message about MDEV status
    pub message: String,
    /// List of supported GPU vendors
    pub supported_vendors: Vec<String>,
}
