use serde::{Deserialize, Serialize};

/// Represents a USB device on the host system
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsbDevice {
    /// Bus number (e.g., 001)
    pub bus: String,
    /// Device number on the bus (e.g., 002)
    pub device: String,
    /// USB Vendor ID (e.g., 046d)
    pub vendor_id: String,
    /// USB Product ID (e.g., c52b)
    pub product_id: String,
    /// Vendor name (e.g., Logitech)
    pub vendor_name: String,
    /// Product name (e.g., USB Receiver)
    pub product_name: String,
    /// Device description from lsusb
    pub description: String,
    /// USB speed (1.5, 12, 480, 5000, 10000, 20000 Mbps)
    pub speed: Option<String>,
    /// Device class (e.g., Human Interface Device, Mass Storage)
    pub device_class: Option<String>,
    /// Whether the device is in use by a VM
    pub in_use: bool,
    /// Which VM is using this device (if any)
    pub used_by_vm: Option<String>,
}

impl UsbDevice {
    /// Get a human-readable display name
    pub fn display_name(&self) -> String {
        if !self.product_name.is_empty() {
            format!("{} {}", self.vendor_name, self.product_name)
        } else {
            format!("{}:{}", self.vendor_id, self.product_id)
        }
    }

    /// Get unique identifier for the device
    #[allow(dead_code)]
    pub fn identifier(&self) -> String {
        format!("{}:{}", self.vendor_id, self.product_id)
    }
}
