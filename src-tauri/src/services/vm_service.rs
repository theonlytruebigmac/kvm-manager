use virt::domain::Domain;
use virt::sys;
use crate::models::vm::{VM, VmState};
use crate::services::libvirt::LibvirtService;
use crate::utils::error::{AppError, map_libvirt_error};

/// VmService provides VM management operations
pub struct VmService;

impl VmService {
    /// List all VMs (active and inactive)
    pub fn list_vms(libvirt: &LibvirtService) -> Result<Vec<VM>, AppError> {
        tracing::debug!("Listing all VMs");

        let conn = libvirt.get_connection();
        let mut vms = Vec::new();

        // Get all domains (both active and inactive)
        let flags = sys::VIR_CONNECT_LIST_DOMAINS_ACTIVE | sys::VIR_CONNECT_LIST_DOMAINS_INACTIVE;
        let domains = conn.list_all_domains(flags)
            .map_err(map_libvirt_error)?;

        for domain in domains {
            match Self::domain_to_vm(&domain) {
                Ok(vm) => vms.push(vm),
                Err(e) => {
                    tracing::warn!("Failed to convert domain to VM: {}", e);
                    continue;
                }
            }
        }

        tracing::info!("Found {} VMs", vms.len());
        Ok(vms)
    }

    /// Get a single VM by ID
    pub fn get_vm(libvirt: &LibvirtService, vm_id: &str) -> Result<VM, AppError> {
        tracing::debug!("Getting VM with ID: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        Self::domain_to_vm(&domain)
    }

    /// Convert a libvirt Domain to our VM model
    fn domain_to_vm(domain: &Domain) -> Result<VM, AppError> {
        let uuid = domain.get_uuid_string()
            .map_err(map_libvirt_error)?;

        let name = domain.get_name()
            .map_err(map_libvirt_error)?;

        let state = Self::get_domain_state(domain)?;

        // Get domain info for CPU and memory
        let info = domain.get_info()
            .map_err(map_libvirt_error)?;

        let cpu_count = info.nr_virt_cpu;
        let memory_mb = info.max_mem / 1024; // Convert from KiB to MiB

        // For now, we'll use placeholder values for disk and network
        // These will be properly implemented when we parse the domain XML
        let disk_size_gb = 0;
        let network_interfaces = Vec::new();

        // Extract tags from metadata
        let tags = Self::get_vm_tags(domain).unwrap_or_default();

        Ok(VM {
            id: uuid,
            name,
            state,
            cpu_count,
            memory_mb,
            disk_size_gb,
            network_interfaces,
            tags,
        })
    }

    /// Get the state of a domain
    fn get_domain_state(domain: &Domain) -> Result<VmState, AppError> {
        let (state, _reason) = domain.get_state()
            .map_err(map_libvirt_error)?;

        let vm_state = match state {
            sys::VIR_DOMAIN_RUNNING => VmState::Running,
            sys::VIR_DOMAIN_PAUSED => VmState::Paused,
            sys::VIR_DOMAIN_SHUTDOWN | sys::VIR_DOMAIN_SHUTOFF | sys::VIR_DOMAIN_CRASHED => VmState::Stopped,
            sys::VIR_DOMAIN_PMSUSPENDED => VmState::Suspended,
            _ => VmState::Stopped,
        };

        Ok(vm_state)
    }

    /// Start a VM
    pub fn start_vm(libvirt: &LibvirtService, vm_id: &str) -> Result<(), AppError> {
        tracing::info!("Starting VM: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let (state, _) = domain.get_state()
            .map_err(map_libvirt_error)?;

        if state == sys::VIR_DOMAIN_RUNNING {
            return Err(AppError::InvalidVmState("running".to_string()));
        }

        domain.create()
            .map_err(map_libvirt_error)?;

        tracing::info!("VM started successfully: {}", vm_id);
        Ok(())
    }

    /// Stop a VM
    pub fn stop_vm(libvirt: &LibvirtService, vm_id: &str) -> Result<(), AppError> {
        tracing::info!("Stopping VM: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let (state, _) = domain.get_state()
            .map_err(map_libvirt_error)?;

        if state != sys::VIR_DOMAIN_RUNNING {
            return Err(AppError::InvalidVmState("not running".to_string()));
        }

        domain.destroy()
            .map_err(map_libvirt_error)?;

        tracing::info!("VM stopped successfully: {}", vm_id);
        Ok(())
    }

    /// Force stop a VM (immediate power off, no state check)
    pub fn force_stop_vm(libvirt: &LibvirtService, vm_id: &str) -> Result<(), AppError> {
        tracing::info!("Force stopping VM: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Force destroy without state check
        domain.destroy()
            .map_err(map_libvirt_error)?;

        tracing::info!("VM force stopped successfully: {}", vm_id);
        Ok(())
    }

    /// Pause a VM
    pub fn pause_vm(libvirt: &LibvirtService, vm_id: &str) -> Result<(), AppError> {
        tracing::info!("Pausing VM: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        domain.suspend()
            .map_err(map_libvirt_error)?;

        tracing::info!("VM paused successfully: {}", vm_id);
        Ok(())
    }

    /// Resume a paused VM
    pub fn resume_vm(libvirt: &LibvirtService, vm_id: &str) -> Result<(), AppError> {
        tracing::info!("Resuming VM: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        domain.resume()
            .map_err(map_libvirt_error)?;

        tracing::info!("VM resumed successfully: {}", vm_id);
        Ok(())
    }

    /// Reboot a VM
    pub fn reboot_vm(libvirt: &LibvirtService, vm_id: &str) -> Result<(), AppError> {
        tracing::info!("Rebooting VM: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        domain.reboot(sys::VIR_DOMAIN_REBOOT_DEFAULT)
            .map_err(map_libvirt_error)?;

        tracing::info!("VM reboot initiated: {}", vm_id);
        Ok(())
    }

    /// Delete a VM (undefine)
    pub fn delete_vm(libvirt: &LibvirtService, vm_id: &str, delete_disks: bool) -> Result<(), AppError> {
        tracing::info!("Deleting VM: {} (delete_disks: {})", vm_id, delete_disks);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Check if VM is running
        let (state, _) = domain.get_state()
            .map_err(map_libvirt_error)?;

        if state == sys::VIR_DOMAIN_RUNNING {
            return Err(AppError::InvalidVmState("Cannot delete a running VM. Stop it first.".to_string()));
        }

        // If delete_disks is true, get disk paths before undefining
        let disk_paths = if delete_disks {
            Self::get_vm_disk_paths(&domain)?
        } else {
            Vec::new()
        };

        // Undefine the domain (delete configuration)
        domain.undefine()
            .map_err(map_libvirt_error)?;

        // Delete disk files if requested
        if delete_disks {
            use virt::storage_vol::StorageVol;

            for disk_path in disk_paths {
                tracing::info!("Deleting disk: {}", disk_path);

                // Try to delete via libvirt storage API first
                match StorageVol::lookup_by_path(conn, &disk_path) {
                    Ok(vol) => {
                        if let Err(e) = vol.delete(0) {
                            tracing::warn!("Failed to delete volume via libvirt API {}: {}", disk_path, e);
                            // Fallback to direct file deletion
                            if let Err(e2) = std::fs::remove_file(&disk_path) {
                                tracing::warn!("Failed to delete disk file {}: {}", disk_path, e2);
                            }
                        } else {
                            tracing::info!("Successfully deleted volume: {}", disk_path);
                        }
                    }
                    Err(_) => {
                        // Volume not in libvirt, try direct file deletion
                        if let Err(e) = std::fs::remove_file(&disk_path) {
                            tracing::warn!("Failed to delete disk file {}: {}", disk_path, e);
                        } else {
                            tracing::info!("Successfully deleted disk file: {}", disk_path);
                        }
                    }
                }
            }
        }

        tracing::info!("VM deleted successfully: {}", vm_id);
        Ok(())
    }

    /// Extract disk paths from domain XML
    fn get_vm_disk_paths(domain: &Domain) -> Result<Vec<String>, AppError> {
        let xml = domain.get_xml_desc(0)
            .map_err(map_libvirt_error)?;

        let mut disk_paths = Vec::new();

        // Simple XML parsing to find disk source files
        for line in xml.lines() {
            if line.contains("<source file=") {
                if let Some(start) = line.find("file=\"") {
                    let start_idx = start + 6;
                    if let Some(end) = line[start_idx..].find('\"') {
                        let path = line[start_idx..start_idx + end].to_string();
                        disk_paths.push(path);
                    }
                }
            }
        }

        Ok(disk_paths)
    }

    /// Create a new VM
    pub fn create_vm(libvirt: &LibvirtService, config: crate::models::vm::VmConfig) -> Result<String, AppError> {
        use virt::storage_pool::StoragePool;

        tracing::info!("Creating VM: {} (OS: {}, Network: {})", config.name, config.os_type, config.network);

        let conn = libvirt.get_connection();

        // Get the default storage pool
        let pool = StoragePool::lookup_by_name(conn, "default")
            .map_err(|e| AppError::LibvirtError(format!("Failed to find default storage pool: {}", e)))?;

        // Define volume name with proper extension based on disk format
        let volume_name = format!("{}.{}", config.name, config.disk_format);
        let disk_size_bytes = config.disk_size_gb * 1024 * 1024 * 1024; // Convert GB to bytes

        // Create storage volume XML with proper disk format
        let volume_xml = format!(
            r#"<volume>
  <name>{}</name>
  <capacity unit='bytes'>{}</capacity>
  <target>
    <format type='{}'/>
  </target>
</volume>"#,
            volume_name, disk_size_bytes, config.disk_format
        );

        tracing::info!("Creating storage volume: {} ({}GB, format: {})",
                      volume_name, config.disk_size_gb, config.disk_format);

        // Create the volume using libvirt storage pool API
        let volume = virt::storage_vol::StorageVol::create_xml(&pool, &volume_xml, 0)
            .map_err(|e| AppError::LibvirtError(format!("Failed to create storage volume: {}", e)))?;

        let disk_path = volume.get_path()
            .map_err(|e| AppError::LibvirtError(format!("Failed to get volume path: {}", e)))?;

        tracing::info!("Storage volume created: {}", disk_path);

        // Determine OS-specific settings
        let (os_type_str, machine_type) = match config.os_type.as_str() {
            "windows" => ("hvm", "pc-q35-6.2"), // Q35 chipset for Windows
            "linux" => ("hvm", "pc"),
            _ => ("hvm", "pc"),
        };

        // Build boot device order
        // If ISO is provided, boot from CD first, then HDD
        // Otherwise, boot from HDD only
        let boot_order = if config.iso_path.is_some() {
            r#"    <boot dev='cdrom'/>
    <boot dev='hd'/>"#
        } else {
            "    <boot dev='hd'/>"
        };

        // Build disk XML for the main system disk
        let main_disk_xml = format!(
            r#"    <disk type='file' device='disk'>
      <driver name='qemu' type='{}'/>
      <source file='{}'/>
      <target dev='vda' bus='virtio'/>
    </disk>"#,
            config.disk_format, disk_path
        );

        // Build CDROM/ISO disk XML if ISO path is provided
        let cdrom_xml = if let Some(ref iso_path) = config.iso_path {
            format!(
                r#"    <disk type='file' device='cdrom'>
      <driver name='qemu' type='raw'/>
      <source file='{}'/>
      <target dev='hdc' bus='ide'/>
      <readonly/>
    </disk>"#,
                iso_path
            )
        } else {
            String::new()
        };

        // Build boot menu option
        let boot_menu = if config.boot_menu {
            "    <bootmenu enable='yes' timeout='3000'/>"
        } else {
            ""
        };

        // Generate XML configuration for the VM
        let xml = format!(
            r#"<domain type='qemu'>
  <name>{}</name>
  <memory unit='MiB'>{}</memory>
  <vcpu>{}</vcpu>
  <os>
    <type arch='x86_64' machine='{}'>{}</type>
{}
{}
  </os>
  <devices>
    <emulator>/usr/bin/qemu-system-x86_64</emulator>
{}
{}
    <interface type='network'>
      <source network='{}'/>
      <model type='virtio'/>
    </interface>
    <graphics type='vnc' port='-1' autoport='yes'/>
    <video>
      <model type='qxl'/>
    </video>
    <console type='pty'/>
  </devices>
</domain>"#,
            config.name,
            config.memory_mb,
            config.cpu_count,
            machine_type,
            os_type_str,
            boot_order,
            boot_menu,
            main_disk_xml,
            cdrom_xml,
            config.network
        );

        tracing::debug!("VM XML:\n{}", xml);

        // Define the domain (create VM configuration)
        let domain = Domain::define_xml(conn, &xml)
            .map_err(map_libvirt_error)?;

        let uuid = domain.get_uuid_string()
            .map_err(map_libvirt_error)?;

        tracing::info!("VM created successfully: {} (UUID: {})", config.name, uuid);
        Ok(uuid)
    }

    /// Get VM performance statistics
    pub fn get_vm_stats(libvirt: &LibvirtService, vm_id: &str) -> Result<crate::models::vm::VmStats, AppError> {
        use virt::sys;

        tracing::debug!("Getting stats for VM: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let info = domain.get_info()
            .map_err(map_libvirt_error)?;

        // For stopped VMs, return zeros
        let (state, _) = domain.get_state()
            .map_err(map_libvirt_error)?;

        if state != sys::VIR_DOMAIN_RUNNING {
            return Ok(crate::models::vm::VmStats {
                vm_id: vm_id.to_string(),
                cpu_usage_percent: 0.0,
                memory_used_mb: 0,
                memory_available_mb: 0,
                disk_read_bytes: 0,
                disk_write_bytes: 0,
                network_rx_bytes: 0,
                network_tx_bytes: 0,
                timestamp: chrono::Utc::now().timestamp_millis(),
            });
        }

        // Memory stats (convert from KiB to MiB)
        let memory_used_mb = info.memory / 1024;
        let memory_available_mb = info.max_mem / 1024;

        // CPU usage (libvirt provides CPU time, not percentage)
        // For simplicity, we'll use a placeholder for now
        // In a real implementation, you'd track CPU time delta over intervals
        let cpu_usage_percent = 0.0;

        // Disk and network stats (cumulative since VM start)
        // Note: These require iterating through all disks and interfaces
        let disk_read_bytes = 0;
        let disk_write_bytes = 0;
        let network_rx_bytes = 0;
        let network_tx_bytes = 0;

        Ok(crate::models::vm::VmStats {
            vm_id: vm_id.to_string(),
            cpu_usage_percent,
            memory_used_mb,
            memory_available_mb,
            disk_read_bytes,
            disk_write_bytes,
            network_rx_bytes,
            network_tx_bytes,
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    /// Clone a VM
    pub fn clone_vm(
        libvirt: &LibvirtService,
        source_vm_id: &str,
        new_name: &str,
    ) -> Result<String, AppError> {
        tracing::info!("Cloning VM {} to {}", source_vm_id, new_name);

        let conn = libvirt.get_connection();
        let source_domain = Domain::lookup_by_uuid_string(conn, source_vm_id)
            .map_err(|_| AppError::VmNotFound(source_vm_id.to_string()))?;

        // Get source VM XML
        let source_xml = source_domain.get_xml_desc(0)
            .map_err(map_libvirt_error)?;

        // Parse and modify XML to create clone
        let clone_xml = Self::modify_xml_for_clone(&source_xml, new_name)?;

        // Define the cloned domain
        let clone_domain = Domain::define_xml(conn, &clone_xml)
            .map_err(map_libvirt_error)?;

        let clone_uuid = clone_domain.get_uuid_string()
            .map_err(map_libvirt_error)?;

        tracing::info!("VM cloned successfully: {} (UUID: {})", new_name, clone_uuid);
        Ok(clone_uuid)
    }

    /// Modify VM XML for cloning
    fn modify_xml_for_clone(xml: &str, new_name: &str) -> Result<String, AppError> {
        let mut modified = xml.to_string();

        // Remove UUID to let libvirt generate a new one
        if let Some(uuid_start) = modified.find("<uuid>") {
            if let Some(uuid_end) = modified[uuid_start..].find("</uuid>") {
                let full_end = uuid_start + uuid_end + 7;
                modified.replace_range(uuid_start..full_end, "");
            }
        }

        // Change name
        if let Some(name_start) = modified.find("<name>") {
            if let Some(name_end) = modified[name_start..].find("</name>") {
                let content_start = name_start + 6;
                let content_end = name_start + name_end;
                modified.replace_range(content_start..content_end, new_name);
            }
        }

        // Generate new MAC addresses for network interfaces
        modified = Self::regenerate_mac_addresses(&modified);

        Ok(modified)
    }

    /// Regenerate MAC addresses in XML
    fn regenerate_mac_addresses(xml: &str) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut result = xml.to_string();

        while let Some(mac_start) = result.find("<mac address='") {
            let mac_content_start = mac_start + 14;
            if let Some(mac_end) = result[mac_content_start..].find("'/>") {
                let mac_content_end = mac_content_start + mac_end;

                // Generate new MAC address (52:54:00 prefix for QEMU)
                let new_mac = format!(
                    "52:54:00:{:02x}:{:02x}:{:02x}",
                    rng.gen::<u8>(),
                    rng.gen::<u8>(),
                    rng.gen::<u8>()
                );

                result.replace_range(mac_content_start..mac_content_end, &new_mac);
            } else {
                break;
            }
        }

        result
    }

    /// Get tags for a VM from its metadata
    fn get_vm_tags(domain: &Domain) -> Result<Vec<String>, AppError> {
        let xml = domain.get_xml_desc(0)
            .map_err(map_libvirt_error)?;

        // Look for our custom metadata tags
        let mut tags = Vec::new();

        if let Some(metadata_start) = xml.find("<metadata>") {
            if let Some(metadata_end) = xml[metadata_start..].find("</metadata>") {
                let metadata_section = &xml[metadata_start..metadata_start + metadata_end + 11];

                // Extract tags from <kvm:tag> elements
                let mut search_pos = 0;
                while let Some(tag_start) = metadata_section[search_pos..].find("<kvm:tag>") {
                    let abs_tag_start = search_pos + tag_start + 9;
                    if let Some(tag_end) = metadata_section[abs_tag_start..].find("</kvm:tag>") {
                        let tag = metadata_section[abs_tag_start..abs_tag_start + tag_end].to_string();
                        tags.push(tag);
                        search_pos = abs_tag_start + tag_end + 10;
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(tags)
    }

    /// Add tags to a VM
    pub fn add_vm_tags(libvirt: &LibvirtService, vm_id: &str, tags: Vec<String>) -> Result<(), AppError> {
        tracing::info!("Adding tags to VM {}: {:?}", vm_id, tags);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Get current XML
        let xml = domain.get_xml_desc(0)
            .map_err(map_libvirt_error)?;

        // Get existing tags
        let mut existing_tags = Self::get_vm_tags(&domain)?;

        // Add new tags (avoiding duplicates)
        for tag in tags {
            if !existing_tags.contains(&tag) {
                existing_tags.push(tag);
            }
        }

        // Build metadata XML
        let tags_xml = existing_tags.iter()
            .map(|t| format!("    <kvm:tag>{}</kvm:tag>", t))
            .collect::<Vec<_>>()
            .join("\n");

        let metadata_xml = format!(
            r#"  <metadata>
  <kvm:tags xmlns:kvm="http://kvm-manager.local/metadata">
{}
  </kvm:tags>
</metadata>"#,
            tags_xml
        );

        // Replace or add metadata section
        let new_xml = if xml.contains("<metadata>") {
            // Replace existing metadata
            let start = xml.find("<metadata>").unwrap();
            let end = xml.find("</metadata>").unwrap() + 11;
            format!("{}{}{}", &xml[..start], metadata_xml, &xml[end..])
        } else {
            // Add metadata before </domain>
            xml.replace("</domain>", &format!("{}\n</domain>", metadata_xml))
        };

        // Redefine domain with new XML
        let _new_domain = Domain::define_xml(conn, &new_xml)
            .map_err(map_libvirt_error)?;

        tracing::info!("Tags added successfully to VM {}", vm_id);
        Ok(())
    }

    /// Remove tags from a VM
    pub fn remove_vm_tags(libvirt: &LibvirtService, vm_id: &str, tags_to_remove: Vec<String>) -> Result<(), AppError> {
        tracing::info!("Removing tags from VM {}: {:?}", vm_id, tags_to_remove);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Get current XML
        let xml = domain.get_xml_desc(0)
            .map_err(map_libvirt_error)?;

        // Get existing tags
        let mut existing_tags = Self::get_vm_tags(&domain)?;

        // Remove specified tags
        existing_tags.retain(|t| !tags_to_remove.contains(t));

        // Build new metadata XML
        if existing_tags.is_empty() {
            // Remove metadata section entirely if no tags left
            let new_xml = if let Some(start) = xml.find("<metadata>") {
                if let Some(end) = xml.find("</metadata>") {
                    format!("{}{}", &xml[..start], &xml[end + 11..])
                } else {
                    xml
                }
            } else {
                xml
            };

            let _new_domain = Domain::define_xml(conn, &new_xml)
                .map_err(map_libvirt_error)?;
        } else {
            // Update with remaining tags
            Self::add_vm_tags(libvirt, vm_id, existing_tags)?;
        }

        tracing::info!("Tags removed successfully from VM {}", vm_id);
        Ok(())
    }

    /// Export a VM's configuration to XML
    pub fn export_vm(libvirt: &LibvirtService, vm_id: &str) -> Result<String, AppError> {
        tracing::info!("Exporting VM configuration for {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Get XML description with secure flag to include sensitive data
        let xml = domain.get_xml_desc(sys::VIR_DOMAIN_XML_SECURE)
            .map_err(map_libvirt_error)?;

        tracing::info!("Successfully exported VM configuration for {}", vm_id);
        Ok(xml)
    }

    /// Import a VM from XML configuration
    pub fn import_vm(libvirt: &LibvirtService, xml: &str) -> Result<String, AppError> {
        tracing::info!("Importing VM from XML configuration");

        let conn = libvirt.get_connection();

        // Define domain from XML
        let domain = Domain::define_xml(conn, xml)
            .map_err(map_libvirt_error)?;

        // Get UUID of newly created domain
        let uuid = domain.get_uuid_string()
            .map_err(map_libvirt_error)?;

        tracing::info!("Successfully imported VM with UUID {}", uuid);
        Ok(uuid)
    }

    /// Attach a disk to a VM
    pub fn attach_disk(
        libvirt: &LibvirtService,
        vm_id: &str,
        disk_path: &str,
        device_target: &str,
        bus_type: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Attaching disk {} to VM {} as {}", disk_path, vm_id, device_target);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Validate bus type
        let valid_bus_types = ["virtio", "scsi", "sata", "ide"];
        if !valid_bus_types.contains(&bus_type) {
            return Err(AppError::InvalidConfig(format!(
                "Invalid bus type '{}'. Must be one of: {:?}",
                bus_type, valid_bus_types
            )));
        }

        // Build disk XML
        let disk_xml = format!(
            r#"<disk type='file' device='disk'>
  <driver name='qemu' type='qcow2'/>
  <source file='{}'/>
  <target dev='{}' bus='{}'/>
</disk>"#,
            disk_path, device_target, bus_type
        );

        // Attach disk (persistent and live if VM is running)
        let flags = if domain.is_active().map_err(map_libvirt_error)? {
            sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain.attach_device_flags(&disk_xml, flags)
            .map_err(map_libvirt_error)?;

        tracing::info!("Successfully attached disk {} to VM {}", disk_path, vm_id);
        Ok(())
    }

    /// Detach a disk from a VM
    pub fn detach_disk(
        libvirt: &LibvirtService,
        vm_id: &str,
        device_target: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Detaching disk {} from VM {}", device_target, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Get current XML to find the disk
        let xml = domain.get_xml_desc(0)
            .map_err(map_libvirt_error)?;

        // Find disk device XML by target
        let disk_start_pattern = format!("<disk");
        let target_pattern = format!("target dev='{}'", device_target);

        let mut search_pos = 0;
        let mut disk_xml = None;

        while let Some(disk_pos) = xml[search_pos..].find(&disk_start_pattern) {
            let abs_disk_pos = search_pos + disk_pos;
            if let Some(disk_end) = xml[abs_disk_pos..].find("</disk>") {
                let disk_section = &xml[abs_disk_pos..abs_disk_pos + disk_end + 7];

                // Check if this disk has the target we're looking for
                if disk_section.contains(&target_pattern) {
                    disk_xml = Some(disk_section.to_string());
                    break;
                }

                search_pos = abs_disk_pos + disk_end + 7;
            } else {
                break;
            }
        }

        let disk_xml = disk_xml.ok_or_else(|| {
            AppError::InvalidConfig(format!("Disk with target '{}' not found", device_target))
        })?;

        // Detach disk (persistent and live if VM is running)
        let flags = if domain.is_active().map_err(map_libvirt_error)? {
            sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain.detach_device_flags(&disk_xml, flags)
            .map_err(map_libvirt_error)?;

        tracing::info!("Successfully detached disk {} from VM {}", device_target, vm_id);
        Ok(())
    }
}
