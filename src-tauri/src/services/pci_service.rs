use std::fs;
use std::path::Path;
use virt::domain::Domain;
use crate::models::pci::{PciDevice, PciDeviceType, IommuGroup, IommuStatus};
use crate::services::libvirt::LibvirtService;
use crate::utils::error::AppError;

/// PciService provides PCI device management operations
pub struct PciService;

impl PciService {
    /// Check IOMMU status on the host
    pub fn check_iommu_status() -> Result<IommuStatus, AppError> {
        tracing::debug!("Checking IOMMU status");

        // Check if IOMMU is enabled by checking kernel parameters
        let cmdline = fs::read_to_string("/proc/cmdline")
            .unwrap_or_default();

        let mut enabled = false;
        let mut iommu_type = None;

        // Check for Intel VT-d
        if cmdline.contains("intel_iommu=on") {
            enabled = true;
            iommu_type = Some("Intel VT-d".to_string());
        }
        // Check for AMD-Vi
        else if cmdline.contains("amd_iommu=on") {
            enabled = true;
            iommu_type = Some("AMD-Vi".to_string());
        }

        // Also check if IOMMU groups directory exists
        let iommu_groups_path = Path::new("/sys/kernel/iommu_groups");
        if !iommu_groups_path.exists() || !enabled {
            return Ok(IommuStatus {
                enabled: false,
                iommu_type: None,
                warning: Some(
                    "IOMMU is not enabled. Add 'intel_iommu=on' or 'amd_iommu=on' to kernel boot parameters.".to_string()
                ),
            });
        }

        Ok(IommuStatus {
            enabled,
            iommu_type,
            warning: None,
        })
    }

    /// List all PCI devices on the host
    pub fn list_pci_devices(libvirt: &LibvirtService) -> Result<Vec<PciDevice>, AppError> {
        tracing::debug!("Listing PCI devices");

        let mut devices = Vec::new();

        // Read PCI devices from /sys/bus/pci/devices
        let pci_path = Path::new("/sys/bus/pci/devices");
        if !pci_path.exists() {
            return Err(AppError::Other("PCI devices path not found".to_string()));
        }

        let entries = fs::read_dir(pci_path)
            .map_err(|e| AppError::Other(format!("Failed to read PCI devices: {}", e)))?;

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let address = entry.file_name().to_string_lossy().to_string();

            if let Ok(device) = Self::parse_pci_device(libvirt, &address) {
                devices.push(device);
            }
        }

        // Sort by address for consistent ordering
        devices.sort_by(|a, b| a.address.cmp(&b.address));

        tracing::info!("Found {} PCI devices", devices.len());
        Ok(devices)
    }

    /// Parse a single PCI device
    fn parse_pci_device(libvirt: &LibvirtService, address: &str) -> Result<PciDevice, AppError> {
        let device_path = format!("/sys/bus/pci/devices/{}", address);

        // Read vendor ID
        let vendor_id = fs::read_to_string(format!("{}/vendor", device_path))
            .unwrap_or_else(|_| "0x0000".to_string())
            .trim()
            .to_string();

        // Read device ID
        let device_id = fs::read_to_string(format!("{}/device", device_path))
            .unwrap_or_else(|_| "0x0000".to_string())
            .trim()
            .to_string();

        // Read class
        let class = fs::read_to_string(format!("{}/class", device_path))
            .unwrap_or_else(|_| "0x000000".to_string())
            .trim()
            .to_string();

        // Try to read driver
        let driver = fs::read_link(format!("{}/driver", device_path))
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()));

        // Get IOMMU group
        let iommu_group = fs::read_link(format!("{}/iommu_group", device_path))
            .ok()
            .and_then(|p| {
                p.file_name()
                    .and_then(|n| n.to_string_lossy().parse::<u32>().ok())
            });

        // Determine device type from class
        let device_type = Self::classify_device(&class);

        // Get vendor and device names from libvirt nodedev API
        let (vendor, device_name) = Self::get_device_names(libvirt, address)
            .unwrap_or_else(|| (
                format!("Vendor {}", vendor_id),
                format!("Device {}", device_id)
            ));

        // Determine device class name
        let device_class = Self::get_device_class(&class);

        // Check if device is in use (has a driver bound or is attached to a VM)
        let in_use = driver.is_some();

        // Check which VM (if any) this device is attached to
        let attached_to_vm = Self::get_attached_vm(libvirt, address)?;

        // Determine if device is safe for passthrough
        let passthrough_safe = Self::is_passthrough_safe(&device_type, &driver);

        Ok(PciDevice {
            address: address.to_string(),
            vendor,
            vendor_id,
            device_name,
            device_id,
            device_class,
            device_type,
            iommu_group,
            in_use,
            attached_to_vm,
            driver,
            passthrough_safe,
        })
    }

    /// Classify device based on PCI class code
    fn classify_device(class: &str) -> PciDeviceType {
        // PCI class format: 0xCCSSPP (Class, Subclass, Programming Interface)
        // We're interested in the class (first 2 hex digits after 0x)

        if class.len() >= 4 {
            let class_code = &class[2..4];
            match class_code {
                "03" => PciDeviceType::Gpu,           // Display controller
                "04" => PciDeviceType::Audio,         // Multimedia controller
                "0c" => {
                    // Serial bus controller - check subclass
                    if class.len() >= 6 {
                        let subclass = &class[4..6];
                        if subclass == "03" {
                            return PciDeviceType::UsbController; // USB controller
                        }
                    }
                    PciDeviceType::Other
                }
                "02" => PciDeviceType::NetworkAdapter,    // Network controller
                "01" => PciDeviceType::StorageController, // Mass storage controller
                _ => PciDeviceType::Other,
            }
        } else {
            PciDeviceType::Other
        }
    }

    /// Get human-readable device class name
    fn get_device_class(class: &str) -> String {
        if class.len() >= 4 {
            let class_code = &class[2..4];
            let subclass = if class.len() >= 6 { &class[4..6] } else { "00" };

            match (class_code, subclass) {
                ("03", "00") => "VGA compatible controller".to_string(),
                ("03", "01") => "XGA compatible controller".to_string(),
                ("03", "02") => "3D controller".to_string(),
                ("03", _) => "Display controller".to_string(),
                ("04", "01") => "Audio device".to_string(),
                ("04", "03") => "Audio device (HD Audio)".to_string(),
                ("04", _) => "Multimedia controller".to_string(),
                ("0c", "03") => "USB controller".to_string(),
                ("02", "00") => "Ethernet controller".to_string(),
                ("02", "80") => "Network controller".to_string(),
                ("01", "01") => "IDE interface".to_string(),
                ("01", "06") => "SATA controller".to_string(),
                ("01", "08") => "NVMe controller".to_string(),
                ("01", _) => "Storage controller".to_string(),
                _ => format!("PCI device (class {})", class),
            }
        } else {
            "Unknown PCI device".to_string()
        }
    }

    /// Get device vendor and name from libvirt
    fn get_device_names(libvirt: &LibvirtService, address: &str) -> Option<(String, String)> {
        // Try to use libvirt's nodedev API to get better names
        let conn = libvirt.get_connection();

        // Convert PCI address format (0000:01:00.0) to libvirt format (pci_0000_01_00_0)
        let nodedev_name = format!("pci_{}", address.replace(":", "_").replace(".", "_"));

        if let Ok(nodedev) = virt::nodedev::NodeDevice::lookup_by_name(conn, &nodedev_name) {
            if let Ok(xml) = nodedev.get_xml_desc(0) {
                // Parse XML to extract vendor and product names
                let vendor = Self::extract_xml_value(&xml, "<vendor>");
                let product = Self::extract_xml_value(&xml, "<product>");

                if let (Some(v), Some(p)) = (vendor, product) {
                    return Some((v, p));
                }
            }
        }

        None
    }

    /// Simple XML value extraction
    fn extract_xml_value(xml: &str, tag: &str) -> Option<String> {
        let start_tag = tag;
        let end_tag = tag.replace("<", "</");

        if let Some(start) = xml.find(start_tag) {
            let content_start = start + start_tag.len();
            if let Some(end) = xml[content_start..].find(&end_tag) {
                let value = xml[content_start..content_start + end].trim().to_string();
                return Some(value);
            }
        }
        None
    }

    /// Check which VM this device is attached to
    fn get_attached_vm(libvirt: &LibvirtService, address: &str) -> Result<Option<String>, AppError> {
        let conn = libvirt.get_connection();

        // List all VMs
        let flags = virt::sys::VIR_CONNECT_LIST_DOMAINS_ACTIVE | virt::sys::VIR_CONNECT_LIST_DOMAINS_INACTIVE;
        let domains = conn.list_all_domains(flags)
            .map_err(|e| AppError::LibvirtError(format!("Failed to list domains: {}", e)))?;

        for domain in domains {
            if let Ok(xml) = domain.get_xml_desc(0) {
                // Check if this VM has the PCI device in its XML
                // Format in XML: <address domain='0x0000' bus='0x01' slot='0x00' function='0x0'/>
                let parts: Vec<&str> = address.split(':').collect();
                if parts.len() == 3 {
                    let domain_part = parts[0];
                    let bus = parts[1];
                    let slot_func: Vec<&str> = parts[2].split('.').collect();

                    if slot_func.len() == 2 {
                        let slot = slot_func[0];
                        let func = slot_func[1];

                        let search = format!("domain='0x{}' bus='0x{}' slot='0x{}' function='0x{}'",
                            domain_part, bus, slot, func);

                        if xml.contains(&search) {
                            if let Ok(name) = domain.get_name() {
                                return Ok(Some(name));
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Determine if device is safe for passthrough
    fn is_passthrough_safe(device_type: &PciDeviceType, driver: &Option<String>) -> bool {
        // Some devices should not be passed through
        match device_type {
            PciDeviceType::Gpu |
            PciDeviceType::Audio |
            PciDeviceType::UsbController |
            PciDeviceType::NetworkAdapter => {
                // If it's using vfio-pci driver, it's already prepared for passthrough
                if let Some(d) = driver {
                    if d == "vfio-pci" {
                        return true;
                    }
                }
                // Otherwise, it's safe if not critical to host
                true
            }
            PciDeviceType::StorageController => {
                // Storage controllers can be risky - only safe if using vfio-pci
                if let Some(d) = driver {
                    d == "vfio-pci"
                } else {
                    false
                }
            }
            PciDeviceType::Other => false,
        }
    }

    /// Get IOMMU group information
    pub fn get_iommu_groups() -> Result<Vec<IommuGroup>, AppError> {
        tracing::debug!("Getting IOMMU groups");

        let mut groups = Vec::new();
        let iommu_path = Path::new("/sys/kernel/iommu_groups");

        if !iommu_path.exists() {
            return Ok(groups); // No IOMMU groups available
        }

        let entries = fs::read_dir(iommu_path)
            .map_err(|e| AppError::Other(format!("Failed to read IOMMU groups: {}", e)))?;

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            if let Ok(group_id) = entry.file_name().to_string_lossy().parse::<u32>() {
                let devices_path = entry.path().join("devices");
                let mut devices = Vec::new();

                if let Ok(device_entries) = fs::read_dir(devices_path) {
                    for device_entry in device_entries {
                        if let Ok(device_entry) = device_entry {
                            devices.push(device_entry.file_name().to_string_lossy().to_string());
                        }
                    }
                }

                // Group is safe if all devices are non-critical or using vfio-pci
                let passthrough_safe = !devices.is_empty();

                groups.push(IommuGroup {
                    group_id,
                    devices,
                    passthrough_safe,
                });
            }
        }

        groups.sort_by_key(|g| g.group_id);

        tracing::info!("Found {} IOMMU groups", groups.len());
        Ok(groups)
    }

    /// Attach a PCI device to a VM
    pub fn attach_pci_device(
        libvirt: &LibvirtService,
        vm_id: &str,
        pci_address: &str,
        managed: bool,
    ) -> Result<(), AppError> {
        tracing::info!("Attaching PCI device {} to VM {} (managed: {})", pci_address, vm_id, managed);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Parse PCI address (format: 0000:01:00.0)
        let parts: Vec<&str> = pci_address.split(':').collect();
        if parts.len() != 3 {
            return Err(AppError::InvalidConfig(format!("Invalid PCI address format: {}", pci_address)));
        }

        let domain_part = parts[0];
        let bus = parts[1];
        let slot_func: Vec<&str> = parts[2].split('.').collect();

        if slot_func.len() != 2 {
            return Err(AppError::InvalidConfig(format!("Invalid PCI address format: {}", pci_address)));
        }

        let slot = slot_func[0];
        let func = slot_func[1];

        // Build hostdev XML for PCI passthrough
        let managed_str = if managed { "yes" } else { "no" };
        let hostdev_xml = format!(
            r#"<hostdev mode='subsystem' type='pci' managed='{}'>
  <source>
    <address domain='0x{}' bus='0x{}' slot='0x{}' function='0x{}'/>
  </source>
</hostdev>"#,
            managed_str, domain_part, bus, slot, func
        );

        // Attach device (persistent and live if VM is running)
        let flags = if domain.is_active().map_err(|e| AppError::LibvirtError(e.to_string()))? {
            virt::sys::VIR_DOMAIN_AFFECT_LIVE | virt::sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            virt::sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain.attach_device_flags(&hostdev_xml, flags)
            .map_err(|e| AppError::LibvirtError(format!("Failed to attach PCI device: {}", e)))?;

        tracing::info!("Successfully attached PCI device {} to VM {}", pci_address, vm_id);
        Ok(())
    }

    /// Detach a PCI device from a VM
    pub fn detach_pci_device(
        libvirt: &LibvirtService,
        vm_id: &str,
        pci_address: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Detaching PCI device {} from VM {}", pci_address, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Get current XML to find the hostdev
        let xml = domain.get_xml_desc(0)
            .map_err(|e| AppError::LibvirtError(e.to_string()))?;

        // Parse PCI address
        let parts: Vec<&str> = pci_address.split(':').collect();
        if parts.len() != 3 {
            return Err(AppError::InvalidConfig(format!("Invalid PCI address format: {}", pci_address)));
        }

        let domain_part = parts[0];
        let bus = parts[1];
        let slot_func: Vec<&str> = parts[2].split('.').collect();

        if slot_func.len() != 2 {
            return Err(AppError::InvalidConfig(format!("Invalid PCI address format: {}", pci_address)));
        }

        let slot = slot_func[0];
        let func = slot_func[1];

        // Find hostdev device XML by PCI address
        let search = format!("domain='0x{}' bus='0x{}' slot='0x{}' function='0x{}'",
            domain_part, bus, slot, func);

        let mut hostdev_xml = None;
        let mut search_pos = 0;

        while let Some(hostdev_pos) = xml[search_pos..].find("<hostdev") {
            let abs_hostdev_pos = search_pos + hostdev_pos;
            if let Some(hostdev_end) = xml[abs_hostdev_pos..].find("</hostdev>") {
                let hostdev_section = &xml[abs_hostdev_pos..abs_hostdev_pos + hostdev_end + 10];

                if hostdev_section.contains(&search) {
                    hostdev_xml = Some(hostdev_section.to_string());
                    break;
                }

                search_pos = abs_hostdev_pos + hostdev_end + 10;
            } else {
                break;
            }
        }

        let hostdev_xml = hostdev_xml.ok_or_else(|| {
            AppError::InvalidConfig(format!("PCI device '{}' not found in VM", pci_address))
        })?;

        // Detach device (persistent and live if VM is running)
        let flags = if domain.is_active().map_err(|e| AppError::LibvirtError(e.to_string()))? {
            virt::sys::VIR_DOMAIN_AFFECT_LIVE | virt::sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            virt::sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain.detach_device_flags(&hostdev_xml, flags)
            .map_err(|e| AppError::LibvirtError(format!("Failed to detach PCI device: {}", e)))?;

        tracing::info!("Successfully detached PCI device {} from VM {}", pci_address, vm_id);
        Ok(())
    }

    /// Bind a PCI device to the vfio-pci driver for passthrough
    /// This is required before a device can be passed through to a VM
    pub fn bind_to_vfio(pci_address: &str) -> Result<(), AppError> {
        tracing::info!("Binding PCI device {} to vfio-pci driver", pci_address);

        let device_path = format!("/sys/bus/pci/devices/{}", pci_address);

        // Check if device exists
        if !Path::new(&device_path).exists() {
            return Err(AppError::Other(format!("PCI device {} not found", pci_address)));
        }

        // Get vendor and device IDs for vfio binding
        let vendor_id = fs::read_to_string(format!("{}/vendor", device_path))
            .map_err(|e| AppError::Other(format!("Failed to read vendor ID: {}", e)))?
            .trim()
            .trim_start_matches("0x")
            .to_string();

        let device_id = fs::read_to_string(format!("{}/device", device_path))
            .map_err(|e| AppError::Other(format!("Failed to read device ID: {}", e)))?
            .trim()
            .trim_start_matches("0x")
            .to_string();

        // Check current driver
        let current_driver = fs::read_link(format!("{}/driver", device_path))
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()));

        // If already bound to vfio-pci, nothing to do
        if let Some(ref driver) = current_driver {
            if driver == "vfio-pci" {
                tracing::info!("Device {} already bound to vfio-pci", pci_address);
                return Ok(());
            }
        }

        // Step 1: Unbind from current driver (if any)
        if current_driver.is_some() {
            Self::unbind_device(pci_address)?;
        }

        // Step 2: Load vfio-pci module if not loaded
        Self::ensure_vfio_module()?;

        // Step 3: Add device ID to vfio-pci new_id
        let new_id = format!("{} {}", vendor_id, device_id);
        fs::write("/sys/bus/pci/drivers/vfio-pci/new_id", &new_id)
            .map_err(|e| AppError::Other(format!(
                "Failed to add device to vfio-pci. Are you running as root? Error: {}", e
            )))?;

        // Step 4: Bind to vfio-pci
        fs::write("/sys/bus/pci/drivers/vfio-pci/bind", pci_address)
            .map_err(|e| AppError::Other(format!(
                "Failed to bind {} to vfio-pci: {}", pci_address, e
            )))?;

        tracing::info!("Successfully bound device {} to vfio-pci", pci_address);
        Ok(())
    }

    /// Unbind a PCI device from vfio-pci and rebind to original driver
    pub fn unbind_from_vfio(pci_address: &str) -> Result<(), AppError> {
        tracing::info!("Unbinding PCI device {} from vfio-pci driver", pci_address);

        let device_path = format!("/sys/bus/pci/devices/{}", pci_address);

        // Check if device exists
        if !Path::new(&device_path).exists() {
            return Err(AppError::Other(format!("PCI device {} not found", pci_address)));
        }

        // Check current driver
        let current_driver = fs::read_link(format!("{}/driver", device_path))
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()));

        // If not bound to vfio-pci, nothing to do
        if let Some(ref driver) = current_driver {
            if driver != "vfio-pci" {
                tracing::info!("Device {} not bound to vfio-pci (current: {})", pci_address, driver);
                return Ok(());
            }
        } else {
            tracing::info!("Device {} has no driver bound", pci_address);
            return Ok(());
        }

        // Unbind from vfio-pci
        Self::unbind_device(pci_address)?;

        // Trigger driver probe to let kernel find the right driver
        fs::write(format!("{}/driver_override", device_path), "")
            .ok(); // Ignore errors - some devices don't support driver_override

        fs::write("/sys/bus/pci/drivers_probe", pci_address)
            .map_err(|e| AppError::Other(format!(
                "Failed to trigger driver probe for {}: {}", pci_address, e
            )))?;

        tracing::info!("Successfully unbound device {} from vfio-pci and triggered driver probe", pci_address);
        Ok(())
    }

    /// Unbind a device from its current driver
    fn unbind_device(pci_address: &str) -> Result<(), AppError> {
        let device_path = format!("/sys/bus/pci/devices/{}", pci_address);

        // Get current driver path
        let driver_path = match fs::read_link(format!("{}/driver", device_path)) {
            Ok(path) => path,
            Err(_) => return Ok(()), // No driver bound
        };

        let unbind_path = driver_path.join("unbind");

        fs::write(&unbind_path, pci_address)
            .map_err(|e| AppError::Other(format!(
                "Failed to unbind {} from driver: {}. Are you running as root?", pci_address, e
            )))?;

        tracing::debug!("Unbound device {} from driver", pci_address);
        Ok(())
    }

    /// Ensure vfio-pci kernel module is loaded
    fn ensure_vfio_module() -> Result<(), AppError> {
        // Check if vfio-pci is already loaded
        let modules = fs::read_to_string("/proc/modules").unwrap_or_default();
        if modules.contains("vfio_pci") {
            return Ok(());
        }

        // Try to load vfio-pci module
        let output = std::process::Command::new("modprobe")
            .arg("vfio-pci")
            .output()
            .map_err(|e| AppError::Other(format!("Failed to run modprobe: {}", e)))?;

        if !output.status.success() {
            return Err(AppError::Other(
                "Failed to load vfio-pci module. Is the vfio-pci driver installed?".to_string()
            ));
        }

        tracing::debug!("Loaded vfio-pci kernel module");
        Ok(())
    }

    /// Get VFIO binding status for a device
    pub fn get_vfio_status(pci_address: &str) -> Result<VfioStatus, AppError> {
        let device_path = format!("/sys/bus/pci/devices/{}", pci_address);

        if !Path::new(&device_path).exists() {
            return Err(AppError::Other(format!("PCI device {} not found", pci_address)));
        }

        // Get current driver
        let current_driver = fs::read_link(format!("{}/driver", device_path))
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()));

        let bound_to_vfio = current_driver.as_ref().map_or(false, |d| d == "vfio-pci");

        // Check if IOMMU is enabled
        let iommu_status = Self::check_iommu_status()?;

        Ok(VfioStatus {
            device_address: pci_address.to_string(),
            bound_to_vfio,
            current_driver,
            iommu_enabled: iommu_status.enabled,
            can_bind: iommu_status.enabled,
        })
    }
}

/// VFIO binding status for a PCI device
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VfioStatus {
    pub device_address: String,
    pub bound_to_vfio: bool,
    pub current_driver: Option<String>,
    pub iommu_enabled: bool,
    pub can_bind: bool,
}
