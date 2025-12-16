use std::fs;
use std::path::Path;
use virt::domain::Domain;
use crate::models::mdev::{MdevType, MdevDevice, MdevStatus};
use crate::services::libvirt::LibvirtService;
use crate::utils::error::{AppError, map_libvirt_error};

/// MdevService provides Mediated Device (vGPU) management operations
pub struct MdevService;

impl MdevService {
    /// Check if MDEV is supported on this system
    pub fn check_mdev_status() -> MdevStatus {
        tracing::debug!("Checking MDEV status");

        let mdev_bus_path = Path::new("/sys/class/mdev_bus");
        if !mdev_bus_path.exists() {
            return MdevStatus {
                supported: false,
                message: "MDEV bus not found. Kernel may not support mediated devices.".to_string(),
                supported_vendors: vec![],
            };
        }

        let mut supported_vendors = Vec::new();

        // Check for Intel GVT-g
        if Path::new("/sys/bus/pci/drivers/i915").exists() {
            // Check if any i915 device supports MDEV
            if let Ok(entries) = fs::read_dir(mdev_bus_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Ok(driver_link) = fs::read_link(path.join("device/driver")) {
                        if driver_link.to_string_lossy().contains("i915") {
                            supported_vendors.push("Intel GVT-g".to_string());
                            break;
                        }
                    }
                }
            }
        }

        // Check for NVIDIA vGPU
        if Path::new("/sys/bus/pci/drivers/nvidia").exists() {
            if let Ok(entries) = fs::read_dir(mdev_bus_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Ok(driver_link) = fs::read_link(path.join("device/driver")) {
                        if driver_link.to_string_lossy().contains("nvidia") {
                            supported_vendors.push("NVIDIA vGPU".to_string());
                            break;
                        }
                    }
                }
            }
        }

        // Check for AMD SR-IOV
        if Path::new("/sys/bus/pci/drivers/amdgpu").exists() {
            if let Ok(entries) = fs::read_dir(mdev_bus_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Ok(driver_link) = fs::read_link(path.join("device/driver")) {
                        if driver_link.to_string_lossy().contains("amdgpu") {
                            supported_vendors.push("AMD SR-IOV".to_string());
                            break;
                        }
                    }
                }
            }
        }

        if supported_vendors.is_empty() {
            // Check if any mdev-supported devices exist at all
            if let Ok(entries) = fs::read_dir(mdev_bus_path) {
                if entries.count() > 0 {
                    return MdevStatus {
                        supported: true,
                        message: "MDEV devices detected but vendor unknown.".to_string(),
                        supported_vendors: vec!["Unknown".to_string()],
                    };
                }
            }

            return MdevStatus {
                supported: false,
                message: "No MDEV-capable devices found. Requires Intel GVT-g, NVIDIA vGPU, or AMD SR-IOV.".to_string(),
                supported_vendors: vec![],
            };
        }

        MdevStatus {
            supported: true,
            message: format!("MDEV supported with: {}", supported_vendors.join(", ")),
            supported_vendors,
        }
    }

    /// List all available MDEV types
    pub fn list_mdev_types() -> Result<Vec<MdevType>, AppError> {
        tracing::debug!("Listing MDEV types");

        let mut types = Vec::new();
        let mdev_bus_path = Path::new("/sys/class/mdev_bus");

        if !mdev_bus_path.exists() {
            return Ok(types);
        }

        let entries = fs::read_dir(mdev_bus_path)
            .map_err(|e| AppError::Other(format!("Failed to read mdev_bus: {}", e)))?;

        for entry in entries.flatten() {
            let parent_device = entry.file_name().to_string_lossy().to_string();
            let parent_path = entry.path();

            // Get parent device name
            let parent_name = Self::get_device_name(&parent_device).unwrap_or_else(|| parent_device.clone());

            // Look for mdev_supported_types
            let types_path = parent_path.join("mdev_supported_types");
            if !types_path.exists() {
                continue;
            }

            if let Ok(type_entries) = fs::read_dir(&types_path) {
                for type_entry in type_entries.flatten() {
                    let type_name = type_entry.file_name().to_string_lossy().to_string();
                    let type_path = type_entry.path();

                    // Read type properties
                    let description = fs::read_to_string(type_path.join("description"))
                        .ok()
                        .map(|s| s.trim().to_string());

                    let available_instances = fs::read_to_string(type_path.join("available_instances"))
                        .ok()
                        .and_then(|s| s.trim().parse().ok())
                        .unwrap_or(0);

                    let max_instances = fs::read_to_string(type_path.join("max_instances"))
                        .ok()
                        .and_then(|s| s.trim().parse().ok())
                        .unwrap_or(available_instances);

                    let device_api = fs::read_to_string(type_path.join("device_api"))
                        .ok()
                        .map(|s| s.trim().to_string())
                        .unwrap_or_else(|| "vfio-pci".to_string());

                    types.push(MdevType {
                        name: type_name,
                        description,
                        available_instances,
                        max_instances,
                        device_api,
                        parent_device: parent_device.clone(),
                        parent_name: parent_name.clone(),
                    });
                }
            }
        }

        tracing::info!("Found {} MDEV types", types.len());
        Ok(types)
    }

    /// List all active MDEV instances
    pub fn list_mdev_devices(libvirt: &LibvirtService) -> Result<Vec<MdevDevice>, AppError> {
        tracing::debug!("Listing MDEV devices");

        let mut devices = Vec::new();
        let mdev_devices_path = Path::new("/sys/bus/mdev/devices");

        if !mdev_devices_path.exists() {
            return Ok(devices);
        }

        let entries = fs::read_dir(mdev_devices_path)
            .map_err(|e| AppError::Other(format!("Failed to read mdev devices: {}", e)))?;

        for entry in entries.flatten() {
            let uuid = entry.file_name().to_string_lossy().to_string();
            let device_path = entry.path();

            // Get mdev type
            let mdev_type = fs::read_link(device_path.join("mdev_type"))
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                .unwrap_or_else(|| "unknown".to_string());

            // Get parent device
            let parent_device = fs::read_link(&device_path)
                .ok()
                .and_then(|p| {
                    // The symlink points to something like ../../devices/pci.../0000:03:00.0/uuid
                    p.parent()
                        .and_then(|parent| parent.file_name())
                        .map(|n| n.to_string_lossy().to_string())
                })
                .unwrap_or_else(|| "unknown".to_string());

            // Check if this MDEV is in use by any VM
            let (in_use, used_by_vm) = Self::check_mdev_in_use(libvirt, &uuid)?;

            devices.push(MdevDevice {
                uuid,
                mdev_type,
                parent_device,
                in_use,
                used_by_vm,
            });
        }

        tracing::info!("Found {} MDEV devices", devices.len());
        Ok(devices)
    }

    /// Attach an MDEV device to a VM
    pub fn attach_mdev(
        libvirt: &LibvirtService,
        vm_id: &str,
        mdev_uuid: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Attaching MDEV {} to VM {}", mdev_uuid, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Verify the MDEV exists
        let mdev_path = format!("/sys/bus/mdev/devices/{}", mdev_uuid);
        if !Path::new(&mdev_path).exists() {
            return Err(AppError::InvalidConfig(format!(
                "MDEV device '{}' not found",
                mdev_uuid
            )));
        }

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        // Build hostdev XML for MDEV
        let mdev_xml = format!(
            r#"<hostdev mode='subsystem' type='mdev' managed='no' model='vfio-pci'>
  <source>
    <address uuid='{}'/>
  </source>
</hostdev>"#,
            mdev_uuid
        );

        let flags = if is_running {
            virt::sys::VIR_DOMAIN_AFFECT_LIVE | virt::sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            virt::sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain.attach_device_flags(&mdev_xml, flags)
            .map_err(|e| {
                if is_running {
                    AppError::InvalidConfig(
                        "Cannot hot-plug MDEV device. Try stopping the VM first.".to_string()
                    )
                } else {
                    map_libvirt_error(e)
                }
            })?;

        tracing::info!("Successfully attached MDEV {} to VM {}", mdev_uuid, vm_id);
        Ok(())
    }

    /// Detach an MDEV device from a VM
    pub fn detach_mdev(
        libvirt: &LibvirtService,
        vm_id: &str,
        mdev_uuid: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Detaching MDEV {} from VM {}", mdev_uuid, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        // Get domain XML to find the MDEV entry
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        // Find the hostdev entry for this MDEV
        let search_pattern = format!("uuid='{}'", mdev_uuid);
        if !xml.contains(&search_pattern) {
            return Err(AppError::InvalidConfig(format!(
                "MDEV '{}' not found in VM configuration",
                mdev_uuid
            )));
        }

        // Build the detach XML
        let mdev_xml = format!(
            r#"<hostdev mode='subsystem' type='mdev' managed='no' model='vfio-pci'>
  <source>
    <address uuid='{}'/>
  </source>
</hostdev>"#,
            mdev_uuid
        );

        let flags = if is_running {
            virt::sys::VIR_DOMAIN_AFFECT_LIVE | virt::sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            virt::sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain.detach_device_flags(&mdev_xml, flags)
            .map_err(|e| {
                if is_running {
                    AppError::InvalidConfig(
                        "Cannot hot-unplug MDEV device from running VM.".to_string()
                    )
                } else {
                    map_libvirt_error(e)
                }
            })?;

        tracing::info!("Successfully detached MDEV {} from VM {}", mdev_uuid, vm_id);
        Ok(())
    }

    /// Create a new MDEV instance from a supported type
    pub fn create_mdev(
        parent_device: &str,
        mdev_type: &str,
    ) -> Result<String, AppError> {
        tracing::info!("Creating MDEV of type {} on device {}", mdev_type, parent_device);

        // Generate a new UUID
        let uuid = uuid::Uuid::new_v4().to_string();

        // Path to create the MDEV
        let create_path = format!(
            "/sys/class/mdev_bus/{}/mdev_supported_types/{}/create",
            parent_device, mdev_type
        );

        if !Path::new(&create_path).exists() {
            return Err(AppError::InvalidConfig(format!(
                "MDEV type '{}' not found on device '{}'",
                mdev_type, parent_device
            )));
        }

        // Write UUID to create the MDEV
        fs::write(&create_path, &uuid)
            .map_err(|e| AppError::Other(format!(
                "Failed to create MDEV: {}. Ensure you have proper permissions (run as root or configure permissions).",
                e
            )))?;

        tracing::info!("Successfully created MDEV {} of type {}", uuid, mdev_type);
        Ok(uuid)
    }

    /// Remove an MDEV instance
    pub fn remove_mdev(mdev_uuid: &str) -> Result<(), AppError> {
        tracing::info!("Removing MDEV {}", mdev_uuid);

        let remove_path = format!("/sys/bus/mdev/devices/{}/remove", mdev_uuid);

        if !Path::new(&remove_path).exists() {
            return Err(AppError::InvalidConfig(format!(
                "MDEV device '{}' not found",
                mdev_uuid
            )));
        }

        fs::write(&remove_path, "1")
            .map_err(|e| AppError::Other(format!(
                "Failed to remove MDEV: {}. It may be in use or you lack permissions.",
                e
            )))?;

        tracing::info!("Successfully removed MDEV {}", mdev_uuid);
        Ok(())
    }

    /// Check if an MDEV is in use by any VM
    fn check_mdev_in_use(libvirt: &LibvirtService, mdev_uuid: &str) -> Result<(bool, Option<String>), AppError> {
        let conn = libvirt.get_connection();

        let flags = virt::sys::VIR_CONNECT_LIST_DOMAINS_ACTIVE | virt::sys::VIR_CONNECT_LIST_DOMAINS_INACTIVE;
        let domains = conn.list_all_domains(flags)
            .map_err(map_libvirt_error)?;

        let search_pattern = format!("uuid='{}'", mdev_uuid);

        for domain in domains {
            if let Ok(xml) = domain.get_xml_desc(0) {
                if xml.contains(&search_pattern) {
                    let vm_name = domain.get_name().unwrap_or_else(|_| "Unknown".to_string());
                    return Ok((true, Some(vm_name)));
                }
            }
        }

        Ok((false, None))
    }

    /// Get device name from PCI address
    fn get_device_name(address: &str) -> Option<String> {
        // Try to get vendor/device name from sysfs
        let vendor_path = format!("/sys/bus/pci/devices/{}/vendor", address);
        let device_path = format!("/sys/bus/pci/devices/{}/device", address);

        let vendor_id = fs::read_to_string(&vendor_path).ok()?.trim().to_string();
        let device_id = fs::read_to_string(&device_path).ok()?.trim().to_string();

        Some(format!("{} {}", vendor_id, device_id))
    }
}
