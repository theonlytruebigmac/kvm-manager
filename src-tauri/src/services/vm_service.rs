use crate::models::operation::OperationContext;
use crate::models::vm::{HostNetworkInterface, VmState, VM};
use crate::services::cloud_init_service::CloudInitService;
use crate::services::host_readiness_service::HostReadinessService;
use crate::services::libvirt::ConnectionProvider;
use crate::utils::error::{map_libvirt_error, AppError};
use crate::utils::xml::{
    descendant_attribute_values, escaped_attribute, escaped_text, first_element_attribute,
    first_element_fragment, first_element_text, first_element_with_descendant_attribute,
    insert_cpu_after_vcpu, remove_element_with_descendant_attribute, remove_first_element,
    replace_direct_child, rewrite_cpu_model, rewrite_cpu_topology, rewrite_disk_settings,
    rewrite_domain_boot_order, rewrite_domain_memory, rewrite_first_text_element,
    rewrite_interface_bandwidth, rewrite_interface_link_state,
};
use virt::domain::Domain;
use virt::sys;

/// VmService provides VM management operations
pub struct VmService;

const LIBVIRT_TPM_UNDEFINE_MIN_VERSION: u32 = 8_009_000;

fn deletion_undefine_flags(libvirt_version: u32) -> sys::virDomainUndefineFlagsValues {
    let mut flags = sys::VIR_DOMAIN_UNDEFINE_MANAGED_SAVE | sys::VIR_DOMAIN_UNDEFINE_NVRAM;

    // VIR_DOMAIN_UNDEFINE_TPM was added in libvirt 8.9.0. Passing it to an older
    // daemon makes the entire undefine operation fail with an unsupported flag.
    if libvirt_version >= LIBVIRT_TPM_UNDEFINE_MIN_VERSION {
        flags |= sys::VIR_DOMAIN_UNDEFINE_TPM;
    }

    flags
}

impl VmService {
    /// Build a virsh invocation bound to the same libvirt URI as the captured operation.
    fn virsh_command(conn: &virt::connect::Connect) -> Result<std::process::Command, AppError> {
        let uri = conn.get_uri().map_err(map_libvirt_error)?;
        let mut command = std::process::Command::new("virsh");
        command.args(["--connect", uri.as_str()]);
        Ok(command)
    }

    fn virsh_command_for_domain(domain: &Domain) -> Result<std::process::Command, AppError> {
        let conn = domain.get_connect().map_err(map_libvirt_error)?;
        Self::virsh_command(&conn)
    }

    fn validate_vm_name(name: &str) -> Result<(), AppError> {
        if name.is_empty()
            || name.len() > 128
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        {
            return Err(AppError::InvalidConfig(
                "VM name may only contain letters, numbers, dots, underscores, and hyphens"
                    .to_string(),
            ));
        }
        Ok(())
    }

    fn choice<'a>(value: &'a str, field: &str, allowed: &[&str]) -> Result<&'a str, AppError> {
        if allowed.contains(&value) {
            Ok(value)
        } else {
            Err(AppError::InvalidConfig(format!("{field} is not supported")))
        }
    }

    fn parse_pci_address(address: &str) -> Result<(u16, u8, u8, u8), AppError> {
        let (domain, remainder) = address.split_once(':').ok_or_else(|| {
            AppError::InvalidConfig(
                "PCI address must use domain:bus:slot.function format".to_string(),
            )
        })?;
        let (bus, remainder) = remainder.split_once(':').ok_or_else(|| {
            AppError::InvalidConfig(
                "PCI address must use domain:bus:slot.function format".to_string(),
            )
        })?;
        let (slot, function) = remainder.split_once('.').ok_or_else(|| {
            AppError::InvalidConfig(
                "PCI address must use domain:bus:slot.function format".to_string(),
            )
        })?;
        if function.contains('.')
            || domain.len() != 4
            || bus.len() != 2
            || slot.len() != 2
            || function.len() != 1
        {
            return Err(AppError::InvalidConfig(
                "PCI address must use domain:bus:slot.function format".to_string(),
            ));
        }
        let parsed = (
            u16::from_str_radix(domain, 16).map_err(|_| {
                AppError::InvalidConfig("PCI address contains non-hexadecimal data".to_string())
            })?,
            u8::from_str_radix(bus, 16).map_err(|_| {
                AppError::InvalidConfig("PCI address contains non-hexadecimal data".to_string())
            })?,
            u8::from_str_radix(slot, 16).map_err(|_| {
                AppError::InvalidConfig("PCI address contains non-hexadecimal data".to_string())
            })?,
            u8::from_str_radix(function, 16).map_err(|_| {
                AppError::InvalidConfig("PCI address contains non-hexadecimal data".to_string())
            })?,
        );
        if parsed.3 > 7 {
            return Err(AppError::InvalidConfig(
                "PCI function must be between 0 and 7".to_string(),
            ));
        }
        Ok(parsed)
    }

    /// List all VMs (active and inactive)
    pub fn list_vms(libvirt: &impl ConnectionProvider) -> Result<Vec<VM>, AppError> {
        tracing::debug!("Listing all VMs");

        let conn = libvirt.get_connection();
        let mut vms = Vec::new();

        // Get all domains (both active and inactive)
        let flags = sys::VIR_CONNECT_LIST_DOMAINS_ACTIVE | sys::VIR_CONNECT_LIST_DOMAINS_INACTIVE;
        let domains = conn.list_all_domains(flags).map_err(map_libvirt_error)?;

        for domain in domains {
            match Self::domain_to_vm(&domain) {
                Ok(vm) => vms.push(vm),
                Err(_) => {
                    tracing::warn!("A VM could not be read");
                    continue;
                }
            }
        }

        // Inventory reads are expected to run periodically while a main window is
        // open. Keep the result available for diagnostics without flooding normal
        // application logs.
        tracing::debug!("Found {} VMs", vms.len());
        Ok(vms)
    }

    /// Get a single VM by ID
    pub fn get_vm(libvirt: &impl ConnectionProvider, vm_id: &str) -> Result<VM, AppError> {
        tracing::debug!("Getting VM with ID: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        Self::domain_to_vm(&domain)
    }

    /// Convert a libvirt Domain to our VM model
    fn domain_to_vm(domain: &Domain) -> Result<VM, AppError> {
        let uuid = domain.get_uuid_string().map_err(map_libvirt_error)?;

        let name = domain.get_name().map_err(map_libvirt_error)?;

        let state = Self::get_domain_state(domain)?;

        // Get domain info for CPU and memory
        let info = domain.get_info().map_err(map_libvirt_error)?;

        let cpu_count = info.nr_virt_cpu;
        let max_memory_mb = info.max_mem / 1024; // Convert from KiB to MiB
        let memory_mb = info.memory / 1024; // Current memory in MiB

        // Extract network interfaces from XML
        let network_interfaces = Self::get_vm_network_interfaces(domain).unwrap_or_default();

        // Extract disk information from XML
        let disks = Self::get_vm_disks(domain).unwrap_or_default();

        // Calculate total disk size using virsh (works with proper permissions)
        let disk_size_gb = Self::calculate_disk_size_from_domain(domain);

        // Extract tags from metadata
        let tags = Self::get_vm_tags(domain).unwrap_or_default();

        // Extract firmware, TPM, and chipset from XML
        let (firmware, tpm_enabled, chipset) = Self::get_vm_hardware_config(domain)
            .unwrap_or_else(|_| ("bios".to_string(), false, "pc".to_string()));

        // Extract CPU topology from XML
        let (cpu_sockets, cpu_cores, cpu_threads) = Self::get_cpu_topology(domain).unwrap_or({
            // If no topology found, default to 1 socket, N cores, 1 thread
            (1, cpu_count, 1)
        });

        Ok(VM {
            id: uuid,
            name,
            state,
            cpu_count,
            memory_mb,
            max_memory_mb,
            disk_size_gb,
            network_interfaces,
            disks,
            tags,
            firmware,
            tpm_enabled,
            chipset,
            cpu_sockets,
            cpu_cores,
            cpu_threads,
            cloud_init: None,
        })
    }

    /// Get the state of a domain
    fn get_domain_state(domain: &Domain) -> Result<VmState, AppError> {
        let (state, _reason) = domain.get_state().map_err(map_libvirt_error)?;

        let vm_state = match state {
            sys::VIR_DOMAIN_RUNNING => VmState::Running,
            sys::VIR_DOMAIN_PAUSED => VmState::Paused,
            sys::VIR_DOMAIN_SHUTDOWN | sys::VIR_DOMAIN_SHUTOFF | sys::VIR_DOMAIN_CRASHED => {
                VmState::Stopped
            }
            sys::VIR_DOMAIN_PMSUSPENDED => VmState::Suspended,
            _ => VmState::Stopped,
        };

        Ok(vm_state)
    }

    /// Start a VM
    pub fn start_vm(libvirt: &impl ConnectionProvider, vm_id: &str) -> Result<(), AppError> {
        tracing::info!("Starting VM: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let (state, _) = domain.get_state().map_err(map_libvirt_error)?;

        if state == sys::VIR_DOMAIN_RUNNING {
            return Err(AppError::InvalidVmState("running".to_string()));
        }

        domain.create().map_err(map_libvirt_error)?;

        tracing::info!("VM started successfully: {}", vm_id);
        Ok(())
    }

    /// Stop a VM
    pub fn stop_vm(libvirt: &impl ConnectionProvider, vm_id: &str) -> Result<(), AppError> {
        tracing::info!("Stopping VM: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let (state, _) = domain.get_state().map_err(map_libvirt_error)?;

        if state != sys::VIR_DOMAIN_RUNNING {
            return Err(AppError::InvalidVmState("not running".to_string()));
        }

        domain.destroy().map_err(map_libvirt_error)?;

        tracing::info!("VM stopped successfully: {}", vm_id);
        Ok(())
    }

    /// Force stop a VM (immediate power off, no state check)
    pub fn force_stop_vm(libvirt: &impl ConnectionProvider, vm_id: &str) -> Result<(), AppError> {
        tracing::info!("Force stopping VM: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Force destroy without state check
        domain.destroy().map_err(map_libvirt_error)?;

        tracing::info!("VM force stopped successfully: {}", vm_id);
        Ok(())
    }

    /// Pause a VM
    pub fn pause_vm(libvirt: &impl ConnectionProvider, vm_id: &str) -> Result<(), AppError> {
        tracing::info!("Pausing VM: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        domain.suspend().map_err(map_libvirt_error)?;

        tracing::info!("VM paused successfully: {}", vm_id);
        Ok(())
    }

    /// Resume a paused VM
    pub fn resume_vm(libvirt: &impl ConnectionProvider, vm_id: &str) -> Result<(), AppError> {
        tracing::info!("Resuming VM: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        domain.resume().map_err(map_libvirt_error)?;

        tracing::info!("VM resumed successfully: {}", vm_id);
        Ok(())
    }

    /// Hibernate (managed save) a VM - saves state to disk and stops
    /// The VM will resume from this state when started again
    pub fn hibernate_vm(libvirt: &impl ConnectionProvider, vm_id: &str) -> Result<(), AppError> {
        tracing::info!("Hibernating VM: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Check if VM is running
        let is_active = domain.is_active().map_err(map_libvirt_error)?;

        if !is_active {
            return Err(AppError::InvalidVmState(
                "stopped - must be running to hibernate".to_string(),
            ));
        }

        // Perform managed save (hibernate)
        domain
            .managed_save(0)
            .map_err(|e| AppError::LibvirtError(format!("Failed to hibernate VM: {}", e)))?;

        tracing::info!("VM hibernated successfully: {}", vm_id);
        Ok(())
    }

    /// Check if VM has a managed save image (hibernated state)
    pub fn has_managed_save(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
    ) -> Result<bool, AppError> {
        tracing::debug!("Checking managed save for VM: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let has_save = domain.has_managed_save(0).map_err(map_libvirt_error)?;

        Ok(has_save)
    }

    /// Remove managed save image (discard hibernated state)
    pub fn remove_managed_save(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Removing managed save for VM: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Check if there's a managed save image
        let has_save = domain.has_managed_save(0).map_err(map_libvirt_error)?;

        if !has_save {
            return Err(AppError::InvalidVmState(
                "no hibernated state to remove".to_string(),
            ));
        }

        domain
            .managed_save_remove(0)
            .map_err(|e| AppError::LibvirtError(format!("Failed to remove managed save: {}", e)))?;

        tracing::info!("Managed save removed successfully: {}", vm_id);
        Ok(())
    }

    /// Reboot a VM
    pub fn reboot_vm(libvirt: &impl ConnectionProvider, vm_id: &str) -> Result<(), AppError> {
        tracing::info!("Rebooting VM: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        domain
            .reboot(sys::VIR_DOMAIN_REBOOT_DEFAULT)
            .map_err(map_libvirt_error)?;

        tracing::info!("VM reboot initiated: {}", vm_id);
        Ok(())
    }

    /// Delete a VM (undefine)
    pub fn delete_vm(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        delete_disks: bool,
    ) -> Result<(), AppError> {
        tracing::info!("Deleting VM: {} (delete_disks: {})", vm_id, delete_disks);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Check if VM is running
        let (state, _) = domain.get_state().map_err(map_libvirt_error)?;

        if state == sys::VIR_DOMAIN_RUNNING {
            return Err(AppError::InvalidVmState(
                "Cannot delete a running VM. Stop it first.".to_string(),
            ));
        }

        // If delete_disks is true, get disk paths before undefining
        let disk_paths = if delete_disks {
            Self::get_vm_disk_paths(&domain)?
        } else {
            Vec::new()
        };

        // UEFI and TPM-backed VMs own persistent firmware state in addition to
        // their domain XML. A plain undefine is rejected while that state exists.
        // Include managed-save state as part of deleting the VM, but deliberately
        // leave snapshot metadata to the caller's explicit delete_snapshots choice.
        let libvirt_version = conn.get_lib_version().unwrap_or_else(|_| {
            tracing::warn!(
                "Could not determine the libvirt version; using legacy-compatible VM deletion flags"
            );
            0
        });
        domain
            .undefine_flags(deletion_undefine_flags(libvirt_version))
            .map_err(map_libvirt_error)?;

        // Delete disk files if requested
        if delete_disks {
            use virt::storage_vol::StorageVol;

            for disk_path in disk_paths {
                tracing::info!("Attempting to delete a VM disk");

                // Try to delete via libvirt storage API
                match StorageVol::lookup_by_path(conn, &disk_path) {
                    Ok(vol) => {
                        // Use VIR_STORAGE_VOL_DELETE_NORMAL flag (0)
                        match vol.delete(0) {
                            Ok(_) => {
                                tracing::info!("Successfully deleted a VM disk");
                            }
                            Err(_) => {
                                tracing::warn!(
                                    "Libvirt could not delete a VM-managed storage volume"
                                );
                                return Err(AppError::LibvirtError(
                                    "A VM-managed storage volume could not be deleted. Review the operation diagnostics and inspect storage through libvirt.".to_string(),
                                ));
                            }
                        }
                    }
                    Err(_) => {
                        tracing::warn!(
                            "A VM-managed disk could not be resolved through libvirt storage"
                        );
                        return Err(AppError::LibvirtError(
                            "A VM-managed storage volume could not be resolved through libvirt. Review the operation diagnostics and inspect storage through libvirt.".to_string(),
                        ));
                    }
                }
            }
        }

        tracing::info!("VM deleted successfully: {}", vm_id);
        Ok(())
    }

    /// Extract disk paths from domain XML
    fn get_vm_disk_paths(domain: &Domain) -> Result<Vec<String>, AppError> {
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;
        Ok(Self::extract_disk_paths(&xml))
    }

    /// Get network interfaces from a VM
    fn get_vm_network_interfaces(
        domain: &Domain,
    ) -> Result<Vec<crate::models::vm::NetworkInterface>, AppError> {
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        let vm_name = domain.get_name().map_err(map_libvirt_error)?;

        // Get IP addresses from virsh domifaddr
        let ip_map = Self::get_interface_ips(domain, &vm_name);

        let mut interfaces = Vec::new();
        let mut in_interface = false;
        let mut in_bandwidth = false;
        let mut current_mac = String::new();
        let mut current_network = String::new();
        let mut current_target = String::new();
        let mut current_model = String::new();
        // QoS settings
        let mut inbound_average: Option<u64> = None;
        let mut inbound_peak: Option<u64> = None;
        let mut inbound_burst: Option<u64> = None;
        let mut outbound_average: Option<u64> = None;
        let mut outbound_peak: Option<u64> = None;
        let mut outbound_burst: Option<u64> = None;

        for line in xml.lines() {
            let trimmed = line.trim();

            // Start of interface element (type='network' or type='bridge')
            if trimmed.starts_with("<interface type=") {
                in_interface = true;
                in_bandwidth = false;
                current_mac = String::new();
                current_network = String::new();
                current_target = String::new();
                current_model = String::new();
                inbound_average = None;
                inbound_peak = None;
                inbound_burst = None;
                outbound_average = None;
                outbound_peak = None;
                outbound_burst = None;
            }

            // Extract MAC address (handle both single and double quotes)
            if in_interface && trimmed.contains("<mac address=") {
                let mac = Self::extract_xml_attribute(trimmed, "address");
                if let Some(m) = mac {
                    current_mac = m;
                }
            }

            // Extract network name from source (network='...' or bridge='...')
            if in_interface && trimmed.contains("<source") {
                // Try network first
                if let Some(net) = Self::extract_xml_attribute(trimmed, "network") {
                    current_network = net;
                } else if let Some(br) = Self::extract_xml_attribute(trimmed, "bridge") {
                    current_network = format!("bridge:{}", br);
                }
            }

            // Extract target device name (e.g., vnet0)
            if in_interface && trimmed.contains("<target dev=") {
                if let Some(dev) = Self::extract_xml_attribute(trimmed, "dev") {
                    current_target = dev;
                }
            }

            // Extract model type (e.g., virtio, e1000)
            if in_interface && trimmed.contains("<model type=") {
                if let Some(model) = Self::extract_xml_attribute(trimmed, "type") {
                    current_model = model;
                }
            }

            // Bandwidth/QoS section
            if in_interface && trimmed.starts_with("<bandwidth") {
                in_bandwidth = true;
            }

            if in_bandwidth && trimmed.starts_with("<inbound") {
                if let Some(avg) = Self::extract_xml_attribute(trimmed, "average") {
                    inbound_average = avg.parse().ok();
                }
                if let Some(peak) = Self::extract_xml_attribute(trimmed, "peak") {
                    inbound_peak = peak.parse().ok();
                }
                if let Some(burst) = Self::extract_xml_attribute(trimmed, "burst") {
                    inbound_burst = burst.parse().ok();
                }
            }

            if in_bandwidth && trimmed.starts_with("<outbound") {
                if let Some(avg) = Self::extract_xml_attribute(trimmed, "average") {
                    outbound_average = avg.parse().ok();
                }
                if let Some(peak) = Self::extract_xml_attribute(trimmed, "peak") {
                    outbound_peak = peak.parse().ok();
                }
                if let Some(burst) = Self::extract_xml_attribute(trimmed, "burst") {
                    outbound_burst = burst.parse().ok();
                }
            }

            if in_bandwidth && trimmed.starts_with("</bandwidth") {
                in_bandwidth = false;
            }

            // End of interface - save it
            if trimmed == "</interface>" && in_interface {
                if !current_mac.is_empty() {
                    // Look up IP address by MAC address
                    let ip_address = ip_map.get(&current_mac.to_lowercase()).cloned();

                    interfaces.push(crate::models::vm::NetworkInterface {
                        name: if current_target.is_empty() {
                            format!("nic{}", interfaces.len() + 1)
                        } else {
                            current_target.clone()
                        },
                        mac_address: current_mac.clone(),
                        mac: current_mac.clone(), // Alias for frontend compatibility
                        network: if current_network.is_empty() {
                            "unknown".to_string()
                        } else {
                            current_network.clone()
                        },
                        ip_address,
                        model_type: if current_model.is_empty() {
                            None
                        } else {
                            Some(current_model.clone())
                        },
                        inbound_average,
                        inbound_peak,
                        inbound_burst,
                        outbound_average,
                        outbound_peak,
                        outbound_burst,
                    });
                }
                in_interface = false;
                in_bandwidth = false;
            }
        }

        Ok(interfaces)
    }

    /// Get IP addresses for network interfaces using virsh domifaddr
    fn get_interface_ips(
        domain: &Domain,
        vm_name: &str,
    ) -> std::collections::HashMap<String, String> {
        let mut ip_map = std::collections::HashMap::new();

        // Try virsh domifaddr (uses QEMU guest agent or lease info)
        let output = Self::virsh_command_for_domain(domain)
            .ok()
            .and_then(|mut command| command.args(["domifaddr", vm_name]).output().ok());

        if let Some(out) = output {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                // Parse output format:
                //  Name       MAC address          Protocol     Address
                // -------------------------------------------------------------------------------
                //  vnet0      52:54:00:03:c9:52    ipv4         192.168.122.35/24
                for line in text.lines().skip(2) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        let mac = parts[1].to_lowercase();
                        let addr = parts[3].to_string();
                        // Remove CIDR notation if present
                        let ip = addr.split('/').next().unwrap_or(&addr).to_string();
                        ip_map.insert(mac, ip);
                    }
                }
            }
        }

        ip_map
    }

    /// Helper to extract an XML attribute value (handles both single and double quotes)
    fn extract_xml_attribute(line: &str, attr_name: &str) -> Option<String> {
        // Try single quotes first
        let single_pattern = format!("{}='", attr_name);
        if let Some(start) = line.find(&single_pattern) {
            let start_idx = start + single_pattern.len();
            if let Some(end) = line[start_idx..].find('\'') {
                return Some(line[start_idx..start_idx + end].to_string());
            }
        }

        // Try double quotes
        let double_pattern = format!("{}=\"", attr_name);
        if let Some(start) = line.find(&double_pattern) {
            let start_idx = start + double_pattern.len();
            if let Some(end) = line[start_idx..].find('\"') {
                return Some(line[start_idx..start_idx + end].to_string());
            }
        }

        None
    }

    /// Get disk devices from a VM
    fn get_vm_disks(domain: &Domain) -> Result<Vec<crate::models::vm::DiskDevice>, AppError> {
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        let mut disks = Vec::new();

        // Helper to extract attribute value from a line
        let extract_attr = |line: &str, attr: &str| -> Option<String> {
            // Try single quotes first
            let pattern1 = format!("{}='", attr);
            let pattern2 = format!("{}=\"", attr);

            if let Some(start) = line.find(&pattern1) {
                let start_idx = start + pattern1.len();
                if let Some(end) = line[start_idx..].find('\'') {
                    return Some(line[start_idx..start_idx + end].to_string());
                }
            }
            if let Some(start) = line.find(&pattern2) {
                let start_idx = start + pattern2.len();
                if let Some(end) = line[start_idx..].find('\"') {
                    return Some(line[start_idx..start_idx + end].to_string());
                }
            }
            None
        };

        // Track current disk being parsed
        struct DiskBuilder {
            device: String,
            path: String,
            disk_type: String,
            bus: String,
            cache: Option<String>,
            io: Option<String>,
            discard: Option<String>,
            detect_zeroes: Option<String>,
            read_iops_sec: Option<u64>,
            write_iops_sec: Option<u64>,
            read_bytes_sec: Option<u64>,
            write_bytes_sec: Option<u64>,
        }

        let mut current_disk: Option<DiskBuilder> = None;
        let mut in_iotune = false;

        for line in xml.lines() {
            let trimmed = line.trim();

            // Start of a disk device (only device='disk', skip cdrom)
            if trimmed.starts_with("<disk type=") && trimmed.contains("device='disk'") {
                let disk_type = extract_attr(trimmed, "type").unwrap_or_else(|| "file".to_string());
                current_disk = Some(DiskBuilder {
                    device: String::new(),
                    path: String::new(),
                    disk_type,
                    bus: String::new(),
                    cache: None,
                    io: None,
                    discard: None,
                    detect_zeroes: None,
                    read_iops_sec: None,
                    write_iops_sec: None,
                    read_bytes_sec: None,
                    write_bytes_sec: None,
                });
            }

            if current_disk.is_none() {
                continue;
            }

            // Extract driver attributes (cache, io, discard, detect_zeroes)
            if trimmed.starts_with("<driver ") {
                if let Some(ref mut disk) = current_disk {
                    disk.cache = extract_attr(trimmed, "cache");
                    disk.io = extract_attr(trimmed, "io");
                    disk.discard = extract_attr(trimmed, "discard");
                    disk.detect_zeroes = extract_attr(trimmed, "detect_zeroes");
                }
            }

            // Extract source path
            if trimmed.contains("<source file=") {
                if let Some(ref mut disk) = current_disk {
                    if let Some(path) = extract_attr(trimmed, "file") {
                        disk.path = path;
                    }
                }
            }

            // Extract target device and bus
            if trimmed.contains("<target dev=") {
                if let Some(ref mut disk) = current_disk {
                    if let Some(dev) = extract_attr(trimmed, "dev") {
                        disk.device = dev;
                    }
                    disk.bus = extract_attr(trimmed, "bus").unwrap_or_else(|| "virtio".to_string());
                }
            }

            // Parse iotune section
            if trimmed == "<iotune>" {
                in_iotune = true;
            }
            if trimmed == "</iotune>" {
                in_iotune = false;
            }

            if in_iotune {
                if let Some(ref mut disk) = current_disk {
                    if trimmed.starts_with("<read_iops_sec>") {
                        let val = trimmed
                            .replace("<read_iops_sec>", "")
                            .replace("</read_iops_sec>", "");
                        disk.read_iops_sec = val.parse().ok();
                    }
                    if trimmed.starts_with("<write_iops_sec>") {
                        let val = trimmed
                            .replace("<write_iops_sec>", "")
                            .replace("</write_iops_sec>", "");
                        disk.write_iops_sec = val.parse().ok();
                    }
                    if trimmed.starts_with("<read_bytes_sec>") {
                        let val = trimmed
                            .replace("<read_bytes_sec>", "")
                            .replace("</read_bytes_sec>", "");
                        disk.read_bytes_sec = val.parse().ok();
                    }
                    if trimmed.starts_with("<write_bytes_sec>") {
                        let val = trimmed
                            .replace("<write_bytes_sec>", "")
                            .replace("</write_bytes_sec>", "");
                        disk.write_bytes_sec = val.parse().ok();
                    }
                }
            }

            // End of disk device - save it
            if trimmed == "</disk>" {
                if let Some(disk) = current_disk.take() {
                    if !disk.device.is_empty() && !disk.path.is_empty() {
                        disks.push(crate::models::vm::DiskDevice {
                            device: disk.device,
                            path: disk.path,
                            disk_type: disk.disk_type,
                            bus: disk.bus,
                            cache: disk.cache,
                            io: disk.io,
                            discard: disk.discard,
                            detect_zeroes: disk.detect_zeroes,
                            read_iops_sec: disk.read_iops_sec,
                            write_iops_sec: disk.write_iops_sec,
                            read_bytes_sec: disk.read_bytes_sec,
                            write_bytes_sec: disk.write_bytes_sec,
                        });
                    }
                }
            }
        }

        Ok(disks)
    }

    /// Calculate total disk size using virsh domblkinfo (works through libvirt with proper permissions)
    fn calculate_disk_size_from_domain(domain: &Domain) -> u64 {
        let vm_name = match domain.get_name() {
            Ok(name) => name,
            Err(_) => return 0,
        };

        let mut total_bytes: u64 = 0;

        // Get list of block devices using virsh domblklist
        let output = Self::virsh_command_for_domain(domain)
            .ok()
            .and_then(|mut command| {
                command
                    .args(["domblklist", &vm_name, "--details"])
                    .output()
                    .ok()
            });

        if let Some(out) = output {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                // Parse output - skip header lines, get device names
                // Format: Type   Device   Target   Source
                for line in text.lines().skip(2) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        let device_type = parts[0];
                        let target = parts[2];

                        // Only count disks, not cdroms
                        if device_type == "file" || device_type == "block" {
                            // Get size using domblkinfo
                            let info_output = Self::virsh_command_for_domain(domain).ok().and_then(
                                |mut command| {
                                    command.args(["domblkinfo", &vm_name, target]).output().ok()
                                },
                            );

                            if let Some(info_out) = info_output {
                                if info_out.status.success() {
                                    let info_text = String::from_utf8_lossy(&info_out.stdout);
                                    for info_line in info_text.lines() {
                                        if info_line.starts_with("Capacity:") {
                                            if let Some(size_str) =
                                                info_line.split_whitespace().nth(1)
                                            {
                                                if let Ok(size) = size_str.parse::<u64>() {
                                                    total_bytes += size;
                                                }
                                            }
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Convert bytes to GB
        total_bytes / (1024 * 1024 * 1024)
    }

    /// Get VM hardware configuration (firmware, TPM, chipset) from XML
    fn get_vm_hardware_config(domain: &Domain) -> Result<(String, bool, String), AppError> {
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        let mut firmware = "bios".to_string();
        let mut tpm_enabled = false;
        let mut chipset = "pc".to_string();

        for line in xml.lines() {
            let trimmed = line.trim();

            // Check for UEFI loader
            if trimmed.contains("<loader") {
                if trimmed.contains("secure='yes'") || trimmed.contains("secboot") {
                    firmware = "uefi-secure".to_string();
                } else {
                    firmware = "uefi".to_string();
                }
            }

            // Check for TPM device
            if trimmed.contains("<tpm ") || trimmed.contains("<tpm>") {
                tpm_enabled = true;
            }

            // Check for machine type/chipset
            if trimmed.contains("<type arch=") && trimmed.contains("machine=") {
                if let Some(start) = trimmed.find("machine='") {
                    let start_idx = start + 9;
                    if let Some(end) = trimmed[start_idx..].find('\'') {
                        let machine = &trimmed[start_idx..start_idx + end];
                        if machine.contains("q35") {
                            chipset = "q35".to_string();
                        } else {
                            chipset = "pc".to_string();
                        }
                    }
                } else if let Some(start) = trimmed.find("machine=\"") {
                    let start_idx = start + 9;
                    if let Some(end) = trimmed[start_idx..].find('\"') {
                        let machine = &trimmed[start_idx..start_idx + end];
                        if machine.contains("q35") {
                            chipset = "q35".to_string();
                        } else {
                            chipset = "pc".to_string();
                        }
                    }
                }
            }
        }

        Ok((firmware, tpm_enabled, chipset))
    }

    /// Get CPU topology from domain XML
    fn get_cpu_topology(domain: &Domain) -> Result<(u32, u32, u32), AppError> {
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        let mut sockets = 1;
        let mut cores = 1;
        let mut threads = 1;

        for line in xml.lines() {
            let trimmed = line.trim();

            // Look for <topology sockets='X' cores='Y' threads='Z'/>
            if trimmed.contains("<topology") {
                // Parse sockets
                if let Some(start) = trimmed.find("sockets='") {
                    let start_idx = start + 9;
                    if let Some(end) = trimmed[start_idx..].find('\'') {
                        if let Ok(val) = trimmed[start_idx..start_idx + end].parse::<u32>() {
                            sockets = val;
                        }
                    }
                } else if let Some(start) = trimmed.find("sockets=\"") {
                    let start_idx = start + 9;
                    if let Some(end) = trimmed[start_idx..].find('\"') {
                        if let Ok(val) = trimmed[start_idx..start_idx + end].parse::<u32>() {
                            sockets = val;
                        }
                    }
                }

                // Parse cores
                if let Some(start) = trimmed.find("cores='") {
                    let start_idx = start + 7;
                    if let Some(end) = trimmed[start_idx..].find('\'') {
                        if let Ok(val) = trimmed[start_idx..start_idx + end].parse::<u32>() {
                            cores = val;
                        }
                    }
                } else if let Some(start) = trimmed.find("cores=\"") {
                    let start_idx = start + 7;
                    if let Some(end) = trimmed[start_idx..].find('\"') {
                        if let Ok(val) = trimmed[start_idx..start_idx + end].parse::<u32>() {
                            cores = val;
                        }
                    }
                }

                // Parse threads
                if let Some(start) = trimmed.find("threads='") {
                    let start_idx = start + 9;
                    if let Some(end) = trimmed[start_idx..].find('\'') {
                        if let Ok(val) = trimmed[start_idx..start_idx + end].parse::<u32>() {
                            threads = val;
                        }
                    }
                } else if let Some(start) = trimmed.find("threads=\"") {
                    let start_idx = start + 9;
                    if let Some(end) = trimmed[start_idx..].find('\"') {
                        if let Ok(val) = trimmed[start_idx..start_idx + end].parse::<u32>() {
                            threads = val;
                        }
                    }
                }

                break; // Found topology, no need to continue
            }
        }

        Ok((sockets, cores, threads))
    }

    /// Create a new VM
    pub fn create_vm(
        libvirt: &impl ConnectionProvider,
        context: &OperationContext,
        config: crate::models::vm::VmConfig,
    ) -> Result<String, AppError> {
        use virt::storage_pool::StoragePool;

        Self::validate_vm_name(&config.name)?;
        Self::choice(
            &config.disk_format,
            "Disk format",
            &["raw", "qcow2", "qcow", "vmdk", "vdi", "vpc", "qed"],
        )?;
        Self::choice(
            &config.firmware,
            "Firmware",
            &["bios", "uefi", "uefi-secure"],
        )?;
        Self::choice(&config.chipset, "Chipset", &["pc", "q35"])?;
        Self::choice(&config.graphics_type, "Graphics type", &["spice", "vnc"])?;
        Self::choice(
            &config.video_model,
            "Video model",
            &[
                "qxl", "virtio", "vga", "cirrus", "vmvga", "bochs", "ramfb", "none",
            ],
        )?;

        tracing::info!("Creating VM: {} (OS: {}, Network: {}, Firmware: {}, Chipset: {}, TPM: {}, Installation: {})",
                      config.name, config.os_type, config.network, config.firmware, config.chipset, config.tpm_enabled, config.installation_type);

        // Validate CPU topology: sockets * cores * threads must equal cpu_count
        let calculated_vcpus = config.cpu_sockets * config.cpu_cores * config.cpu_threads;
        if calculated_vcpus != config.cpu_count {
            return Err(AppError::InvalidConfig(
                format!("CPU topology mismatch: sockets({}) × cores({}) × threads({}) = {} but cpu_count is {}. They must be equal.",
                    config.cpu_sockets, config.cpu_cores, config.cpu_threads, calculated_vcpus, config.cpu_count)
            ));
        }

        tracing::info!(
            "CPU Topology: {} sockets × {} cores × {} threads = {} vCPUs",
            config.cpu_sockets,
            config.cpu_cores,
            config.cpu_threads,
            config.cpu_count
        );

        let conn = libvirt.get_connection();

        let review = HostReadinessService::preflight(libvirt, context, &config)?;
        if !review.can_create {
            return Err(AppError::InvalidConfig(
                "The selected connection does not satisfy the reviewed VM requirements".to_string(),
            ));
        }

        // Validate network install URL if network installation is selected
        if config.installation_type == "network" {
            if let Some(ref url) = config.network_install_url {
                // Basic URL validation
                if !url.starts_with("http://")
                    && !url.starts_with("https://")
                    && !url.starts_with("ftp://")
                {
                    return Err(AppError::InvalidConfig(
                        "Network install URL must start with http://, https://, or ftp://"
                            .to_string(),
                    ));
                }
                let _ = url;
                tracing::info!("Network installation was configured");
            }
            // URL is optional - network boot can work with PXE server on the network
        }

        // Handle disk path based on installation type:
        // - "iso": Create a new storage volume
        // - "import": Use existing disk image
        // - "network": Create a new storage volume (boot from network)
        // - "manual": Create a new storage volume (but no ISO)
        let (disk_path, created_volume): (String, Option<virt::storage_vol::StorageVol>) = if config
            .installation_type
            == "import"
        {
            // Import existing disk image
            let existing_path = config.existing_disk_path.clone().ok_or_else(|| {
                AppError::InvalidConfig(
                    "Import installation type requires existing_disk_path".to_string(),
                )
            })?;

            // Validate the existing disk path exists
            if !std::path::Path::new(&existing_path).exists() {
                return Err(AppError::InvalidConfig(format!(
                    "Existing disk image not found: {}",
                    existing_path
                )));
            }

            tracing::info!("Using an existing disk image");
            (existing_path, None)
        } else {
            let pool_id = config.storage_pool_id.as_deref().ok_or_else(|| {
                AppError::InvalidConfig(
                    "Select an active storage pool with enough available capacity".to_string(),
                )
            })?;
            let pool = StoragePool::lookup_by_uuid_string(conn, pool_id).map_err(|_| {
                AppError::InvalidConfig(
                    "The selected storage pool is no longer available on this connection"
                        .to_string(),
                )
            })?;
            if !pool.is_active().map_err(map_libvirt_error)? {
                return Err(AppError::Unavailable(
                    "The selected storage pool is inactive".to_string(),
                ));
            }

            // Define volume name with proper extension based on disk format
            let volume_name = format!("{}.{}", config.name, config.disk_format);
            let disk_size_bytes =
                crate::services::storage_service::StorageService::disk_bytes(config.disk_size_gb)?;
            if pool.get_info().map_err(map_libvirt_error)?.available < disk_size_bytes {
                return Err(AppError::InvalidConfig(
                    "The selected storage pool no longer has enough available capacity".to_string(),
                ));
            }

            // Create storage volume XML with proper disk format
            let volume_xml = format!(
                r#"<volume>
  <name>{}</name>
  <capacity unit='bytes'>{}</capacity>
  <target>
    <format type='{}'/>
  </target>
</volume>"#,
                escaped_text(&volume_name, "Volume name")?,
                disk_size_bytes,
                escaped_attribute(&config.disk_format, "Disk format")?
            );

            tracing::info!(
                "Creating storage volume: {} ({}GB, format: {})",
                volume_name,
                config.disk_size_gb,
                config.disk_format
            );

            // Create the volume using libvirt storage pool API
            let volume =
                virt::storage_vol::StorageVol::create_xml(&pool, &volume_xml, 0).map_err(|e| {
                    AppError::LibvirtError(format!("Failed to create storage volume: {}", e))
                })?;

            let path = volume
                .get_path()
                .map_err(|e| AppError::LibvirtError(format!("Failed to get volume path: {}", e)))?;

            tracing::info!("Storage volume created");
            (path, Some(volume))
        };

        // Determine chipset/machine type
        let machine_type = match config.chipset.as_str() {
            "q35" => "q35",
            _ => "pc",
        };

        // Build boot device order from configuration
        // If boot_order is empty, use intelligent defaults based on ISO presence
        let boot_devices = if config.boot_order.is_empty() {
            if config.iso_path.is_some() {
                vec!["cdrom", "hd"]
            } else {
                vec!["hd"]
            }
        } else {
            config.boot_order.iter().map(|s| s.as_str()).collect()
        };

        let boot_order = boot_devices
            .iter()
            .map(|dev| {
                Self::choice(dev, "Boot device", &["hd", "cdrom", "network", "fd"])?;
                Ok(format!("    <boot dev='{dev}'/>"))
            })
            .collect::<Result<Vec<_>, AppError>>()?
            .join("\n");

        // Let libvirt select firmware advertised by this connection. Capability preflight has
        // already verified the selected mode without assuming distribution-specific paths.
        let firmware_section = match config.firmware.as_str() {
            "uefi" => {
                "    <firmware><feature enabled='no' name='secure-boot'/></firmware>".to_string()
            }
            "uefi-secure" => "    <firmware><feature enabled='yes' name='secure-boot'/><feature enabled='yes' name='enrolled-keys'/></firmware>".to_string(),
            _ => String::new(),
        };
        let os_firmware_attribute = if config.firmware == "bios" {
            ""
        } else {
            " firmware='efi'"
        };

        // Determine disk format - for imports, detect from file extension
        let actual_disk_format = if config.installation_type == "import" {
            // Try to detect format from file extension
            let path = std::path::Path::new(&disk_path);
            match path.extension().and_then(|e| e.to_str()) {
                Some("qcow2") => "qcow2",
                Some("raw") | Some("img") => "raw",
                Some("vmdk") => "vmdk",
                Some("vdi") => "vdi",
                Some("vpc") | Some("vhd") => "vpc",
                Some("qed") => "qed",
                _ => &config.disk_format, // Fallback to config
            }
        } else {
            &config.disk_format
        };

        // Build disk XML for the main system disk
        let main_disk_xml = format!(
            r#"    <disk type='file' device='disk'>
      <driver name='qemu' type='{}'/>
      <source file='{}'/>
      <target dev='vda' bus='virtio'/>
    </disk>"#,
            escaped_attribute(actual_disk_format, "Disk format")?,
            escaped_attribute(&disk_path, "Disk path")?
        );

        // Build CDROM/ISO disk XML if ISO path is provided
        // Q35 chipset doesn't support IDE, use SATA instead
        let (cdrom_dev, cdrom_bus) = if config.chipset == "q35" {
            ("sda", "sata")
        } else {
            ("hdc", "ide")
        };

        let cdrom_xml = if let Some(ref iso_path) = config.iso_path {
            format!(
                r#"    <disk type='file' device='cdrom'>
      <driver name='qemu' type='raw'/>
      <source file='{}'/>
      <target dev='{}' bus='{}'/>
      <readonly/>
    </disk>"#,
                escaped_attribute(iso_path, "ISO path")?,
                cdrom_dev,
                cdrom_bus
            )
        } else {
            String::new()
        };

        // Cloud-init uses a separate readonly CD-ROM from installation media. The service derives
        // and validates the filename before creating the ISO and emits an escaped XML fragment.
        let cloud_init_xml = if let Some(cloud_init) =
            config.cloud_init.as_ref().filter(|config| config.enabled)
        {
            let instance_id = uuid::Uuid::new_v4().to_string();
            let iso_path = CloudInitService::generate_iso(cloud_init, &config.name, &instance_id)?;
            let (target, bus) = if config.chipset == "q35" {
                ("sdb", "sata")
            } else {
                ("hdd", "ide")
            };
            CloudInitService::attachment_definition(&iso_path, target, bus)?
        } else {
            String::new()
        };

        // Build network interface XML
        // For network installation, add boot order to the interface
        let interface_xml = if config.installation_type == "network" {
            format!(
                r#"    <interface type='network'>
      <source network='{}'/>
      <model type='virtio'/>
      <boot order='1'/>
    </interface>"#,
                escaped_attribute(&config.network, "Network name")?
            )
        } else {
            format!(
                r#"    <interface type='network'>
      <source network='{}'/>
      <model type='virtio'/>
    </interface>"#,
                escaped_attribute(&config.network, "Network name")?
            )
        };

        // Build boot menu option
        let boot_menu = if config.boot_menu {
            "    <bootmenu enable='yes' timeout='3000'/>"
        } else {
            ""
        };

        // Build direct kernel boot XML if enabled
        // This allows booting directly from a kernel/initrd on the host
        let direct_kernel_boot_xml = if config.direct_kernel_boot {
            let mut kernel_xml = String::new();

            if let Some(ref kernel_path) = config.kernel_path {
                if !kernel_path.is_empty() {
                    // Verify kernel file exists
                    if !std::path::Path::new(kernel_path).exists() {
                        return Err(AppError::InvalidConfig(format!(
                            "Kernel file not found: {}",
                            kernel_path
                        )));
                    }
                    kernel_xml.push_str(&format!(
                        "    <kernel>{}</kernel>\n",
                        escaped_text(kernel_path, "Kernel path")?
                    ));
                }
            }

            if let Some(ref initrd_path) = config.initrd_path {
                if !initrd_path.is_empty() {
                    // Verify initrd file exists
                    if !std::path::Path::new(initrd_path).exists() {
                        return Err(AppError::InvalidConfig(format!(
                            "Initrd file not found: {}",
                            initrd_path
                        )));
                    }
                    kernel_xml.push_str(&format!(
                        "    <initrd>{}</initrd>\n",
                        escaped_text(initrd_path, "Initrd path")?
                    ));
                }
            }

            if let Some(ref kernel_args) = config.kernel_args {
                if !kernel_args.is_empty() {
                    kernel_xml.push_str(&format!(
                        "    <cmdline>{}</cmdline>\n",
                        escaped_text(kernel_args, "Kernel arguments")?
                    ));
                }
            }

            if let Some(ref dtb_path) = config.dtb_path {
                if !dtb_path.is_empty() {
                    // Verify DTB file exists (for ARM systems)
                    if !std::path::Path::new(dtb_path).exists() {
                        return Err(AppError::InvalidConfig(format!(
                            "Device tree blob file not found: {}",
                            dtb_path
                        )));
                    }
                    kernel_xml.push_str(&format!(
                        "    <dtb>{}</dtb>\n",
                        escaped_text(dtb_path, "Device tree path")?
                    ));
                }
            }

            kernel_xml.trim_end().to_string()
        } else {
            String::new()
        };

        // Build TPM device XML if enabled
        let tpm_xml = if config.tpm_enabled {
            r#"    <tpm model='tpm-tis'>
      <backend type='emulator' version='2.0'/>
    </tpm>"#
        } else {
            ""
        };

        // Build PCI device passthrough XML
        let mut pci_devices_xml = String::new();
        for pci_address in &config.pci_devices {
            let (domain_part, bus, slot, function) = Self::parse_pci_address(pci_address)?;
            let hostdev = format!(
                r#"    <hostdev mode='subsystem' type='pci' managed='yes'>
      <source>
        <address domain='0x{domain_part:04x}' bus='0x{bus:02x}' slot='0x{slot:02x}' function='0x{function:x}'/>
      </source>
    </hostdev>
"#,
            );
            pci_devices_xml.push_str(&hostdev);
            tracing::info!(
                operation = "vm-pci-device-configured",
                "Configured PCI device"
            );
        }

        // Determine OS type
        let os_type_str = "hvm";

        // Build features section - ACPI is required for UEFI
        let features_xml = if config.firmware != "bios" {
            r#"  <features>
    <acpi/>
    <apic/>
  </features>"#
        } else {
            ""
        };

        // Build CPU topology XML
        let cpu_topology_xml = format!(
            r#"  <cpu>
    <topology sockets='{}' cores='{}' threads='{}'/>
  </cpu>"#,
            config.cpu_sockets, config.cpu_cores, config.cpu_threads
        );

        // Build graphics XML based on config
        let graphics_xml = match config.graphics_type.as_str() {
            "spice" => r#"    <graphics type='spice' autoport='yes'>
      <listen type='address'/>
    </graphics>"#
                .to_string(),
            "vnc" => "    <graphics type='vnc' port='-1' autoport='yes'/>".to_string(),
            _ => "    <graphics type='vnc' port='-1' autoport='yes'/>".to_string(),
        };

        // Build video XML based on config
        let video_xml = format!(
            r#"    <video>
      <model type='{}'/>
    </video>"#,
            escaped_attribute(&config.video_model, "Video model")?
        );

        // Build RNG device XML if enabled
        let rng_xml = if config.rng_enabled {
            r#"    <rng model='virtio'>
      <backend model='random'>/dev/urandom</backend>
    </rng>"#
                .to_string()
        } else {
            String::new()
        };

        // Build watchdog device XML if specified
        let watchdog_xml = if let Some(ref model) = config.watchdog_model {
            Self::choice(
                model,
                "Watchdog model",
                &["i6300esb", "ib700", "diag288", "itco", "watchdog"],
            )?;
            Self::choice(
                &config.watchdog_action,
                "Watchdog action",
                &[
                    "reset",
                    "shutdown",
                    "poweroff",
                    "pause",
                    "none",
                    "dump",
                    "inject-nmi",
                ],
            )?;
            format!(
                r#"    <watchdog model='{}' action='{}'/>"#,
                escaped_attribute(model, "Watchdog model")?,
                escaped_attribute(&config.watchdog_action, "Watchdog action")?
            )
        } else {
            String::new()
        };

        // Build hugepages memoryBacking XML if enabled
        let memory_backing_xml = if config.hugepages_enabled {
            if let Some(size) = config.hugepage_size {
                format!(
                    r#"  <memoryBacking>
    <hugepages>
      <page size='{}' unit='KiB'/>
    </hugepages>
  </memoryBacking>"#,
                    size
                )
            } else {
                // Use default hugepage size
                r#"  <memoryBacking>
    <hugepages/>
  </memoryBacking>"#
                    .to_string()
            }
        } else {
            String::new()
        };

        // Generate XML configuration for the VM
        // Set max memory to 2x current for ballooning headroom (capped at 128GB)
        let max_memory_mb = std::cmp::min(config.memory_mb * 2, 131072);

        let xml = format!(
            r#"<domain type='kvm'>
  <name>{}</name>
  <memory unit='MiB'>{}</memory>
  <currentMemory unit='MiB'>{}</currentMemory>
{}
  <vcpu>{}</vcpu>
{}
  <os{}>
    <type arch='x86_64' machine='{}'>{}</type>
{}
{}
{}
{}
  </os>
{}
  <devices>
{}
{}
{}
{}
{}
{}
    <serial type='pty'>
      <target port='0'/>
    </serial>
    <console type='pty'>
      <target type='serial' port='0'/>
    </console>
    <controller type='virtio-serial' index='0'>
      <address type='pci' domain='0x0000' bus='0x00' slot='0x08' function='0x0'/>
    </controller>
    <memballoon model='virtio'>
      <address type='pci' domain='0x0000' bus='0x00' slot='0x07' function='0x0'/>
    </memballoon>
{}
{}
{}
{}
  </devices>
</domain>"#,
            escaped_text(&config.name, "VM name")?,
            max_memory_mb,      // max memory
            config.memory_mb,   // current memory
            memory_backing_xml, // hugepages if enabled
            config.cpu_count,
            cpu_topology_xml,
            os_firmware_attribute,
            machine_type,
            os_type_str,
            firmware_section,
            boot_order,
            boot_menu,
            direct_kernel_boot_xml,
            features_xml,
            main_disk_xml,
            cdrom_xml,
            cloud_init_xml,
            interface_xml,
            graphics_xml,
            video_xml,
            tpm_xml,
            pci_devices_xml,
            rng_xml,
            watchdog_xml
        );

        tracing::debug!("Generated VM definition");

        // Define the domain (create VM configuration)
        let domain = match Domain::define_xml(conn, &xml) {
            Ok(d) => d,
            Err(e) => {
                // VM creation failed, clean up the storage volume if we created one
                tracing::error!("VM definition failed");
                if let Some(ref vol) = created_volume {
                    tracing::info!("Cleaning up storage volume after VM creation failure");
                    if vol.delete(0).is_err() {
                        tracing::warn!("VM creation cleanup did not complete");
                    }
                }
                return Err(map_libvirt_error(e));
            }
        };

        let uuid = domain.get_uuid_string().map_err(map_libvirt_error)?;

        tracing::info!("VM created successfully: {} (UUID: {})", config.name, uuid);
        Ok(uuid)
    }

    /// Get VM performance statistics
    /// Tries to get stats from guest agent first, falls back to libvirt stats
    pub fn get_vm_stats(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
    ) -> Result<crate::models::vm::VmStats, AppError> {
        use virt::sys;

        tracing::debug!("Getting stats for VM: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let vm_name = domain.get_name().map_err(map_libvirt_error)?;
        let info = domain.get_info().map_err(map_libvirt_error)?;

        // For stopped VMs, return zeros
        let (state, _) = domain.get_state().map_err(map_libvirt_error)?;

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

        // Get actual memory usage from dommemstat (not just allocated memory)
        let (memory_used_mb, memory_available_mb) = Self::get_libvirt_memory_stats(conn, &vm_name)
            .unwrap_or({
                // Fallback to basic info (allocated memory)
                ((info.memory / 1024) as u64, (info.max_mem / 1024) as u64)
            });

        // Try to get CPU usage from guest agent
        let cpu_usage_percent =
            Self::get_cpu_from_guest_agent(conn, &vm_name).unwrap_or_else(|_| {
                Self::calculate_libvirt_cpu_usage(&domain, info.nr_virt_cpu as u32)
            });

        // Get disk stats from libvirt
        let (disk_read_bytes, disk_write_bytes) = Self::get_libvirt_disk_stats(conn, &vm_name);

        // Get network stats from libvirt
        let (network_rx_bytes, network_tx_bytes) = Self::get_libvirt_network_stats(conn, &vm_name);

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

    /// Try to get CPU usage from guest agent
    fn get_cpu_from_guest_agent(
        conn: &virt::connect::Connect,
        vm_name: &str,
    ) -> Result<f64, AppError> {
        // Use virsh qemu-agent-command to get CPU stats
        let output = Self::virsh_command(conn)?
            .args([
                "qemu-agent-command",
                vm_name,
                r#"{"execute":"guest-get-cpustats"}"#,
            ])
            .output()
            .map_err(|e| AppError::Other(format!("Failed to execute virsh: {}", e)))?;

        if !output.status.success() {
            return Err(AppError::Other("Guest agent not available".to_string()));
        }

        let result: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| AppError::Other(format!("Failed to parse guest agent response: {}", e)))?;

        // Parse CPU stats and calculate usage
        let cpus = result
            .get("return")
            .and_then(|r| r.as_array())
            .ok_or_else(|| AppError::Other("Invalid guest agent response".to_string()))?;

        let mut total_active: u64 = 0;
        let mut total_all: u64 = 0;

        for cpu in cpus {
            let user = cpu.get("user").and_then(|v| v.as_u64()).unwrap_or(0);
            let system = cpu.get("system").and_then(|v| v.as_u64()).unwrap_or(0);
            let idle = cpu.get("idle").and_then(|v| v.as_u64()).unwrap_or(0);
            let iowait = cpu.get("iowait").and_then(|v| v.as_u64()).unwrap_or(0);
            let nice = cpu.get("nice").and_then(|v| v.as_u64()).unwrap_or(0);
            let irq = cpu.get("irq").and_then(|v| v.as_u64()).unwrap_or(0);
            let softirq = cpu.get("softirq").and_then(|v| v.as_u64()).unwrap_or(0);
            let steal = cpu.get("steal").and_then(|v| v.as_u64()).unwrap_or(0);

            let total = user + system + idle + iowait + nice + irq + softirq + steal;
            let active = total - idle;

            total_active += active;
            total_all += total;
        }

        let usage = if total_all > 0 {
            (total_active as f64 / total_all as f64) * 100.0
        } else {
            0.0
        };

        Ok(usage)
    }

    /// Calculate CPU usage from libvirt domain info (fallback method)
    /// Uses a quick sample to estimate CPU usage
    fn calculate_libvirt_cpu_usage(domain: &Domain, vcpus: u32) -> f64 {
        // For accurate CPU usage, we need to compare CPU time at two points.
        // Since this is a fallback when guest agent isn't available,
        // we'll use a short 50ms sample.
        use std::time::{Duration, Instant};

        let start = Instant::now();
        let info1 = match domain.get_info() {
            Ok(i) => i,
            Err(_) => return 0.0,
        };
        let time1 = info1.cpu_time;

        // Wait 50ms for a sample
        std::thread::sleep(Duration::from_millis(50));

        let info2 = match domain.get_info() {
            Ok(i) => i,
            Err(_) => return 0.0,
        };
        let time2 = info2.cpu_time;
        let elapsed = start.elapsed();

        // Calculate CPU usage as percentage
        // cpu_time is in nanoseconds
        let cpu_time_delta = time2.saturating_sub(time1);
        let real_time_ns = elapsed.as_nanos() as u64;

        if real_time_ns == 0 || vcpus == 0 {
            return 0.0;
        }

        // Usage across all vCPUs
        let usage = (cpu_time_delta as f64 / (real_time_ns as f64 * vcpus as f64)) * 100.0;

        usage.clamp(0.0, 100.0)
    }

    /// Get disk I/O stats from libvirt using virsh domblkstat
    fn get_libvirt_disk_stats(conn: &virt::connect::Connect, vm_name: &str) -> (u64, u64) {
        // Use virsh domblkstat to get disk stats
        let output = Self::virsh_command(conn).ok().and_then(|mut command| {
            command
                .args(["domblkstat", vm_name, "--human"])
                .output()
                .ok()
        });

        match output {
            Some(out) if out.status.success() => {
                let text = String::from_utf8_lossy(&out.stdout);
                let mut read_bytes: u64 = 0;
                let mut write_bytes: u64 = 0;

                for line in text.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("number of bytes read:") {
                        if let Some(value) = trimmed.split(':').nth(1) {
                            read_bytes = value.trim().parse().unwrap_or(0);
                        }
                    } else if trimmed.starts_with("number of bytes written:") {
                        if let Some(value) = trimmed.split(':').nth(1) {
                            write_bytes = value.trim().parse().unwrap_or(0);
                        }
                    }
                }

                (read_bytes, write_bytes)
            }
            _ => (0, 0),
        }
    }

    /// Get actual memory usage from libvirt using virsh dommemstat
    fn get_libvirt_memory_stats(
        conn: &virt::connect::Connect,
        vm_name: &str,
    ) -> Option<(u64, u64)> {
        let output = Self::virsh_command(conn)
            .ok()?
            .args(["dommemstat", vm_name])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let mut actual: u64 = 0;
        let mut available: u64 = 0;
        let mut usable: u64 = 0;

        for line in text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                match parts[0] {
                    "actual" => actual = parts[1].parse().unwrap_or(0),
                    "available" => available = parts[1].parse().unwrap_or(0),
                    "usable" => usable = parts[1].parse().unwrap_or(0),
                    _ => {}
                }
            }
        }

        // Memory is in KiB, convert to MiB
        // "available" is total memory seen by guest
        // "usable" is memory the guest can still allocate
        // Used = available - usable (matches what htop shows)
        let total_mb = available / 1024;
        let usable_mb = usable / 1024;
        let used_mb = total_mb.saturating_sub(usable_mb);

        // If available is 0, fall back to actual
        if total_mb == 0 {
            return Some((actual / 1024, actual / 1024));
        }

        Some((used_mb, total_mb))
    }

    /// Get network I/O stats from libvirt using virsh domifstat
    fn get_libvirt_network_stats(conn: &virt::connect::Connect, vm_name: &str) -> (u64, u64) {
        // First get list of interfaces
        let iface_list = Self::virsh_command(conn)
            .ok()
            .and_then(|mut command| command.args(["domiflist", vm_name]).output().ok());

        let ifaces: Vec<String> = match iface_list {
            Some(out) if out.status.success() => {
                let text = String::from_utf8_lossy(&out.stdout);
                text.lines()
                    .skip(2) // Skip header line and separator
                    .filter(|line| !line.trim().is_empty())
                    .filter_map(|line| {
                        let first = line.split_whitespace().next()?;
                        // Skip lines that are just dashes (separator)
                        if first.chars().all(|c| c == '-') {
                            None
                        } else {
                            Some(first.to_string())
                        }
                    })
                    .collect()
            }
            _ => vec!["vnet0".to_string()], // Fallback
        };

        let mut total_rx: u64 = 0;
        let mut total_tx: u64 = 0;

        for iface in &ifaces {
            let output = Self::virsh_command(conn)
                .ok()
                .and_then(|mut command| command.args(["domifstat", vm_name, iface]).output().ok());

            if let Some(out) = output {
                if out.status.success() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    for line in text.lines() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            match parts[1] {
                                "rx_bytes" => total_rx += parts[2].parse().unwrap_or(0),
                                "tx_bytes" => total_tx += parts[2].parse().unwrap_or(0),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        (total_rx, total_tx)
    }

    /// Clone a VM with disk and snapshot options
    pub fn clone_vm_with_options(
        libvirt: &impl ConnectionProvider,
        source_vm_id: &str,
        config: &crate::models::vm::CloneConfig,
    ) -> Result<String, AppError> {
        use std::path::Path;
        use std::process::Command;

        let cleanup_cloned_disks = |paths: &[String]| {
            paths.iter().all(|path| match std::fs::remove_file(path) {
                Ok(()) => true,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
                Err(_) => false,
            })
        };

        tracing::info!(
            "Cloning VM {} to {} (clone_disks: {}, clone_snapshots: {})",
            source_vm_id,
            config.new_name,
            config.clone_disks,
            config.clone_snapshots
        );

        let conn = libvirt.get_connection();
        let source_domain = Domain::lookup_by_uuid_string(conn, source_vm_id)
            .map_err(|_| AppError::VmNotFound(source_vm_id.to_string()))?;

        // Ensure source VM is not running
        let state = source_domain.get_state().map_err(map_libvirt_error)?;
        if state.0 == virt::sys::VIR_DOMAIN_RUNNING {
            return Err(AppError::LibvirtError(
                "Cannot clone a running VM. Please shut it down first.".to_string(),
            ));
        }

        // Get source VM XML
        let source_xml = source_domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        // Extract disk paths from source XML
        let disk_paths = Self::extract_disk_paths(&source_xml);
        let mut disk_mappings: Vec<(String, String)> = Vec::new();
        let mut created_disk_paths = Vec::new();

        if config.clone_disks && !disk_paths.is_empty() {
            tracing::info!("Cloning {} disk(s)", disk_paths.len());

            for (index, source_path) in disk_paths.iter().enumerate() {
                let source_path_obj = Path::new(source_path);

                // Skip CDROM devices or non-existent paths
                if !source_path_obj.exists() {
                    continue;
                }

                // Generate new disk filename
                let parent = source_path_obj
                    .parent()
                    .unwrap_or(Path::new("/var/lib/libvirt/images"));
                let extension = source_path_obj
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("qcow2");

                let new_filename = if disk_paths.len() > 1 {
                    format!("{}-disk{}.{}", config.new_name, index, extension)
                } else {
                    format!("{}.{}", config.new_name, extension)
                };

                let target_path = if let Some(ref pool) = config.target_pool {
                    // Get pool path from libvirt
                    if let Ok(pool_obj) =
                        virt::storage_pool::StoragePool::lookup_by_name(conn, pool)
                    {
                        if let Ok(pool_xml) = pool_obj.get_xml_desc(0) {
                            let pool_path = Self::extract_pool_path(&pool_xml)
                                .unwrap_or_else(|| "/var/lib/libvirt/images".to_string());
                            format!("{}/{}", pool_path, new_filename)
                        } else {
                            parent.join(&new_filename).to_string_lossy().to_string()
                        }
                    } else {
                        parent.join(&new_filename).to_string_lossy().to_string()
                    }
                } else {
                    parent.join(&new_filename).to_string_lossy().to_string()
                };

                // Clone the disk using qemu-img
                tracing::info!("Cloning a VM disk");
                let output = match Command::new("qemu-img")
                    .args([
                        "create",
                        "-f",
                        "qcow2",
                        "-F",
                        "qcow2",
                        "-b",
                        source_path,
                        &target_path,
                    ])
                    .output()
                {
                    Ok(output) => output,
                    Err(_) => {
                        return Err(if cleanup_cloned_disks(&created_disk_paths) {
                            AppError::Other("Disk clone could not be started".to_string())
                        } else {
                            AppError::Partial(
                                "A failed disk clone left resources that require inspection"
                                    .to_string(),
                            )
                        });
                    }
                };

                if !output.status.success() {
                    // If backing file method fails, try full copy
                    tracing::info!("Backing file clone failed, attempting full copy");
                    let _ = std::fs::remove_file(&target_path);
                    let output = match Command::new("qemu-img")
                        .args(["convert", "-O", "qcow2", source_path, &target_path])
                        .output()
                    {
                        Ok(output) => output,
                        Err(_) => {
                            let mut cleanup_paths = created_disk_paths.clone();
                            cleanup_paths.push(target_path.clone());
                            return Err(if cleanup_cloned_disks(&cleanup_paths) {
                                AppError::Other("Disk clone could not be started".to_string())
                            } else {
                                AppError::Partial(
                                    "A failed disk clone left resources that require inspection"
                                        .to_string(),
                                )
                            });
                        }
                    };

                    if !output.status.success() {
                        let mut cleanup_paths = created_disk_paths.clone();
                        cleanup_paths.push(target_path.clone());
                        return Err(if cleanup_cloned_disks(&cleanup_paths) {
                            AppError::Other("Disk clone failed".to_string())
                        } else {
                            AppError::Partial(
                                "A failed disk clone left resources that require inspection"
                                    .to_string(),
                            )
                        });
                    }
                }

                created_disk_paths.push(target_path.clone());
                disk_mappings.push((source_path.clone(), target_path));
            }
        }

        // Modify XML for clone (new name, MAC addresses, disk paths if cloned)
        let clone_xml = match Self::modify_xml_for_clone_extended(
            &source_xml,
            &config.new_name,
            &disk_mappings,
            &config.description,
        ) {
            Ok(xml) => xml,
            Err(error) => {
                return Err(if cleanup_cloned_disks(&created_disk_paths) {
                    error
                } else {
                    AppError::Partial(
                        "Clone preparation left disk resources that require inspection".to_string(),
                    )
                });
            }
        };

        // Define the cloned domain
        let clone_domain = match Domain::define_xml(conn, &clone_xml) {
            Ok(domain) => domain,
            Err(_) => {
                return Err(if cleanup_cloned_disks(&created_disk_paths) {
                    AppError::LibvirtError(
                        "The cloned virtual machine could not be defined".to_string(),
                    )
                } else {
                    AppError::Partial(
                        "Clone definition failed after creating disk resources".to_string(),
                    )
                });
            }
        };

        let clone_uuid = match clone_domain.get_uuid_string() {
            Ok(uuid) => uuid,
            Err(_) => {
                let domain_removed = clone_domain.undefine().is_ok();
                let disks_removed = cleanup_cloned_disks(&created_disk_paths);
                return Err(if domain_removed && disks_removed {
                    AppError::LibvirtError(
                        "The cloned virtual machine could not be finalized".to_string(),
                    )
                } else {
                    AppError::Partial(
                        "Clone finalization left resources that require inspection".to_string(),
                    )
                });
            }
        };

        // Clone snapshots if requested
        if config.clone_snapshots {
            let snapshots_complete =
                Self::clone_snapshots(libvirt, source_vm_id, &clone_uuid).unwrap_or(false);
            if !snapshots_complete {
                return Err(AppError::Partial(
                    "The clone was created but its snapshots require inspection".to_string(),
                ));
            }
        }

        tracing::info!(
            "VM cloned successfully: {} (UUID: {})",
            config.new_name,
            clone_uuid
        );
        Ok(clone_uuid)
    }

    /// Extract disk paths from VM XML
    fn extract_disk_paths(xml: &str) -> Vec<String> {
        let mut paths = Vec::new();
        let mut search_pos = 0;

        while let Some(disk_start) = xml[search_pos..].find("<disk ") {
            let abs_disk_start = search_pos + disk_start;

            // Find the end of this disk element
            if let Some(disk_end) = xml[abs_disk_start..].find("</disk>") {
                let disk_section = &xml[abs_disk_start..abs_disk_start + disk_end + 7];

                // Check if it's a disk device (not cdrom)
                if disk_section.contains("device='disk'")
                    || disk_section.contains("device=\"disk\"")
                {
                    // Extract source file path
                    if let Some(source_match) = Self::extract_source_file(disk_section) {
                        paths.push(source_match);
                    }
                }

                search_pos = abs_disk_start + disk_end + 7;
            } else {
                break;
            }
        }

        paths
    }

    /// Extract source file from disk XML section
    fn extract_source_file(disk_section: &str) -> Option<String> {
        // Look for <source file='...'> or <source file="...">
        if let Some(source_start) = disk_section.find("<source ") {
            let source_section = &disk_section[source_start..];

            // Try single quotes first
            if let Some(file_start) = source_section.find("file='") {
                let path_start = source_section[file_start + 6..]
                    .find("'")
                    .map(|end| &source_section[file_start + 6..file_start + 6 + end]);
                if let Some(path) = path_start {
                    return Some(path.to_string());
                }
            }

            // Try double quotes
            if let Some(file_start) = source_section.find("file=\"") {
                let path_start = source_section[file_start + 6..]
                    .find("\"")
                    .map(|end| &source_section[file_start + 6..file_start + 6 + end]);
                if let Some(path) = path_start {
                    return Some(path.to_string());
                }
            }
        }
        None
    }

    /// Extract pool path from pool XML
    fn extract_pool_path(xml: &str) -> Option<String> {
        if let Some(target_start) = xml.find("<target>") {
            if let Some(path_start) = xml[target_start..].find("<path>") {
                let abs_path_start = target_start + path_start + 6;
                if let Some(path_end) = xml[abs_path_start..].find("</path>") {
                    return Some(xml[abs_path_start..abs_path_start + path_end].to_string());
                }
            }
        }
        None
    }

    /// Modify VM XML for cloning with disk path updates
    fn modify_xml_for_clone_extended(
        xml: &str,
        new_name: &str,
        disk_mappings: &[(String, String)],
        description: &Option<String>,
    ) -> Result<String, AppError> {
        Self::validate_vm_name(new_name)?;
        let mut modified = crate::utils::xml::remove_first_element(xml, "uuid")?;
        modified = crate::utils::xml::rewrite_first_text_element(&modified, "name", new_name)?;

        // Add or update description
        if let Some(desc) = description {
            if crate::utils::xml::first_element_text(&modified, "domain", "description")?.is_some()
            {
                modified =
                    crate::utils::xml::rewrite_first_text_element(&modified, "description", desc)?;
            } else {
                modified = crate::utils::xml::insert_text_element_after_first(
                    &modified,
                    "name",
                    "description",
                    desc,
                )?;
            }
        }

        // Update disk paths
        for (old_path, new_path) in disk_mappings {
            modified = crate::utils::xml::rewrite_disk_source_path(&modified, old_path, new_path)?;
        }

        // Generate new MAC addresses for network interfaces
        modified = Self::regenerate_mac_addresses(&modified);

        Ok(modified)
    }

    /// Clone snapshots from source VM to target VM
    fn clone_snapshots(
        libvirt: &impl ConnectionProvider,
        source_vm_id: &str,
        target_vm_id: &str,
    ) -> Result<bool, AppError> {
        use virt::domain_snapshot::DomainSnapshot;

        tracing::info!(
            "Cloning snapshots from {} to {}",
            source_vm_id,
            target_vm_id
        );

        let conn = libvirt.get_connection();
        let source_domain = Domain::lookup_by_uuid_string(conn, source_vm_id)
            .map_err(|_| AppError::VmNotFound(source_vm_id.to_string()))?;
        let target_domain = Domain::lookup_by_uuid_string(conn, target_vm_id)
            .map_err(|_| AppError::VmNotFound(target_vm_id.to_string()))?;

        let snapshot_count = DomainSnapshot::num(&source_domain, 0).map_err(map_libvirt_error)?;

        if snapshot_count == 0 {
            tracing::info!("No snapshots to clone");
            return Ok(true);
        }

        let source_snapshots = source_domain
            .list_all_snapshots(0)
            .map_err(map_libvirt_error)?;

        // Clone each snapshot (without memory state - disk-only snapshots)
        let mut complete = true;
        for snap in source_snapshots {
            let name = snap.get_name().map_err(map_libvirt_error)?;
            let description = snap
                .get_xml_desc(0)
                .ok()
                .and_then(|xml| {
                    if let Some(start) = xml.find("<description>") {
                        let section = &xml[start + 13..];
                        if let Some(end) = section.find("</description>") {
                            return Some(section[..end].to_string());
                        }
                    }
                    None
                })
                .unwrap_or_else(|| format!("Cloned from {}", source_vm_id));

            // Create disk-only snapshot on target
            let snap_xml = format!(
                r#"<domainsnapshot>
  <name>{}</name>
  <description>{}</description>
  <memory snapshot='no'/>
</domainsnapshot>"#,
                name, description
            );

            match DomainSnapshot::create_xml(&target_domain, &snap_xml, 0) {
                Ok(_) => tracing::info!("Cloned snapshot: {}", name),
                Err(_) => {
                    complete = false;
                    tracing::warn!("A VM snapshot could not be cloned");
                }
            }
        }

        Ok(complete)
    }

    /// Clone a VM (legacy method - calls enhanced version with defaults)
    pub fn clone_vm(
        libvirt: &impl ConnectionProvider,
        source_vm_id: &str,
        new_name: &str,
    ) -> Result<String, AppError> {
        let config = crate::models::vm::CloneConfig {
            new_name: new_name.to_string(),
            clone_disks: false, // Legacy behavior: reference existing disks
            clone_snapshots: false,
            target_pool: None,
            description: None,
        };
        Self::clone_vm_with_options(libvirt, source_vm_id, &config)
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
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

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
                        let tag =
                            metadata_section[abs_tag_start..abs_tag_start + tag_end].to_string();
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
    pub fn add_vm_tags(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        tags: Vec<String>,
    ) -> Result<(), AppError> {
        tracing::info!("Adding VM tags");

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Get current XML
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        // Get existing tags
        let mut existing_tags = Self::get_vm_tags(&domain)?;

        // Add new tags (avoiding duplicates)
        for tag in tags {
            if !existing_tags.contains(&tag) {
                existing_tags.push(tag);
            }
        }

        // Build metadata XML
        let tags_xml = existing_tags
            .iter()
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
        let _new_domain = Domain::define_xml(conn, &new_xml).map_err(map_libvirt_error)?;

        tracing::info!("Tags added successfully to VM {}", vm_id);
        Ok(())
    }

    /// Remove tags from a VM
    pub fn remove_vm_tags(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        tags_to_remove: Vec<String>,
    ) -> Result<(), AppError> {
        tracing::info!("Removing VM tags");

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Get current XML
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

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

            let _new_domain = Domain::define_xml(conn, &new_xml).map_err(map_libvirt_error)?;
        } else {
            // Update with remaining tags
            Self::add_vm_tags(libvirt, vm_id, existing_tags)?;
        }

        tracing::info!("Tags removed successfully from VM {}", vm_id);
        Ok(())
    }

    /// Export a VM's configuration to XML
    pub fn export_vm(libvirt: &impl ConnectionProvider, vm_id: &str) -> Result<String, AppError> {
        tracing::info!("Exporting VM configuration for {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Get XML description with secure flag to include sensitive data
        let xml = domain
            .get_xml_desc(sys::VIR_DOMAIN_XML_SECURE)
            .map_err(map_libvirt_error)?;

        tracing::info!("Successfully exported VM configuration for {}", vm_id);
        Ok(xml)
    }

    /// Import a VM from XML configuration
    pub fn import_vm(libvirt: &impl ConnectionProvider, xml: &str) -> Result<String, AppError> {
        tracing::info!("Importing VM from XML configuration");

        crate::utils::xml::validate_document_root(xml, "domain")?;

        let conn = libvirt.get_connection();

        // Define domain from XML
        let domain = Domain::define_xml(conn, xml).map_err(map_libvirt_error)?;

        // A definition with no returned identity cannot be safely managed by the caller.
        let uuid = match domain.get_uuid_string() {
            Ok(uuid) => uuid,
            Err(_) if domain.undefine().is_ok() => {
                return Err(AppError::LibvirtError(
                    "The imported virtual machine could not be finalized".to_string(),
                ));
            }
            Err(_) => {
                return Err(AppError::Partial(
                    "Import finalization left a virtual machine that requires inspection"
                        .to_string(),
                ));
            }
        };

        tracing::info!("Successfully imported VM with UUID {}", uuid);
        Ok(uuid)
    }

    /// Attach a disk to a VM
    pub fn attach_disk(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        disk_path: &str,
        device_target: &str,
        bus_type: &str,
    ) -> Result<(), AppError> {
        tracing::info!(
            "Attaching disk {} to VM {} as {}",
            disk_path,
            vm_id,
            device_target
        );

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Validate bus type (including NVMe)
        let valid_bus_types = ["virtio", "scsi", "sata", "ide", "nvme"];
        if !valid_bus_types.contains(&bus_type) {
            return Err(AppError::InvalidConfig(format!(
                "Invalid bus type '{}'. Must be one of: {:?}",
                bus_type, valid_bus_types
            )));
        }
        if device_target.is_empty()
            || device_target.len() > 32
            || !device_target
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return Err(AppError::InvalidConfig(
                "Disk target has an invalid format".to_string(),
            ));
        }
        let disk_path = escaped_attribute(disk_path, "Disk path")?;

        // Build disk XML - NVMe requires different target naming
        let disk_xml = if bus_type == "nvme" {
            // NVMe disks use nvmeXnY naming (e.g., nvme0n1)
            format!(
                r#"<disk type='file' device='disk'>
  <driver name='qemu' type='qcow2'/>
  <source file='{}'/>
  <target dev='{}' bus='nvme'/>
</disk>"#,
                disk_path, device_target
            )
        } else {
            format!(
                r#"<disk type='file' device='disk'>
  <driver name='qemu' type='qcow2'/>
  <source file='{}'/>
  <target dev='{}' bus='{}'/>
</disk>"#,
                disk_path, device_target, bus_type
            )
        };

        // Attach disk (persistent and live if VM is running)
        let flags = if domain.is_active().map_err(map_libvirt_error)? {
            sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain
            .attach_device_flags(&disk_xml, flags)
            .map_err(map_libvirt_error)?;

        tracing::info!("Successfully attached a disk to a VM");
        Ok(())
    }

    /// Detach a disk from a VM
    pub fn detach_disk(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        device_target: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Detaching disk {} from VM {}", device_target, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Get current XML to find the disk
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        let disk_xml =
            first_element_with_descendant_attribute(&xml, "disk", "target", "dev", device_target)?
                .ok_or_else(|| {
                    AppError::InvalidConfig(format!(
                        "Disk with target '{}' not found",
                        device_target
                    ))
                })?;

        // Detach disk (persistent and live if VM is running)
        let flags = if domain.is_active().map_err(map_libvirt_error)? {
            sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain
            .detach_device_flags(&disk_xml, flags)
            .map_err(map_libvirt_error)?;

        tracing::info!(
            "Successfully detached disk {} from VM {}",
            device_target,
            vm_id
        );
        Ok(())
    }

    /// Update disk I/O settings (cache, io mode, discard, throttling)
    #[allow(clippy::too_many_arguments)] // Mirrors the existing Tauri command contract.
    pub fn update_disk_settings(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        device_target: &str,
        cache: Option<String>,
        io: Option<String>,
        discard: Option<String>,
        detect_zeroes: Option<String>,
        read_iops_sec: Option<u64>,
        write_iops_sec: Option<u64>,
        read_bytes_sec: Option<u64>,
        write_bytes_sec: Option<u64>,
    ) -> Result<(), AppError> {
        tracing::info!(
            "Updating disk settings for {} on VM {}",
            device_target,
            vm_id
        );

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Validate cache mode
        if let Some(ref c) = cache {
            let valid_cache_modes = ["none", "writeback", "writethrough", "directsync", "unsafe"];
            if !valid_cache_modes.contains(&c.as_str()) {
                return Err(AppError::InvalidConfig(format!(
                    "Invalid cache mode '{}'. Must be one of: {:?}",
                    c, valid_cache_modes
                )));
            }
        }

        // Validate I/O mode
        if let Some(ref i) = io {
            let valid_io_modes = ["native", "threads", "io_uring"];
            if !valid_io_modes.contains(&i.as_str()) {
                return Err(AppError::InvalidConfig(format!(
                    "Invalid I/O mode '{}'. Must be one of: {:?}",
                    i, valid_io_modes
                )));
            }
        }

        // Validate discard mode
        if let Some(ref d) = discard {
            let valid_discard_modes = ["unmap", "ignore"];
            if !valid_discard_modes.contains(&d.as_str()) {
                return Err(AppError::InvalidConfig(format!(
                    "Invalid discard mode '{}'. Must be one of: {:?}",
                    d, valid_discard_modes
                )));
            }
        }

        // Get current XML
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        let old_disk_xml =
            first_element_with_descendant_attribute(&xml, "disk", "target", "dev", device_target)?
                .ok_or_else(|| {
                    AppError::InvalidConfig(format!(
                        "Disk with target '{}' not found",
                        device_target
                    ))
                })?;
        // Rebuild only the selected disk's owned settings through XML events. The device handed
        // to libvirt retains its source, addressing, and extension elements.
        let updated_xml = rewrite_disk_settings(
            &xml,
            device_target,
            cache.as_deref(),
            io.as_deref(),
            discard.as_deref(),
            detect_zeroes.as_deref(),
            read_iops_sec,
            write_iops_sec,
            read_bytes_sec,
            write_bytes_sec,
        )?;
        let new_disk_xml = first_element_with_descendant_attribute(
            &updated_xml,
            "disk",
            "target",
            "dev",
            device_target,
        )?
        .ok_or_else(|| AppError::InvalidConfig("Updated disk was not found".to_string()))?;

        // VM must be stopped to change disk settings (unless using blkdeviotune for throttling)
        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        if is_running {
            // For a running VM, we can only use blkdeviotune for throttling
            // Cache/IO/discard changes require a restart
            if cache.is_some() || io.is_some() || discard.is_some() || detect_zeroes.is_some() {
                return Err(AppError::InvalidConfig(
                    "Cache, I/O mode, and discard settings can only be changed when the VM is stopped. \
                     I/O throttling (IOPS/bandwidth limits) can be changed on a running VM.".to_string()
                ));
            }

            // Apply I/O throttling to running VM using virsh
            if read_iops_sec.is_some()
                || write_iops_sec.is_some()
                || read_bytes_sec.is_some()
                || write_bytes_sec.is_some()
            {
                let vm_name = domain.get_name().map_err(map_libvirt_error)?;
                let mut args = vec![
                    "blkdeviotune".to_string(),
                    vm_name,
                    device_target.to_string(),
                ];

                if let Some(val) = read_iops_sec {
                    args.push("--read-iops-sec".to_string());
                    args.push(val.to_string());
                }
                if let Some(val) = write_iops_sec {
                    args.push("--write-iops-sec".to_string());
                    args.push(val.to_string());
                }
                if let Some(val) = read_bytes_sec {
                    args.push("--read-bytes-sec".to_string());
                    args.push(val.to_string());
                }
                if let Some(val) = write_bytes_sec {
                    args.push("--write-bytes-sec".to_string());
                    args.push(val.to_string());
                }

                // Add --live and --config flags
                args.push("--live".to_string());
                args.push("--config".to_string());

                let output = Self::virsh_command(conn)?
                    .args(&args)
                    .output()
                    .map_err(|e| AppError::LibvirtError(format!("Failed to run virsh: {}", e)))?;

                if !output.status.success() {
                    return Err(AppError::LibvirtError(
                        "Failed to set I/O throttling".to_string(),
                    ));
                }

                tracing::info!(
                    "Successfully updated I/O throttling for disk {} on VM {}",
                    device_target,
                    vm_id
                );
                return Ok(());
            }
        } else {
            // For a stopped VM, update the persistent config
            // We need to detach and reattach with new settings
            let flags = sys::VIR_DOMAIN_AFFECT_CONFIG;

            // Detach old disk
            domain
                .detach_device_flags(&old_disk_xml, flags)
                .map_err(map_libvirt_error)?;

            // Attach new disk
            domain
                .attach_device_flags(&new_disk_xml, flags)
                .map_err(map_libvirt_error)?;

            tracing::info!(
                "Successfully updated disk settings for {} on VM {}",
                device_target,
                vm_id
            );
        }

        Ok(())
    }

    /// Mount an ISO image as a CDROM to a VM
    pub fn mount_cd_iso(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        iso_path: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Mounting an ISO to a VM");

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Check if ISO file exists
        if !std::path::Path::new(iso_path).exists() {
            return Err(AppError::InvalidConfig(format!(
                "ISO file not found: {}",
                iso_path
            )));
        }

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        // Get current XML to check for existing CDROM
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        let escaped_iso_path = escaped_attribute(iso_path, "ISO path")?;

        // Check if there's already a CDROM device we can update. Selection is event-based and
        // accepts either file/block devices plus either quote style.
        if let Some(existing_cdrom) =
            first_element_with_descendant_attribute(&xml, "disk", "disk", "device", "cdrom")?
        {
            // Read target and bus through parsed attributes, independent of quote style.
            let target_dev = first_element_attribute(&existing_cdrom, "disk", "target", "dev")?
                .unwrap_or_else(|| "hda".to_string());
            let bus_type = first_element_attribute(&existing_cdrom, "disk", "target", "bus")?
                .unwrap_or_else(|| "ide".to_string());

            // Create updated CDROM XML with the ISO
            let updated_cdrom_xml = format!(
                r#"<disk type='file' device='cdrom'>
  <driver name='qemu' type='raw'/>
  <source file='{}'/>
  <target dev='{}' bus='{}'/>
  <readonly/>
</disk>"#,
                escaped_iso_path, target_dev, bus_type
            );

            // Update the device
            let flags = if is_running {
                sys::VIR_DOMAIN_DEVICE_MODIFY_LIVE | sys::VIR_DOMAIN_DEVICE_MODIFY_CONFIG
            } else {
                sys::VIR_DOMAIN_DEVICE_MODIFY_CONFIG
            };

            domain
                .update_device_flags(&updated_cdrom_xml, flags)
                .map_err(map_libvirt_error)?;

            tracing::info!(
                "Successfully mounted ISO to VM {} using existing CDROM",
                vm_id
            );
            return Ok(());
        }

        // No existing CDROM - for running VMs, we need to use virtio-scsi which can be hotplugged
        // First, check if we need to add a SCSI controller
        if is_running {
            // Check if SCSI controller exists
            if !xml.contains("controller type='scsi'") {
                tracing::info!("Adding SCSI controller for hotplug CDROM support");
                let scsi_controller_xml = r#"<controller type='scsi' model='virtio-scsi'/>"#;
                if domain
                    .attach_device_flags(
                        scsi_controller_xml,
                        sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG,
                    )
                    .is_err()
                {
                    tracing::warn!("SCSI controller hotplug did not complete");
                }
            }

            // Try to add a SCSI CDROM (hotpluggable)
            let cdrom_xml = format!(
                r#"<disk type='file' device='cdrom'>
  <driver name='qemu' type='raw'/>
  <source file='{}'/>
  <target dev='sda' bus='scsi'/>
  <readonly/>
</disk>"#,
                escaped_iso_path
            );

            match domain.attach_device_flags(
                &cdrom_xml,
                sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG,
            ) {
                Ok(_) => {
                    tracing::info!("Successfully mounted ISO to VM {} via SCSI hotplug", vm_id);
                    return Ok(());
                }
                Err(_) => {
                    tracing::warn!("SCSI CDROM hotplug failed");
                    return Err(AppError::InvalidConfig(
                        "Cannot hotplug CDROM to this VM. Please stop the VM, mount the ISO, then start it again.".to_string()
                    ));
                }
            }
        }

        // VM is not running - add SATA CDROM (better compatibility)
        let cdrom_xml = format!(
            r#"<disk type='file' device='cdrom'>
  <driver name='qemu' type='raw'/>
  <source file='{}'/>
  <target dev='sda' bus='sata'/>
  <readonly/>
</disk>"#,
            escaped_iso_path
        );

        domain
            .attach_device_flags(&cdrom_xml, sys::VIR_DOMAIN_AFFECT_CONFIG)
            .map_err(map_libvirt_error)?;

        tracing::info!("Successfully mounted ISO to VM {}", vm_id);
        Ok(())
    }

    /// Eject CD/DVD from a VM
    pub fn eject_cd(libvirt: &impl ConnectionProvider, vm_id: &str) -> Result<(), AppError> {
        tracing::info!("Ejecting CD from VM {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Get current XML to find the CDROM device
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        // Select the CD-ROM device structurally rather than relying on serialized attribute order.
        let cdrom_xml =
            first_element_with_descendant_attribute(&xml, "disk", "disk", "device", "cdrom")?
                .ok_or_else(|| AppError::InvalidConfig("No CD ROM device found".to_string()))?;

        // Detach CD (persistent and live if VM is running)
        let flags = if domain.is_active().map_err(map_libvirt_error)? {
            sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain
            .detach_device_flags(&cdrom_xml, flags)
            .map_err(map_libvirt_error)?;

        tracing::info!("Successfully ejected CD from VM {}", vm_id);
        Ok(())
    }

    /// Attach a sound device to a VM
    /// model: ich9, ich6, ac97, es1370, sb16, usb
    pub fn attach_sound(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        model: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Attaching sound device {} to VM {}", model, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Validate sound model
        let valid_models = ["ich9", "ich6", "ac97", "es1370", "sb16", "usb"];
        if !valid_models.contains(&model) {
            return Err(AppError::InvalidConfig(format!(
                "Invalid sound model '{}'. Valid options: {:?}",
                model, valid_models
            )));
        }

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        // Build sound device XML
        // For SPICE connections, we also add an audio element
        let sound_xml = format!(
            r#"<sound model='{}'>
  <audio id='1'/>
</sound>"#,
            model
        );

        let flags = if is_running {
            sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain.attach_device_flags(&sound_xml, flags).map_err(|e| {
            if is_running {
                AppError::InvalidConfig(
                    "Cannot add sound device to running VM. Please stop the VM first.".to_string(),
                )
            } else {
                map_libvirt_error(e)
            }
        })?;

        tracing::info!(
            "Successfully attached sound device {} to VM {}",
            model,
            vm_id
        );
        Ok(())
    }

    /// Detach a sound device from a VM
    pub fn detach_sound(libvirt: &impl ConnectionProvider, vm_id: &str) -> Result<(), AppError> {
        tracing::info!("Detaching sound device from VM {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Get current XML to find the sound device
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        let sound_xml = first_element_fragment(&xml, "sound")?
            .ok_or_else(|| AppError::InvalidConfig("No sound device found".to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        let flags = if is_running {
            sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain.detach_device_flags(&sound_xml, flags).map_err(|e| {
            if is_running {
                AppError::InvalidConfig(
                    "Cannot remove sound device from running VM. Please stop the VM first."
                        .to_string(),
                )
            } else {
                map_libvirt_error(e)
            }
        })?;

        tracing::info!("Successfully detached sound device from VM {}", vm_id);
        Ok(())
    }

    /// Attach an input device to a VM (tablet, mouse, keyboard)
    /// device_type: tablet, mouse, keyboard
    /// bus: usb, virtio, ps2
    pub fn attach_input(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        device_type: &str,
        bus: &str,
    ) -> Result<(), AppError> {
        tracing::info!(
            "Attaching input device {} ({}) to VM {}",
            device_type,
            bus,
            vm_id
        );

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Validate device type
        let valid_types = ["tablet", "mouse", "keyboard"];
        if !valid_types.contains(&device_type) {
            return Err(AppError::InvalidConfig(format!(
                "Invalid input device type '{}'. Valid options: {:?}",
                device_type, valid_types
            )));
        }

        // Validate bus type
        let valid_buses = ["usb", "virtio", "ps2"];
        if !valid_buses.contains(&bus) {
            return Err(AppError::InvalidConfig(format!(
                "Invalid bus type '{}'. Valid options: {:?}",
                bus, valid_buses
            )));
        }

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        // Build input device XML
        let input_xml = format!(r#"<input type='{}' bus='{}'/>"#, device_type, bus);

        let flags = if is_running {
            sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain.attach_device_flags(&input_xml, flags).map_err(|e| {
            if is_running {
                AppError::InvalidConfig(
                    "Cannot add input device to running VM. Please stop the VM first.".to_string(),
                )
            } else {
                map_libvirt_error(e)
            }
        })?;

        tracing::info!(
            "Successfully attached input device {} to VM {}",
            device_type,
            vm_id
        );
        Ok(())
    }

    /// Attach an RNG (Random Number Generator) device to a VM
    pub fn attach_rng(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        backend: &str, // /dev/urandom or /dev/random
    ) -> Result<(), AppError> {
        tracing::info!("Attaching RNG device ({}) to VM {}", backend, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Validate backend
        let valid_backends = ["/dev/urandom", "/dev/random"];
        if !valid_backends.contains(&backend) {
            return Err(AppError::InvalidConfig(format!(
                "Invalid RNG backend '{}'. Valid options: {:?}",
                backend, valid_backends
            )));
        }

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        let rng_xml = format!(
            r#"<rng model='virtio'>
  <backend model='random'>{}</backend>
</rng>"#,
            backend
        );

        let flags = if is_running {
            sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain.attach_device_flags(&rng_xml, flags).map_err(|e| {
            if is_running {
                AppError::InvalidConfig(
                    "Cannot add RNG device to running VM. Please stop the VM first.".to_string(),
                )
            } else {
                map_libvirt_error(e)
            }
        })?;

        tracing::info!("Successfully attached RNG device to VM {}", vm_id);
        Ok(())
    }

    /// Attach a watchdog device to a VM
    /// model: i6300esb, ib700, diag288
    /// action: reset, shutdown, poweroff, pause, none
    pub fn attach_watchdog(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        model: &str,
        action: &str,
    ) -> Result<(), AppError> {
        tracing::info!(
            "Attaching watchdog device {} (action: {}) to VM {}",
            model,
            action,
            vm_id
        );

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Validate model
        let valid_models = ["i6300esb", "ib700", "diag288"];
        if !valid_models.contains(&model) {
            return Err(AppError::InvalidConfig(format!(
                "Invalid watchdog model '{}'. Valid options: {:?}",
                model, valid_models
            )));
        }

        // Validate action
        let valid_actions = ["reset", "shutdown", "poweroff", "pause", "none"];
        if !valid_actions.contains(&action) {
            return Err(AppError::InvalidConfig(format!(
                "Invalid watchdog action '{}'. Valid options: {:?}",
                action, valid_actions
            )));
        }

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        let watchdog_xml = format!(r#"<watchdog model='{}' action='{}'/>"#, model, action);

        let flags = if is_running {
            sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain
            .attach_device_flags(&watchdog_xml, flags)
            .map_err(|e| {
                if is_running {
                    AppError::InvalidConfig(
                        "Cannot add watchdog device to running VM. Please stop the VM first."
                            .to_string(),
                    )
                } else {
                    map_libvirt_error(e)
                }
            })?;

        tracing::info!("Successfully attached watchdog device to VM {}", vm_id);
        Ok(())
    }

    /// Attach a channel device to a VM
    /// channel_type: qemu-ga, spice
    pub fn attach_channel(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        channel_type: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Attaching {} channel to VM {}", channel_type, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        // Generate channel XML based on type
        let channel_xml = match channel_type {
            "qemu-ga" => r#"<channel type='unix'>
  <target type='virtio' name='org.qemu.guest_agent.0'/>
</channel>"#
                .to_string(),
            "kvmmanager-agent" => r#"<channel type='unix'>
  <target type='virtio' name='org.kvmmanager.agent.0'/>
</channel>"#
                .to_string(),
            "spice" => r#"<channel type='spicevmc'>
  <target type='virtio' name='com.redhat.spice.0'/>
</channel>"#
                .to_string(),
            _ => {
                return Err(AppError::InvalidConfig(format!(
                    "Invalid channel type '{}'. Valid options: qemu-ga, kvmmanager-agent, spice",
                    channel_type
                )));
            }
        };

        let flags = if is_running {
            sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain.attach_device_flags(&channel_xml, flags)
            .map_err(|e| {
                if is_running && channel_type == "qemu-ga" {
                    // QEMU GA channel might not work at runtime
                    AppError::InvalidConfig(
                        "Cannot add QEMU Guest Agent channel to running VM. Please stop the VM first.".to_string()
                    )
                } else {
                    map_libvirt_error(e)
                }
            })?;

        tracing::info!(
            "Successfully attached {} channel to VM {}",
            channel_type,
            vm_id
        );
        Ok(())
    }

    /// Attach a filesystem (folder share) to a VM
    /// fs_type: virtio-9p, virtiofs
    pub fn attach_filesystem(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        source_path: &str,
        target_mount: &str,
        fs_type: &str,
        readonly: bool,
    ) -> Result<(), AppError> {
        tracing::info!(
            "Attaching filesystem {} -> {} ({}) to VM {}",
            source_path,
            target_mount,
            fs_type,
            vm_id
        );

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Validate source path exists
        if !std::path::Path::new(source_path).exists() {
            return Err(AppError::InvalidConfig(format!(
                "Source path '{}' does not exist",
                source_path
            )));
        }

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        let readonly_attr = if readonly { " readonly='yes'" } else { "" };

        // Generate filesystem XML based on type
        let fs_xml = match fs_type {
            "virtio-9p" => {
                format!(
                    r#"<filesystem type='mount' accessmode='mapped'>
  <source dir='{}'/>
  <target dir='{}'/>
</filesystem>"#,
                    source_path, target_mount
                )
            }
            "virtiofs" => {
                // virtiofs requires a socket
                format!(
                    r#"<filesystem type='mount'>
  <driver type='virtiofs'{} queue='1024'/>
  <source dir='{}'/>
  <target dir='{}'/>
</filesystem>"#,
                    readonly_attr, source_path, target_mount
                )
            }
            _ => {
                return Err(AppError::InvalidConfig(format!(
                    "Invalid filesystem type '{}'. Valid options: virtio-9p, virtiofs",
                    fs_type
                )));
            }
        };

        let flags = if is_running {
            sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain.attach_device_flags(&fs_xml, flags).map_err(|e| {
            if is_running {
                AppError::InvalidConfig(
                    "Cannot add filesystem to running VM. Please stop the VM first.".to_string(),
                )
            } else {
                map_libvirt_error(e)
            }
        })?;

        tracing::info!(
            "Successfully attached filesystem {} to VM {}",
            target_mount,
            vm_id
        );
        Ok(())
    }

    /// Attach a serial port to a VM
    /// port_type: pty, tcp, unix, file
    pub fn attach_serial(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        port_type: &str,
        target_port: u32,
    ) -> Result<(), AppError> {
        tracing::info!("Attaching serial port ({}) to VM {}", port_type, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        // Generate serial XML based on type
        let serial_xml = match port_type {
            "pty" => format!(
                r#"<serial type='pty'>
  <target port='{}'/>
</serial>"#,
                target_port
            ),
            "tcp" => format!(
                r#"<serial type='tcp'>
  <source mode='bind' host='127.0.0.1' service='0'/>
  <protocol type='raw'/>
  <target port='{}'/>
</serial>"#,
                target_port
            ),
            "unix" => format!(
                r#"<serial type='unix'>
  <source mode='bind'/>
  <target port='{}'/>
</serial>"#,
                target_port
            ),
            _ => {
                return Err(AppError::InvalidConfig(format!(
                    "Invalid serial port type '{}'. Valid options: pty, tcp, unix",
                    port_type
                )));
            }
        };

        let flags = if is_running {
            sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain
            .attach_device_flags(&serial_xml, flags)
            .map_err(|e| {
                if is_running {
                    AppError::InvalidConfig(
                        "Cannot add serial port to running VM. Please stop the VM first."
                            .to_string(),
                    )
                } else {
                    map_libvirt_error(e)
                }
            })?;

        tracing::info!(
            "Successfully attached serial port {} to VM {}",
            target_port,
            vm_id
        );
        Ok(())
    }

    /// Attach a virtio console device to a VM
    pub fn attach_console(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        target_port: u32,
        target_type: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Attaching console device to VM {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        // Generate console XML
        let console_xml = match target_type {
            "virtio" => format!(
                r#"<console type='pty'>
  <target type='virtio' port='{}'/>
</console>"#,
                target_port
            ),
            "serial" => format!(
                r#"<console type='pty'>
  <target type='serial' port='{}'/>
</console>"#,
                target_port
            ),
            _ => {
                return Err(AppError::InvalidConfig(format!(
                    "Invalid console target type '{}'. Valid options: virtio, serial",
                    target_type
                )));
            }
        };

        let flags = if is_running {
            sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain
            .attach_device_flags(&console_xml, flags)
            .map_err(|e| {
                if is_running {
                    AppError::InvalidConfig(
                        "Cannot add console device to running VM. Please stop the VM first."
                            .to_string(),
                    )
                } else {
                    map_libvirt_error(e)
                }
            })?;

        tracing::info!("Successfully attached console device to VM {}", vm_id);
        Ok(())
    }

    /// Attach a TPM device to an existing VM
    /// model: tpm-tis, tpm-crb
    /// version: 1.2, 2.0
    pub fn attach_tpm(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        model: &str,
        version: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Attaching TPM ({} v{}) to VM {}", model, version, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        if is_running {
            return Err(AppError::InvalidConfig(
                "Cannot add TPM to running VM. Please stop the VM first.".to_string(),
            ));
        }

        // Validate model
        let valid_models = ["tpm-tis", "tpm-crb"];
        if !valid_models.contains(&model) {
            return Err(AppError::InvalidConfig(format!(
                "Invalid TPM model '{}'. Valid options: {:?}",
                model, valid_models
            )));
        }

        // Validate version
        let valid_versions = ["1.2", "2.0"];
        if !valid_versions.contains(&version) {
            return Err(AppError::InvalidConfig(format!(
                "Invalid TPM version '{}'. Valid options: {:?}",
                version, valid_versions
            )));
        }

        let tpm_xml = format!(
            r#"<tpm model='{}'>
  <backend type='emulator' version='{}'/>
</tpm>"#,
            model, version
        );

        domain
            .attach_device_flags(&tpm_xml, sys::VIR_DOMAIN_AFFECT_CONFIG)
            .map_err(map_libvirt_error)?;

        tracing::info!("Successfully attached TPM to VM {}", vm_id);
        Ok(())
    }

    /// Attach a USB controller to a VM
    /// model: piix3-uhci (USB 1.1), ich9-ehci1 (USB 2.0), nec-xhci (USB 3.0), qemu-xhci (USB 3.0)
    pub fn attach_usb_controller(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        model: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Attaching USB controller ({}) to VM {}", model, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        if is_running {
            return Err(AppError::InvalidConfig(
                "Cannot add USB controller to running VM. Please stop the VM first.".to_string(),
            ));
        }

        // Validate model and determine type
        let (controller_type, model_name) = match model {
            "piix3-uhci" => ("usb", "piix3-uhci"),
            "ich9-uhci1" => ("usb", "ich9-uhci1"),
            "ich9-uhci2" => ("usb", "ich9-uhci2"),
            "ich9-uhci3" => ("usb", "ich9-uhci3"),
            "ich9-ehci1" => ("usb", "ich9-ehci1"),
            "nec-xhci" => ("usb", "nec-xhci"),
            "qemu-xhci" => ("usb", "qemu-xhci"),
            _ => {
                return Err(AppError::InvalidConfig(format!(
                    "Invalid USB controller model '{}'. Valid options: piix3-uhci, ich9-uhci1, ich9-ehci1, nec-xhci, qemu-xhci",
                    model
                )));
            }
        };

        let controller_xml = format!(
            r#"<controller type='{}' model='{}'/>
"#,
            controller_type, model_name
        );

        domain
            .attach_device_flags(&controller_xml, sys::VIR_DOMAIN_AFFECT_CONFIG)
            .map_err(map_libvirt_error)?;

        tracing::info!("Successfully attached USB controller to VM {}", vm_id);
        Ok(())
    }

    /// Attach USB redirection channels to a VM for SPICE USB passthrough
    /// This adds redirdev elements that SPICE clients can use to redirect USB devices
    /// count: number of USB redirection channels (1-4, default 4)
    pub fn attach_usb_redirection(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        count: u32,
    ) -> Result<(), AppError> {
        let count = count.clamp(1, 4);
        tracing::info!(
            "Attaching {} USB redirection channel(s) to VM {}",
            count,
            vm_id
        );

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        if is_running {
            return Err(AppError::InvalidConfig(
                "Cannot add USB redirection to running VM. Please stop the VM first.".to_string(),
            ));
        }

        // Check if the VM has SPICE graphics
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;
        if !xml.contains("type='spice'") {
            return Err(AppError::InvalidConfig(
                "USB redirection requires SPICE graphics. Please add SPICE graphics first."
                    .to_string(),
            ));
        }

        // Add USB redirection channels
        let mut complete = true;
        for i in 0..count {
            let redir_xml = format!(
                r#"<redirdev bus='usb' type='spicevmc'>
  <alias name='redir{}'/>
</redirdev>"#,
                i
            );

            match domain.attach_device_flags(&redir_xml, sys::VIR_DOMAIN_AFFECT_CONFIG) {
                Ok(_) => {
                    tracing::info!("Added USB redirection channel {} to VM {}", i, vm_id);
                }
                Err(_) => {
                    complete = false;
                    tracing::warn!("A USB redirection channel could not be added");
                }
            }
        }

        if !complete {
            return Err(AppError::Partial(
                "USB redirection was only partially configured".to_string(),
            ));
        }

        tracing::info!(
            "Successfully attached {} USB redirection channel(s) to VM {}",
            count,
            vm_id
        );
        Ok(())
    }

    /// Get USB redirection configuration for a VM
    pub fn get_usb_redirection(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
    ) -> Result<UsbRedirectionInfo, AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        // Count redirdev elements with type='spicevmc'
        let count = xml.matches("<redirdev bus='usb' type='spicevmc'").count() as u32;
        let has_spice = xml.contains("type='spice'");

        Ok(UsbRedirectionInfo {
            enabled: count > 0,
            channel_count: count,
            has_spice_graphics: has_spice,
        })
    }

    /// Remove all USB redirection channels from a VM
    pub fn remove_usb_redirection(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Removing USB redirection channels from VM {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        if is_running {
            return Err(AppError::InvalidConfig(
                "Cannot remove USB redirection from running VM. Please stop the VM first."
                    .to_string(),
            ));
        }

        let mut new_xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        // Remove complete device events rather than slicing a definition. This accepts either
        // element form and retains all unrelated devices and extension namespaces.
        while first_element_fragment(&new_xml, "redirdev")?.is_some() {
            new_xml = remove_first_element(&new_xml, "redirdev")?;
        }

        // Redefine the domain with updated XML
        Domain::define_xml(conn, &new_xml)
            .map_err(|e| AppError::Other(format!("Failed to remove USB redirection: {}", e)))?;

        tracing::info!(
            "Successfully removed USB redirection channels from VM {}",
            vm_id
        );
        Ok(())
    }

    /// Attach a SCSI controller to a VM
    /// model: virtio-scsi, lsilogic, lsisas1068, megasas
    pub fn attach_scsi_controller(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        model: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Attaching SCSI controller ({}) to VM {}", model, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        if is_running {
            return Err(AppError::InvalidConfig(
                "Cannot add SCSI controller to running VM. Please stop the VM first.".to_string(),
            ));
        }

        // Validate model
        let valid_models = [
            "virtio-scsi",
            "lsilogic",
            "lsisas1068",
            "lsisas1078",
            "megasas",
        ];
        if !valid_models.contains(&model) {
            return Err(AppError::InvalidConfig(format!(
                "Invalid SCSI controller model '{}'. Valid options: {:?}",
                model, valid_models
            )));
        }

        let controller_xml = format!(
            r#"<controller type='scsi' model='{}'/>
"#,
            model
        );

        domain
            .attach_device_flags(&controller_xml, sys::VIR_DOMAIN_AFFECT_CONFIG)
            .map_err(map_libvirt_error)?;

        tracing::info!("Successfully attached SCSI controller to VM {}", vm_id);
        Ok(())
    }

    /// Update VM boot order
    pub fn update_boot_order(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        boot_order: Vec<String>,
    ) -> Result<(), AppError> {
        tracing::info!("Updating boot order for VM {} to: {:?}", vm_id, boot_order);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Check if VM is running
        let is_running = domain.is_active().map_err(map_libvirt_error)?;
        if is_running {
            return Err(AppError::InvalidConfig(
                "Cannot change boot order while VM is running. Please stop the VM first."
                    .to_string(),
            ));
        }

        // Get current XML
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        let boot_devices = boot_order.iter().map(String::as_str).collect::<Vec<_>>();
        let new_xml = rewrite_domain_boot_order(&xml, &boot_devices)?;

        // Undefine and redefine domain with new XML
        domain.undefine().map_err(map_libvirt_error)?;
        Domain::define_xml(conn, &new_xml).map_err(map_libvirt_error)?;

        tracing::info!("Successfully updated boot order for VM {}", vm_id);
        Ok(())
    }

    /// Rename a VM (change domain name)
    pub fn rename_vm(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        new_name: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Renaming VM {} to {}", vm_id, new_name);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Check if VM is running
        let is_running = domain.is_active().map_err(map_libvirt_error)?;
        if is_running {
            return Err(AppError::InvalidConfig(
                "Cannot rename a running VM. Please stop the VM first.".to_string(),
            ));
        }

        // Validate new name (basic validation)
        if new_name.is_empty() {
            return Err(AppError::InvalidConfig(
                "VM name cannot be empty".to_string(),
            ));
        }

        if new_name.contains(' ') || new_name.contains('/') || new_name.contains('\\') {
            return Err(AppError::InvalidConfig(
                "VM name cannot contain spaces or slashes".to_string(),
            ));
        }

        // Check if name already exists
        if let Ok(_existing) = Domain::lookup_by_name(conn, new_name) {
            return Err(AppError::InvalidConfig(format!(
                "A VM with name '{}' already exists",
                new_name
            )));
        }

        // Get current XML
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        // Extract old name for disk renaming
        let old_name = domain.get_name().map_err(map_libvirt_error)?;

        let new_xml = rewrite_first_text_element(&xml, "name", new_name)?;

        // Undefine the old domain
        domain.undefine().map_err(map_libvirt_error)?;

        // Define domain with new XML (new name)
        let _new_domain = Domain::define_xml(conn, &new_xml).map_err(map_libvirt_error)?;

        // Optionally rename disk files (best effort, don't fail if it doesn't work)
        // This is a convenience feature - we'll try to rename disk files to match the new VM name
        let disk_paths = Self::get_vm_disk_paths(
            &Domain::lookup_by_name(conn, new_name).map_err(map_libvirt_error)?,
        )?;
        for disk_path in disk_paths {
            if disk_path.contains(&old_name) {
                let new_disk_path = disk_path.replace(&old_name, new_name);
                if let Err(_error) = std::fs::rename(&disk_path, &new_disk_path) {
                    tracing::warn!(
                        "Failed to rename a VM disk file; manual reconciliation may be required"
                    );
                } else {
                    tracing::info!("Renamed a VM disk file");

                    // Update XML with new disk path
                    let updated_domain =
                        Domain::lookup_by_name(conn, new_name).map_err(map_libvirt_error)?;
                    let updated_xml = updated_domain.get_xml_desc(0).map_err(map_libvirt_error)?;
                    let final_xml = crate::utils::xml::rewrite_disk_source_path(
                        &updated_xml,
                        &disk_path,
                        &new_disk_path,
                    )?;

                    updated_domain.undefine().map_err(map_libvirt_error)?;
                    Domain::define_xml(conn, &final_xml).map_err(map_libvirt_error)?;
                }
            }
        }

        tracing::info!("Successfully renamed VM from {} to {}", old_name, new_name);
        Ok(())
    }

    /// Get VM autostart status
    pub fn get_vm_autostart(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
    ) -> Result<bool, AppError> {
        tracing::debug!("Getting autostart for VM: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let autostart = domain.get_autostart().map_err(map_libvirt_error)?;

        Ok(autostart)
    }

    /// Set VM autostart status
    pub fn set_vm_autostart(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        enable: bool,
    ) -> Result<(), AppError> {
        tracing::info!("Setting autostart for VM {}: {}", vm_id, enable);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        domain.set_autostart(enable).map_err(map_libvirt_error)?;

        tracing::info!("Autostart set to {} for VM: {}", enable, vm_id);
        Ok(())
    }

    /// Attach a network interface to a VM
    pub fn attach_interface(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        network: &str,
        model: &str,
        mac_address: Option<&str>,
    ) -> Result<String, AppError> {
        tracing::info!(
            "Attaching network interface to VM {} on network {}",
            vm_id,
            network
        );

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Validate model
        let valid_models = ["virtio", "e1000", "e1000e", "rtl8139", "vmxnet3"];
        if !valid_models.contains(&model) {
            return Err(AppError::InvalidConfig(format!(
                "Invalid model '{}'. Must be one of: {:?}",
                model, valid_models
            )));
        }

        // Generate MAC address if not provided
        let mac = mac_address.map(String::from).unwrap_or_else(|| {
            // Generate random MAC with the common QEMU OUI prefix 52:54:00
            use std::time::{SystemTime, UNIX_EPOCH};
            let t = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            format!(
                "52:54:00:{:02x}:{:02x}:{:02x}",
                ((t >> 16) & 0xff) as u8,
                ((t >> 8) & 0xff) as u8,
                (t & 0xff) as u8
            )
        });

        // Build interface XML
        let interface_xml = format!(
            r#"<interface type='network'>
  <mac address='{}'/>
  <source network='{}'/>
  <model type='{}'/>
</interface>"#,
            mac, network, model
        );

        // Attach interface (persistent and live if VM is running)
        let flags = if domain.is_active().map_err(map_libvirt_error)? {
            sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain
            .attach_device_flags(&interface_xml, flags)
            .map_err(map_libvirt_error)?;

        tracing::info!(
            "Successfully attached network interface to VM {} with MAC {}",
            vm_id,
            mac
        );
        Ok(mac)
    }

    /// Attach a network interface to a VM with advanced options
    /// Supports: network (NAT/isolated), bridge, direct (macvtap), and OVS
    #[allow(clippy::too_many_arguments)] // Mirrors the existing Tauri command contract.
    pub fn attach_interface_advanced(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        interface_type: &str, // "network", "bridge", "direct", "ovs"
        source: &str,         // network name, bridge name, or host interface
        model: &str,
        mac_address: Option<&str>,
        source_mode: Option<&str>, // For direct: bridge, vepa, private, passthrough
        vlan_id: Option<u16>,
        portgroup: Option<&str>,
        mtu: Option<u32>,
    ) -> Result<String, AppError> {
        tracing::info!(
            "Attaching {} interface to VM {} source={}",
            interface_type,
            vm_id,
            source
        );

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Validate model
        let valid_models = ["virtio", "e1000", "e1000e", "rtl8139", "vmxnet3"];
        if !valid_models.contains(&model) {
            return Err(AppError::InvalidConfig(format!(
                "Invalid model '{}'. Must be one of: {:?}",
                model, valid_models
            )));
        }

        // Generate MAC address if not provided
        let mac = mac_address.map(String::from).unwrap_or_else(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let t = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            format!(
                "52:54:00:{:02x}:{:02x}:{:02x}",
                ((t >> 16) & 0xff) as u8,
                ((t >> 8) & 0xff) as u8,
                (t & 0xff) as u8
            )
        });

        let mtu_xml = mtu
            .map(|size| format!("\n  <mtu size='{size}'/>"))
            .unwrap_or_default();
        let safe_mac = escaped_attribute(&mac, "MAC address")?;
        let safe_source = escaped_attribute(source, "Interface source")?;
        let safe_model = escaped_attribute(model, "Interface model")?;

        // Build interface XML based on type.
        let interface_xml = match interface_type {
            "network" => {
                let pg_attr = portgroup
                    .map(|value| {
                        escaped_attribute(value, "Network port group")
                            .map(|escaped| format!(" portgroup='{escaped}'"))
                    })
                    .transpose()?
                    .unwrap_or_default();
                format!(
                    r#"<interface type='network'>
  <mac address='{}'/>
  <source network='{}'{}/>
  <model type='{}'/>{}
</interface>"#,
                    safe_mac, safe_source, pg_attr, safe_model, mtu_xml
                )
            }
            "bridge" => {
                format!(
                    r#"<interface type='bridge'>
  <mac address='{}'/>
  <source bridge='{}'/>
  <model type='{}'/>{}
</interface>"#,
                    safe_mac, safe_source, safe_model, mtu_xml
                )
            }
            "direct" => {
                // macvtap direct attachment to host interface
                let mode = source_mode.unwrap_or("bridge");
                let valid_modes = ["vepa", "bridge", "private", "passthrough"];
                if !valid_modes.contains(&mode) {
                    return Err(AppError::InvalidConfig(format!(
                        "Invalid direct mode '{}'. Must be one of: {:?}",
                        mode, valid_modes
                    )));
                }
                format!(
                    r#"<interface type='direct'>
  <mac address='{}'/>
  <source dev='{}' mode='{}'/>
  <model type='{}'/>{}
</interface>"#,
                    safe_mac,
                    safe_source,
                    escaped_attribute(mode, "Direct interface mode")?,
                    safe_model,
                    mtu_xml
                )
            }
            "ovs" => {
                // Open vSwitch bridge
                let vlan_xml = vlan_id
                    .map(|v| format!("\n  <vlan>\n    <tag id='{}'/>\n  </vlan>", v))
                    .unwrap_or_default();
                format!(
                    r#"<interface type='bridge'>
  <mac address='{}'/>
  <source bridge='{}'/>
  <virtualport type='openvswitch'/>
  <model type='{}'/>{}{}
</interface>"#,
                    safe_mac, safe_source, safe_model, vlan_xml, mtu_xml
                )
            }
            _ => {
                return Err(AppError::InvalidConfig(format!(
                    "Invalid interface type '{}'. Must be one of: network, bridge, direct, ovs",
                    interface_type
                )));
            }
        };

        // Attach interface (persistent and live if VM is running)
        let flags = if domain.is_active().map_err(map_libvirt_error)? {
            sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain
            .attach_device_flags(&interface_xml, flags)
            .map_err(map_libvirt_error)?;

        tracing::info!(
            "Successfully attached {} interface to VM {} with MAC {}",
            interface_type,
            vm_id,
            mac
        );
        Ok(mac)
    }

    /// List host network interfaces available for direct (macvtap) attachment
    pub fn list_host_interfaces() -> Result<Vec<HostNetworkInterface>, AppError> {
        use std::fs;

        let mut interfaces = Vec::new();

        // Read from /sys/class/net/
        let net_path = std::path::Path::new("/sys/class/net");
        if !net_path.exists() {
            return Ok(interfaces);
        }

        for entry in fs::read_dir(net_path)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip virtual interfaces (lo, virbr*, veth*, docker*, etc.)
            if name == "lo"
                || name.starts_with("virbr")
                || name.starts_with("veth")
                || name.starts_with("docker")
                || name.starts_with("br-")
                || name.starts_with("vnet")
            {
                continue;
            }

            let iface_path = entry.path();

            // Read operstate
            let state = fs::read_to_string(iface_path.join("operstate"))
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "unknown".to_string());

            // Read MAC address
            let mac = fs::read_to_string(iface_path.join("address"))
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "".to_string());

            // Read speed if available
            let speed = fs::read_to_string(iface_path.join("speed"))
                .ok()
                .and_then(|s| s.trim().parse::<u32>().ok());

            // Read MTU
            let mtu = fs::read_to_string(iface_path.join("mtu"))
                .ok()
                .and_then(|s| s.trim().parse::<u32>().ok());

            // Check if it's a physical interface (has device symlink)
            let is_physical = iface_path.join("device").exists();

            // Get driver name
            let driver = fs::read_link(iface_path.join("device/driver"))
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()));

            interfaces.push(HostNetworkInterface {
                name: name.clone(),
                mac_address: mac,
                state,
                speed,
                mtu,
                is_physical,
                driver,
            });
        }

        // Sort physical interfaces first
        interfaces.sort_by(|a, b| match (a.is_physical, b.is_physical) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        Ok(interfaces)
    }

    /// Detach a network interface from a VM by MAC address
    pub fn detach_interface(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        mac_address: &str,
    ) -> Result<(), AppError> {
        tracing::info!(
            "Detaching network interface with MAC {} from VM {}",
            mac_address,
            vm_id
        );

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Get current XML to find the interface
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        let interface_xml = first_element_with_descendant_attribute(
            &xml,
            "interface",
            "mac",
            "address",
            &mac_address.to_lowercase(),
        )?
        .ok_or_else(|| {
            AppError::InvalidConfig(format!("Interface with MAC '{}' not found", mac_address))
        })?;

        // Detach interface (persistent and live if VM is running)
        let flags = if domain.is_active().map_err(map_libvirt_error)? {
            sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain
            .detach_device_flags(&interface_xml, flags)
            .map_err(map_libvirt_error)?;

        tracing::info!(
            "Successfully detached interface with MAC {} from VM {}",
            mac_address,
            vm_id
        );
        Ok(())
    }

    /// Attach a graphics device to a VM (VNC, Spice)
    /// graphics_type: vnc, spice
    pub fn attach_graphics(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        graphics_type: &str,
        listen_address: Option<&str>,
        port: Option<i32>,
    ) -> Result<(), AppError> {
        tracing::info!("Attaching {} graphics to VM {}", graphics_type, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Validate graphics type
        let valid_types = ["vnc", "spice"];
        if !valid_types.contains(&graphics_type) {
            return Err(AppError::InvalidConfig(format!(
                "Invalid graphics type '{}'. Valid options: {:?}",
                graphics_type, valid_types
            )));
        }

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        let listen = listen_address.unwrap_or("127.0.0.1");
        let port_attr = match port {
            Some(p) if p > 0 => format!("port='{}'", p),
            _ => "port='-1' autoport='yes'".to_string(),
        };

        let graphics_xml = match graphics_type {
            "spice" => format!(
                r#"<graphics type='spice' {} listen='{}'>
  <listen type='address' address='{}'/>
  <image compression='auto_glz'/>
  <streaming mode='filter'/>
  <clipboard copypaste='yes'/>
  <mouse mode='client'/>
</graphics>"#,
                port_attr, listen, listen
            ),
            _ => format!(
                r#"<graphics type='vnc' {} listen='{}'>
  <listen type='address' address='{}'/>
</graphics>"#,
                port_attr, listen, listen
            ),
        };

        let flags = if is_running {
            sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain
            .attach_device_flags(&graphics_xml, flags)
            .map_err(|e| {
                if is_running {
                    AppError::InvalidConfig(
                        "Cannot add graphics device to running VM. Please stop the VM first."
                            .to_string(),
                    )
                } else {
                    map_libvirt_error(e)
                }
            })?;

        tracing::info!(
            "Successfully attached {} graphics to VM {}",
            graphics_type,
            vm_id
        );
        Ok(())
    }

    /// Attach a video device to a VM
    /// model: virtio, qxl, vga, bochs, ramfb, cirrus
    pub fn attach_video(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        model: &str,
        vram: Option<u32>,
        heads: Option<u32>,
        acceleration_3d: bool,
    ) -> Result<(), AppError> {
        tracing::info!("Attaching {} video device to VM {}", model, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Validate model
        let valid_models = ["virtio", "qxl", "vga", "bochs", "ramfb", "cirrus"];
        if !valid_models.contains(&model) {
            return Err(AppError::InvalidConfig(format!(
                "Invalid video model '{}'. Valid options: {:?}",
                model, valid_models
            )));
        }

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        // Build video XML based on model
        let vram_kb = vram.unwrap_or(65536); // Default 64MB
        let heads_count = heads.unwrap_or(1);

        let video_xml = match model {
            "virtio" => {
                let accel = if acceleration_3d {
                    " accel3d='yes'"
                } else {
                    ""
                };
                format!(
                    r#"<video>
  <model type='virtio' heads='{}' primary='yes'{}>
    <acceleration accel3d='{}'/>
  </model>
</video>"#,
                    heads_count,
                    accel,
                    if acceleration_3d { "yes" } else { "no" }
                )
            }
            "qxl" => format!(
                r#"<video>
  <model type='qxl' ram='{}' vram='{}' vgamem='16384' heads='{}' primary='yes'/>
</video>"#,
                vram_kb * 2,
                vram_kb,
                heads_count
            ),
            "vga" | "bochs" | "cirrus" => format!(
                r#"<video>
  <model type='{}' vram='{}' heads='{}' primary='yes'/>
</video>"#,
                model, vram_kb, heads_count
            ),
            "ramfb" => r#"<video>
  <model type='ramfb' primary='yes'/>
</video>"#
                .to_string(),
            _ => format!(
                r#"<video>
  <model type='{}' vram='{}' heads='{}' primary='yes'/>
</video>"#,
                model, vram_kb, heads_count
            ),
        };

        let flags = if is_running {
            sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain.attach_device_flags(&video_xml, flags).map_err(|e| {
            if is_running {
                AppError::InvalidConfig(
                    "Cannot add video device to running VM. Please stop the VM first.".to_string(),
                )
            } else {
                map_libvirt_error(e)
            }
        })?;

        tracing::info!(
            "Successfully attached {} video device to VM {}",
            model,
            vm_id
        );
        Ok(())
    }

    /// Set the vCPU count for a VM
    pub fn set_vcpus(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        vcpus: u32,
    ) -> Result<(), AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let info = domain.get_info().map_err(map_libvirt_error)?;
        let is_running = info.state == sys::VIR_DOMAIN_RUNNING;

        if is_running {
            // For running VMs, we can only set vCPUs up to the maximum
            // First try to set current vCPUs
            domain
                .set_vcpus_flags(
                    vcpus,
                    sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG,
                )
                .map_err(|e| {
                    AppError::InvalidConfig(format!(
                        "Cannot change vCPUs while VM is running: {}. Stop the VM first.",
                        e
                    ))
                })?;
        } else {
            // For stopped VMs, update the domain XML
            let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;
            let new_xml =
                crate::utils::xml::rewrite_first_text_element(&xml, "vcpu", &vcpus.to_string())?;

            // Redefine the domain with updated XML
            Domain::define_xml(conn, &new_xml)
                .map_err(|e| AppError::Other(format!("Failed to update VM vCPUs: {}", e)))?;
        }

        tracing::info!("Successfully set vCPUs to {} for VM {}", vcpus, vm_id);
        Ok(())
    }

    /// Set the memory allocation for a VM (in MB)
    pub fn set_memory(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        memory_mb: u64,
    ) -> Result<(), AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let info = domain.get_info().map_err(map_libvirt_error)?;
        let is_running = info.state == sys::VIR_DOMAIN_RUNNING;

        let memory_kb = memory_mb * 1024;

        if is_running {
            // For running VMs, try to set memory (requires memory ballooning)
            domain.set_memory_flags(memory_kb, sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG)
                .map_err(|e| AppError::InvalidConfig(format!(
                    "Cannot change memory while VM is running: {}. Stop the VM first or enable memory ballooning.", e
                )))?;
        } else {
            // For stopped VMs, update the domain XML
            let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

            let memory_unit =
                crate::utils::xml::first_element_attribute(&xml, "domain", "memory", "unit")?
                    .unwrap_or_else(|| "KiB".to_string());
            let memory_value = match memory_unit.as_str() {
                "KiB" => memory_kb,
                "MiB" => memory_mb,
                "GiB" if memory_mb.is_multiple_of(1024) => memory_mb / 1024,
                _ => {
                    return Err(AppError::InvalidConfig(
                        "VM memory unit is not supported".to_string(),
                    ))
                }
            };
            let new_xml = rewrite_domain_memory(&xml, &memory_value.to_string())?;

            // Redefine the domain with updated XML
            Domain::define_xml(conn, &new_xml)
                .map_err(|e| AppError::Other(format!("Failed to update VM memory: {}", e)))?;
        }

        tracing::info!(
            "Successfully set memory to {} MB for VM {}",
            memory_mb,
            vm_id
        );
        Ok(())
    }

    /// Set the CPU topology for a VM (sockets, cores, threads)
    pub fn set_cpu_topology(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        sockets: u32,
        cores: u32,
        threads: u32,
    ) -> Result<(), AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let info = domain.get_info().map_err(map_libvirt_error)?;
        let is_running = info.state == sys::VIR_DOMAIN_RUNNING;

        if is_running {
            return Err(AppError::InvalidConfig(
                "Cannot change CPU topology while VM is running. Stop the VM first.".to_string(),
            ));
        }

        let total_vcpus = sockets
            .checked_mul(cores)
            .and_then(|value| value.checked_mul(threads))
            .ok_or_else(|| AppError::InvalidConfig("CPU topology is too large".to_string()))?;
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;
        let new_xml = rewrite_first_text_element(&xml, "vcpu", &total_vcpus.to_string())?;
        let new_xml = rewrite_cpu_topology(&new_xml, sockets, cores, threads)?;

        // Redefine the domain with updated XML
        Domain::define_xml(conn, &new_xml)
            .map_err(|e| AppError::Other(format!("Failed to update CPU topology: {}", e)))?;

        tracing::info!(
            "Successfully set CPU topology to {}s/{}c/{}t for VM {}",
            sockets,
            cores,
            threads,
            vm_id
        );
        Ok(())
    }

    /// Get the current CPU model configuration for a VM
    pub fn get_cpu_model(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
    ) -> Result<CpuModelConfig, AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        let mode = first_element_attribute(&xml, "domain", "cpu", "mode")?
            .unwrap_or_else(|| "host-passthrough".to_string());

        // Extract CPU model name (for custom mode)
        let model = if mode == "custom" {
            first_element_text(&xml, "domain", "model")?.map(|value| value.trim().to_string())
        } else {
            None
        };

        Ok(CpuModelConfig { mode, model })
    }

    /// Set the CPU model for a VM
    pub fn set_cpu_model(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        mode: &str,
        model: Option<&str>,
    ) -> Result<(), AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let info = domain.get_info().map_err(map_libvirt_error)?;
        if info.state == sys::VIR_DOMAIN_RUNNING {
            return Err(AppError::InvalidConfig(
                "Cannot change CPU model while VM is running. Stop the VM first.".to_string(),
            ));
        }

        let mut xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        if first_element_fragment(&xml, "cpu")?.is_some() {
            xml = rewrite_cpu_model(&xml, mode, model)?;
        } else {
            xml = insert_cpu_after_vcpu(&xml, mode, model)?;
        }

        // Redefine the domain
        Domain::define_xml(conn, &xml)
            .map_err(|e| AppError::Other(format!("Failed to update CPU model: {}", e)))?;

        tracing::info!("Successfully set CPU mode to {} for VM {}", mode, vm_id);
        Ok(())
    }

    /// Get available CPU models from libvirt capabilities
    pub fn get_available_cpu_models(
        _libvirt: &impl ConnectionProvider,
    ) -> Result<Vec<String>, AppError> {
        // Try to get CPU models from capabilities
        let output = Self::virsh_command(_libvirt.get_connection())?
            .args(["cpu-models", "x86_64"])
            .output()
            .map_err(|e| AppError::Other(format!("Failed to run virsh: {}", e)))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let models: Vec<String> = stdout
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|s| s.trim().to_string())
                .collect();
            return Ok(models);
        }

        // Fallback to common CPU models
        Ok(vec![
            "qemu64".to_string(),
            "qemu32".to_string(),
            "Nehalem".to_string(),
            "Westmere".to_string(),
            "SandyBridge".to_string(),
            "IvyBridge".to_string(),
            "Haswell".to_string(),
            "Broadwell".to_string(),
            "Skylake-Client".to_string(),
            "Skylake-Server".to_string(),
            "Cascadelake-Server".to_string(),
            "Icelake-Client".to_string(),
            "Icelake-Server".to_string(),
            "Cooperlake".to_string(),
            "EPYC".to_string(),
            "EPYC-Rome".to_string(),
            "EPYC-Milan".to_string(),
        ])
    }

    /// Get current CPU pinning configuration for a VM
    pub fn get_cpu_pinning(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
    ) -> Result<Vec<(u32, Vec<u32>)>, AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;
        let mut pinnings = Vec::new();

        // Parse cputune/vcpupin elements from XML
        // <cputune>
        //   <vcpupin vcpu='0' cpuset='0-1'/>
        //   <vcpupin vcpu='1' cpuset='2-3'/>
        // </cputune>
        if let Some(cputune_start) = xml.find("<cputune>") {
            if let Some(cputune_end) = xml[cputune_start..].find("</cputune>") {
                let cputune_section = &xml[cputune_start..cputune_start + cputune_end];

                // Find all vcpupin elements
                let mut search_start = 0;
                while let Some(pin_start) = cputune_section[search_start..].find("<vcpupin") {
                    let pin_abs_start = search_start + pin_start;
                    if let Some(pin_end) = cputune_section[pin_abs_start..].find("/>") {
                        let pin_element = &cputune_section[pin_abs_start..pin_abs_start + pin_end];

                        // Extract vcpu number
                        if let Some(vcpu_start) = pin_element.find("vcpu='") {
                            let vcpu_val_start = vcpu_start + 6;
                            if let Some(vcpu_end) = pin_element[vcpu_val_start..].find("'") {
                                if let Ok(vcpu) = pin_element
                                    [vcpu_val_start..vcpu_val_start + vcpu_end]
                                    .parse::<u32>()
                                {
                                    // Extract cpuset
                                    if let Some(cpuset_start) = pin_element.find("cpuset='") {
                                        let cpuset_val_start = cpuset_start + 8;
                                        if let Some(cpuset_end) =
                                            pin_element[cpuset_val_start..].find("'")
                                        {
                                            let cpuset_str = &pin_element
                                                [cpuset_val_start..cpuset_val_start + cpuset_end];
                                            let cpus = Self::parse_cpuset(cpuset_str);
                                            pinnings.push((vcpu, cpus));
                                        }
                                    }
                                }
                            }
                        }
                        search_start = pin_abs_start + pin_end + 2;
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(pinnings)
    }

    /// Parse a cpuset string like "0-3,5,7-9" into a vector of CPU numbers
    fn parse_cpuset(cpuset: &str) -> Vec<u32> {
        let mut cpus = Vec::new();
        for part in cpuset.split(',') {
            let part = part.trim();
            if part.contains('-') {
                // Range like "0-3"
                let bounds: Vec<&str> = part.split('-').collect();
                if bounds.len() == 2 {
                    if let (Ok(start), Ok(end)) =
                        (bounds[0].parse::<u32>(), bounds[1].parse::<u32>())
                    {
                        for i in start..=end {
                            cpus.push(i);
                        }
                    }
                }
            } else if let Ok(cpu) = part.parse::<u32>() {
                cpus.push(cpu);
            }
        }
        cpus.sort();
        cpus.dedup();
        cpus
    }

    /// Format a vector of CPU numbers into a cpuset string like "0-3,5,7-9"
    fn format_cpuset(cpus: &[u32]) -> String {
        if cpus.is_empty() {
            return String::new();
        }

        let mut sorted_cpus = cpus.to_vec();
        sorted_cpus.sort();
        sorted_cpus.dedup();

        let mut result = Vec::new();
        let mut range_start = sorted_cpus[0];
        let mut range_end = sorted_cpus[0];

        for &cpu in &sorted_cpus[1..] {
            if cpu == range_end + 1 {
                range_end = cpu;
            } else {
                if range_start == range_end {
                    result.push(format!("{}", range_start));
                } else {
                    result.push(format!("{}-{}", range_start, range_end));
                }
                range_start = cpu;
                range_end = cpu;
            }
        }

        // Push the last range
        if range_start == range_end {
            result.push(format!("{}", range_start));
        } else {
            result.push(format!("{}-{}", range_start, range_end));
        }

        result.join(",")
    }

    /// Set CPU pinning for a specific vCPU
    pub fn set_cpu_pin(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        vcpu: u32,
        host_cpus: Vec<u32>,
    ) -> Result<(), AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Note: is_running check kept for future live pinning support
        let _is_running = domain.is_active().map_err(map_libvirt_error)?;

        let cpuset = Self::format_cpuset(&host_cpus);
        if cpuset.is_empty() {
            return Err(AppError::InvalidConfig(
                "No host CPUs specified for pinning".to_string(),
            ));
        }

        tracing::info!(
            "Setting CPU pin: vCPU {} -> host CPUs {} for VM {}",
            vcpu,
            cpuset,
            vm_id
        );

        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;
        let new_xml = crate::utils::xml::rewrite_vcpu_pin(&xml, vcpu, Some(&cpuset))?;

        // Apply changes
        // Note: For running VMs, ideally we'd use virDomainPinVcpu for live pinning
        // but for now we update the persistent config which takes effect on restart
        Domain::define_xml(conn, &new_xml)
            .map_err(|e| AppError::Other(format!("Failed to update CPU pinning: {}", e)))?;

        tracing::info!(
            "Successfully set CPU pin for vCPU {} to {} for VM {}",
            vcpu,
            cpuset,
            vm_id
        );
        Ok(())
    }

    /// Clear CPU pinning for a specific vCPU
    pub fn clear_cpu_pin(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        vcpu: u32,
    ) -> Result<(), AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;
        if is_running {
            return Err(AppError::InvalidConfig(
                "Cannot modify CPU pinning while VM is running. Stop the VM first.".to_string(),
            ));
        }

        tracing::info!("Clearing CPU pin for vCPU {} on VM {}", vcpu, vm_id);

        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;
        let new_xml = crate::utils::xml::rewrite_vcpu_pin(&xml, vcpu, None)?;

        Domain::define_xml(conn, &new_xml)
            .map_err(|e| AppError::Other(format!("Failed to clear CPU pinning: {}", e)))?;

        tracing::info!(
            "Successfully cleared CPU pin for vCPU {} on VM {}",
            vcpu,
            vm_id
        );
        Ok(())
    }

    /// Attach a panic notifier device to a VM
    /// This allows the host to be notified when the guest kernel panics
    pub fn attach_panic_notifier(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        model: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Attaching panic notifier ({}) to VM {}", model, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        // Panic notifier can only be attached to stopped VMs
        if is_running {
            return Err(AppError::InvalidConfig(
                "Cannot add panic notifier to running VM. Please stop the VM first.".to_string(),
            ));
        }

        // Validate model
        let valid_models = ["isa", "hyperv", "pseries"];
        if !valid_models.contains(&model) {
            return Err(AppError::InvalidConfig(format!(
                "Invalid panic notifier model '{}'. Valid options: isa, hyperv, pseries",
                model
            )));
        }

        // Generate panic XML
        let panic_xml = format!(
            r#"<panic model='{}'/>
"#,
            model
        );

        domain
            .attach_device_flags(&panic_xml, sys::VIR_DOMAIN_AFFECT_CONFIG)
            .map_err(map_libvirt_error)?;

        tracing::info!("Successfully attached panic notifier to VM {}", vm_id);
        Ok(())
    }

    /// Attach a VirtIO VSOCK device to a VM
    /// VSOCK allows fast guest-host communication without network configuration
    pub fn attach_vsock(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        cid: u32,
    ) -> Result<(), AppError> {
        tracing::info!("Attaching VSOCK (CID: {}) to VM {}", cid, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        // Validate CID (Context ID)
        // CID 0, 1, and 2 are reserved
        // 0 = hypervisor, 1 = local loopback, 2 = host
        if cid < 3 {
            return Err(AppError::InvalidConfig(format!(
                "Invalid VSOCK CID {}. CID must be >= 3 (0, 1, 2 are reserved)",
                cid
            )));
        }

        // Generate VSOCK XML
        let vsock_xml = format!(
            r#"<vsock model='virtio'>
  <cid auto='no' value='{}'/>
</vsock>"#,
            cid
        );

        let flags = if is_running {
            sys::VIR_DOMAIN_AFFECT_LIVE | sys::VIR_DOMAIN_AFFECT_CONFIG
        } else {
            sys::VIR_DOMAIN_AFFECT_CONFIG
        };

        domain.attach_device_flags(&vsock_xml, flags).map_err(|e| {
            if is_running {
                AppError::InvalidConfig(
                    "Cannot add VSOCK to running VM. Please stop the VM first.".to_string(),
                )
            } else {
                map_libvirt_error(e)
            }
        })?;

        tracing::info!("Successfully attached VSOCK (CID: {}) to VM {}", cid, vm_id);
        Ok(())
    }

    /// Attach a parallel port device to a VM
    pub fn attach_parallel(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        target_port: u32,
    ) -> Result<(), AppError> {
        tracing::info!("Attaching parallel port {} to VM {}", target_port, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        // Parallel ports can only be attached to stopped VMs
        if is_running {
            return Err(AppError::InvalidConfig(
                "Cannot add parallel port to running VM. Please stop the VM first.".to_string(),
            ));
        }

        // Generate parallel port XML (pty backend is most common)
        let parallel_xml = format!(
            r#"<parallel type='pty'>
  <target port='{}'/>
</parallel>"#,
            target_port
        );

        domain
            .attach_device_flags(&parallel_xml, sys::VIR_DOMAIN_AFFECT_CONFIG)
            .map_err(map_libvirt_error)?;

        tracing::info!(
            "Successfully attached parallel port {} to VM {}",
            target_port,
            vm_id
        );
        Ok(())
    }

    /// Attach a smartcard reader device to a VM
    /// mode: passthrough (host reader) or emulated (software emulation)
    pub fn attach_smartcard(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        mode: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Attaching smartcard ({}) to VM {}", mode, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        // Smartcard can only be attached to stopped VMs
        if is_running {
            return Err(AppError::InvalidConfig(
                "Cannot add smartcard to running VM. Please stop the VM first.".to_string(),
            ));
        }

        // Validate mode
        let valid_modes = ["passthrough", "emulated"];
        if !valid_modes.contains(&mode) {
            return Err(AppError::InvalidConfig(format!(
                "Invalid smartcard mode '{}'. Valid options: passthrough, emulated",
                mode
            )));
        }

        // Generate smartcard XML
        // Passthrough mode forwards the host's smartcard reader
        // Emulated mode uses certificate database for software emulation
        let smartcard_xml = if mode == "passthrough" {
            r#"<smartcard mode='passthrough' type='spicevmc'>
  <address type='ccid' controller='0' slot='0'/>
</smartcard>"#
                .to_string()
        } else {
            // Emulated mode with NSS database
            r#"<smartcard mode='host-certificates'>
  <certificate>cert1</certificate>
  <certificate>cert2</certificate>
  <certificate>cert3</certificate>
  <database>/etc/pki/nssdb</database>
  <address type='ccid' controller='0' slot='0'/>
</smartcard>"#
                .to_string()
        };

        domain
            .attach_device_flags(&smartcard_xml, sys::VIR_DOMAIN_AFFECT_CONFIG)
            .map_err(map_libvirt_error)?;

        tracing::info!("Successfully attached smartcard ({}) to VM {}", mode, vm_id);
        Ok(())
    }

    /// Get host NUMA topology information
    pub fn get_host_numa_topology(
        libvirt: &impl ConnectionProvider,
    ) -> Result<Vec<crate::models::vm::HostNumaNode>, AppError> {
        let conn = libvirt.get_connection();

        // Use virsh capabilities to get NUMA topology
        let caps_xml = conn
            .get_capabilities()
            .map_err(|e| AppError::LibvirtError(format!("Failed to get capabilities: {}", e)))?;

        let mut nodes = Vec::new();

        // Parse NUMA topology from capabilities XML
        // Looking for <topology><cells><cell ...>
        for line in caps_xml.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("<cell id=") {
                // Parse cell id
                if let Some(id_start) = trimmed.find("id='") {
                    let id_str_start = id_start + 4;
                    if let Some(id_end) = trimmed[id_str_start..].find('\'') {
                        if let Ok(id) = trimmed[id_str_start..id_str_start + id_end].parse::<u32>()
                        {
                            // Get CPUs and memory from nested elements
                            // For now, create a basic structure
                            nodes.push(crate::models::vm::HostNumaNode {
                                id,
                                cpus: Vec::new(), // Would need more parsing
                                memory_mb: 0,     // Would need more parsing
                            });
                        }
                    }
                }
            }
        }

        // If we couldn't parse from capabilities, fall back to reading /sys/devices/system/node/
        if nodes.is_empty() {
            // Get NUMA info from sysfs
            if let Ok(entries) = std::fs::read_dir("/sys/devices/system/node") {
                for entry in entries.filter_map(|e| e.ok()) {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if let Some(node_id) = name_str.strip_prefix("node") {
                        if let Ok(id) = node_id.parse::<u32>() {
                            // Read CPUs
                            let cpulist_path = entry.path().join("cpulist");
                            let cpus = if let Ok(cpulist) = std::fs::read_to_string(&cpulist_path) {
                                Self::parse_cpu_list(cpulist.trim())
                            } else {
                                Vec::new()
                            };

                            // Read memory
                            let meminfo_path = entry.path().join("meminfo");
                            let memory_mb =
                                if let Ok(meminfo) = std::fs::read_to_string(&meminfo_path) {
                                    // Parse MemTotal line
                                    meminfo
                                        .lines()
                                        .find(|l| l.contains("MemTotal:"))
                                        .and_then(|l| {
                                            l.split_whitespace()
                                                .nth(3)
                                                .and_then(|kb| kb.parse::<u64>().ok())
                                        })
                                        .map(|kb| kb / 1024)
                                        .unwrap_or(0)
                                } else {
                                    0
                                };

                            nodes.push(crate::models::vm::HostNumaNode {
                                id,
                                cpus,
                                memory_mb,
                            });
                        }
                    }
                }
            }
        }

        // Sort by node ID
        nodes.sort_by_key(|n| n.id);

        tracing::info!("Found {} NUMA nodes", nodes.len());
        Ok(nodes)
    }

    /// Parse a CPU list string like "0-3,8-11" into a Vec of CPU IDs
    fn parse_cpu_list(cpulist: &str) -> Vec<u32> {
        let mut cpus = Vec::new();
        for part in cpulist.split(',') {
            let part = part.trim();
            if part.contains('-') {
                let range: Vec<&str> = part.split('-').collect();
                if range.len() == 2 {
                    if let (Ok(start), Ok(end)) = (range[0].parse::<u32>(), range[1].parse::<u32>())
                    {
                        for cpu in start..=end {
                            cpus.push(cpu);
                        }
                    }
                }
            } else if let Ok(cpu) = part.parse::<u32>() {
                cpus.push(cpu);
            }
        }
        cpus
    }

    /// Get VM NUMA configuration
    pub fn get_vm_numa_config(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
    ) -> Result<Option<crate::models::vm::VmNumaConfig>, AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        let Some(numatune_xml) = first_element_fragment(&xml, "numatune")? else {
            return Ok(None);
        };
        let mode = first_element_attribute(&numatune_xml, "numatune", "memory", "mode")?
            .unwrap_or_else(|| "strict".to_string());
        let nodeset = first_element_attribute(&numatune_xml, "numatune", "memory", "nodeset")?;

        // Check if any NUMA config exists
        if nodeset.is_none() {
            return Ok(None);
        }

        Ok(Some(crate::models::vm::VmNumaConfig {
            mode,
            nodeset,
            cells: Vec::new(), // Would need more parsing for guest NUMA cells
        }))
    }

    /// Set VM NUMA configuration
    pub fn set_vm_numa_config(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        config: crate::models::vm::VmNumaConfig,
    ) -> Result<(), AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;
        if is_running {
            return Err(AppError::InvalidConfig(
                "Cannot modify NUMA configuration while VM is running. Please stop the VM first."
                    .to_string(),
            ));
        }

        // Validate mode
        let valid_modes = ["strict", "preferred", "interleave"];
        if !valid_modes.contains(&config.mode.as_str()) {
            return Err(AppError::InvalidConfig(format!(
                "Invalid NUMA mode '{}'. Valid options: strict, preferred, interleave",
                config.mode
            )));
        }

        // Build the owned NUMA subtree. Escaping keeps user-provided topology values from
        // becoming XML markup when the definition is redefined.
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;
        let nodeset_str = config.nodeset.as_deref().unwrap_or("0");
        let numatune_xml = format!(
            "<numatune><memory mode='{}' nodeset='{}'/></numatune>",
            escaped_attribute(&config.mode, "NUMA mode")?,
            escaped_attribute(nodeset_str, "NUMA nodeset")?,
        );
        let new_xml = replace_direct_child(&xml, "domain", "numatune", Some(&numatune_xml))?;

        // Undefine and redefine the domain with new XML
        domain.undefine().map_err(map_libvirt_error)?;
        Domain::define_xml(conn, &new_xml).map_err(|e| {
            AppError::LibvirtError(format!("Failed to redefine VM with NUMA config: {}", e))
        })?;

        tracing::info!(
            "Successfully set NUMA config for VM {}: mode={}, nodeset={:?}",
            vm_id,
            config.mode,
            config.nodeset
        );
        Ok(())
    }

    /// Clear VM NUMA configuration
    pub fn clear_vm_numa_config(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
    ) -> Result<(), AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;
        if is_running {
            return Err(AppError::InvalidConfig(
                "Cannot modify NUMA configuration while VM is running. Please stop the VM first."
                    .to_string(),
            ));
        }

        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;
        let new_xml = replace_direct_child(&xml, "domain", "numatune", None)?;

        // Undefine and redefine the domain with new XML
        domain.undefine().map_err(map_libvirt_error)?;
        Domain::define_xml(conn, &new_xml)
            .map_err(|e| AppError::LibvirtError(format!("Failed to redefine VM: {}", e)))?;

        tracing::info!("Successfully cleared NUMA config for VM {}", vm_id);
        Ok(())
    }

    /// Migrate a VM to another host
    /// Supports both live (running) and offline (stopped) migration
    pub fn migrate_vm(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        dest_uri: &str,
        live: bool,
        unsafe_migration: bool,
    ) -> Result<(), AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let vm_name = domain.get_name().map_err(map_libvirt_error)?;
        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        tracing::info!(
            "Starting {} migration of VM {} to {}",
            if live { "live" } else { "offline" },
            vm_name,
            dest_uri
        );

        // Connect to destination
        let dest_conn = virt::connect::Connect::open(Some(dest_uri)).map_err(|e| {
            AppError::LibvirtError(format!(
                "Failed to connect to destination {}: {}",
                dest_uri, e
            ))
        })?;

        if live && is_running {
            // Live migration for running VMs
            let mut flags = sys::VIR_MIGRATE_LIVE
                | sys::VIR_MIGRATE_PERSIST_DEST
                | sys::VIR_MIGRATE_UNDEFINE_SOURCE;

            if unsafe_migration {
                // Allow migration even with CPU model differences
                flags |= sys::VIR_MIGRATE_UNSAFE;
            }

            // Perform live migration
            domain
                .migrate(&dest_conn, flags, None, None, 0)
                .map_err(|e| AppError::LibvirtError(format!("Live migration failed: {}", e)))?;

            tracing::info!("Live migration of VM {} completed successfully", vm_name);
        } else {
            // Offline migration for stopped VMs
            // Get the XML definition
            let xml = domain
                .get_xml_desc(sys::VIR_DOMAIN_XML_INACTIVE)
                .map_err(map_libvirt_error)?;

            // Define on destination
            Domain::define_xml(&dest_conn, &xml).map_err(|e| {
                AppError::LibvirtError(format!("Failed to define VM on destination: {}", e))
            })?;

            // Undefine on source (after successful define on destination)
            domain.undefine().map_err(|e| {
                AppError::LibvirtError(format!("Failed to undefine VM on source: {}", e))
            })?;

            tracing::info!("Offline migration of VM {} completed successfully", vm_name);
        }

        Ok(())
    }

    /// Get migration capabilities and estimated time
    pub fn get_migration_info(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
    ) -> Result<MigrationInfo, AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let info = domain.get_info().map_err(map_libvirt_error)?;
        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        // Get disk paths to estimate data transfer size
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;
        let mut disk_size_bytes: u64 = 0;

        for line in xml.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("<source file='") {
                if let Some(start) = trimmed.find("file='") {
                    let path_start = start + 6;
                    if let Some(end) = trimmed[path_start..].find("'") {
                        let path = &trimmed[path_start..path_start + end];
                        if let Ok(metadata) = std::fs::metadata(path) {
                            disk_size_bytes += metadata.len();
                        }
                    }
                }
            }
        }

        Ok(MigrationInfo {
            can_live_migrate: is_running,
            memory_mb: (info.memory / 1024) as u64,
            disk_size_bytes,
            estimated_downtime_ms: if is_running { 100 } else { 0 }, // Rough estimate
        })
    }

    /// Get direct kernel boot settings for a VM
    pub fn get_kernel_boot_settings(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
    ) -> Result<KernelBootSettings, AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        // Event-based readers decode entities and remain correct when libvirt changes
        // indentation or attribute quote style.
        let kernel_path = first_element_text(&xml, "domain", "kernel")?;
        let initrd_path = first_element_text(&xml, "domain", "initrd")?;
        let kernel_args = first_element_text(&xml, "domain", "cmdline")?;
        let dtb_path = first_element_text(&xml, "domain", "dtb")?;

        let enabled = kernel_path.is_some() || initrd_path.is_some() || kernel_args.is_some();

        Ok(KernelBootSettings {
            enabled,
            kernel_path,
            initrd_path,
            kernel_args,
            dtb_path,
        })
    }

    /// Update direct kernel boot settings for a VM (VM must be shut off)
    pub fn set_kernel_boot_settings(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        settings: KernelBootSettings,
    ) -> Result<(), AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Verify VM is not running
        if domain.is_active().map_err(map_libvirt_error)? {
            return Err(AppError::InvalidConfig(
                "VM must be shut down to modify kernel boot settings".to_string(),
            ));
        }

        // Validate paths if provided
        if settings.enabled {
            if let Some(ref path) = settings.kernel_path {
                if !path.is_empty() && !std::path::Path::new(path).exists() {
                    return Err(AppError::InvalidConfig(format!(
                        "Kernel file not found: {}",
                        path
                    )));
                }
            }
            if let Some(ref path) = settings.initrd_path {
                if !path.is_empty() && !std::path::Path::new(path).exists() {
                    return Err(AppError::InvalidConfig(format!(
                        "Initrd file not found: {}",
                        path
                    )));
                }
            }
            if let Some(ref path) = settings.dtb_path {
                if !path.is_empty() && !std::path::Path::new(path).exists() {
                    return Err(AppError::InvalidConfig(format!(
                        "Device tree blob file not found: {}",
                        path
                    )));
                }
            }
        }

        let mut xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;
        let kernel_element = if settings.enabled {
            settings
                .kernel_path
                .as_deref()
                .filter(|path| !path.is_empty())
                .map(|path| {
                    escaped_text(path, "kernel path")
                        .map(|value| format!("<kernel>{value}</kernel>"))
                })
                .transpose()?
        } else {
            None
        };
        let initrd_element = if settings.enabled {
            settings
                .initrd_path
                .as_deref()
                .filter(|path| !path.is_empty())
                .map(|path| {
                    escaped_text(path, "initrd path")
                        .map(|value| format!("<initrd>{value}</initrd>"))
                })
                .transpose()?
        } else {
            None
        };
        let cmdline_element = if settings.enabled {
            settings
                .kernel_args
                .as_deref()
                .filter(|args| !args.is_empty())
                .map(|args| {
                    escaped_text(args, "kernel command line")
                        .map(|value| format!("<cmdline>{value}</cmdline>"))
                })
                .transpose()?
        } else {
            None
        };
        let dtb_element = if settings.enabled {
            settings
                .dtb_path
                .as_deref()
                .filter(|path| !path.is_empty())
                .map(|path| {
                    escaped_text(path, "device tree path")
                        .map(|value| format!("<dtb>{value}</dtb>"))
                })
                .transpose()?
        } else {
            None
        };

        for (tag, replacement) in [
            ("kernel", kernel_element.as_deref()),
            ("initrd", initrd_element.as_deref()),
            ("cmdline", cmdline_element.as_deref()),
            ("dtb", dtb_element.as_deref()),
        ] {
            xml = replace_direct_child(&xml, "os", tag, replacement)?;
        }

        // Undefine and redefine the domain with updated XML
        domain.undefine().map_err(map_libvirt_error)?;
        Domain::define_xml(conn, &xml).map_err(map_libvirt_error)?;

        tracing::info!("Updated kernel boot settings for VM {}", vm_id);
        Ok(())
    }

    /// Update network interface bandwidth/QoS settings
    /// Units: average/peak in KB/s, burst in KB
    #[allow(clippy::too_many_arguments)] // Mirrors the existing Tauri command contract.
    pub fn update_interface_bandwidth(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        mac_address: &str,
        inbound_average: Option<u64>,
        inbound_peak: Option<u64>,
        inbound_burst: Option<u64>,
        outbound_average: Option<u64>,
        outbound_peak: Option<u64>,
        outbound_burst: Option<u64>,
    ) -> Result<(), AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;
        let original_xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;
        // Resolve the target and its owned bandwidth subtree by XML events before any live
        // operation. This handles namespace prefixes and either XML quote style safely.
        let xml = rewrite_interface_bandwidth(
            &original_xml,
            mac_address,
            inbound_average,
            inbound_peak,
            inbound_burst,
            outbound_average,
            outbound_peak,
            outbound_burst,
        )?;
        let interface_xml = first_element_with_descendant_attribute(
            &xml,
            "interface",
            "mac",
            "address",
            mac_address,
        )?
        .ok_or_else(|| {
            AppError::InvalidConfig(format!(
                "Network interface with MAC {mac_address} not found"
            ))
        })?;
        let target_dev = first_element_attribute(&interface_xml, "interface", "target", "dev")?
            .unwrap_or_else(|| mac_address.replace(':', ""));

        if is_running {
            // For running VMs, we need to use domiftune to apply bandwidth limits live
            // and also update the persistent config
            let vm_name = domain.get_name().map_err(map_libvirt_error)?;

            // Find target device for this interface
            // Apply using virsh domiftune for live changes
            let mut args = vec!["domiftune".to_string(), vm_name.clone(), target_dev.clone()];

            if let Some(avg) = inbound_average {
                args.push("--inbound".to_string());
                args.push(format!(
                    "{},{},{}",
                    avg,
                    inbound_peak.unwrap_or(avg * 2),
                    inbound_burst.unwrap_or(avg)
                ));
            }

            if let Some(avg) = outbound_average {
                args.push("--outbound".to_string());
                args.push(format!(
                    "{},{},{}",
                    avg,
                    outbound_peak.unwrap_or(avg * 2),
                    outbound_burst.unwrap_or(avg)
                ));
            }

            if args.len() > 3 {
                let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                let output = Self::virsh_command(conn)?
                    .args(&args_ref)
                    .output()
                    .map_err(|e| AppError::Other(format!("Failed to run virsh: {}", e)))?;

                if !output.status.success() {
                    tracing::warn!("Failed to apply live bandwidth limits");
                } else {
                    tracing::info!("Applied live bandwidth limits to interface {}", target_dev);
                }
            }

            // Also update persistent config
            domain.undefine().map_err(map_libvirt_error)?;
            Domain::define_xml(conn, &xml).map_err(map_libvirt_error)?;
        } else {
            // For stopped VMs, just update the definition
            domain.undefine().map_err(map_libvirt_error)?;
            Domain::define_xml(conn, &xml).map_err(map_libvirt_error)?;
        }

        tracing::info!(
            "Updated bandwidth settings for interface {} on VM {}",
            mac_address,
            vm_id
        );
        Ok(())
    }

    /// Set the link state (up/down) for a network interface
    /// This simulates connecting/disconnecting the network cable
    pub fn set_interface_link_state(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        mac_address: &str,
        link_up: bool,
    ) -> Result<(), AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;
        let original_xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;
        let link_state = if link_up { "up" } else { "down" };
        let xml = rewrite_interface_link_state(&original_xml, mac_address, link_state)?;
        let new_interface_xml = first_element_with_descendant_attribute(
            &xml,
            "interface",
            "mac",
            "address",
            mac_address,
        )?
        .ok_or_else(|| AppError::InvalidConfig("Network interface was not found".to_string()))?;

        if is_running {
            // For running VMs, use virsh domif-setlink for live change
            let vm_name = domain.get_name().map_err(map_libvirt_error)?;

            let target_dev =
                first_element_attribute(&new_interface_xml, "interface", "target", "dev")?
                    .unwrap_or_else(|| mac_address.replace(':', ""));

            // Apply using virsh domif-setlink for live changes
            let output = Self::virsh_command(conn)?
                .args(["domif-setlink", &vm_name, &target_dev, link_state, "--live"])
                .output()
                .map_err(|e| AppError::Other(format!("Failed to run virsh: {}", e)))?;

            if !output.status.success() {
                return Err(AppError::Other("Failed to set link state".to_string()));
            }

            tracing::info!(
                "Set live link state {} for interface {} on VM {}",
                link_state,
                target_dev,
                vm_id
            );

            // Also update persistent config
            domain.undefine().map_err(map_libvirt_error)?;
            Domain::define_xml(conn, &xml).map_err(map_libvirt_error)?;
        } else {
            // For stopped VMs, just update the definition
            domain.undefine().map_err(map_libvirt_error)?;
            Domain::define_xml(conn, &xml).map_err(map_libvirt_error)?;
        }

        tracing::info!(
            "Set link state to {} for interface {} on VM {}",
            link_state,
            mac_address,
            vm_id
        );
        Ok(())
    }

    /// Get the current link state of a network interface
    pub fn get_interface_link_state(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        mac_address: &str,
    ) -> Result<bool, AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;
        let interface_xml = first_element_with_descendant_attribute(
            &xml,
            "interface",
            "mac",
            "address",
            mac_address,
        )?;

        if is_running {
            if let (Some(interface_xml), vm_name) = (
                interface_xml.as_deref(),
                domain.get_name().map_err(map_libvirt_error)?,
            ) {
                if let Some(target_dev) =
                    first_element_attribute(interface_xml, "interface", "target", "dev")?
                {
                    let output = Self::virsh_command(conn)?
                        .args(["domif-getlink", &vm_name, &target_dev])
                        .output()
                        .map_err(|e| AppError::Other(format!("Failed to run virsh: {}", e)))?;
                    if output.status.success() {
                        return Ok(!String::from_utf8_lossy(&output.stdout).contains("down"));
                    }
                }
            }
        }

        if let Some(interface_xml) = interface_xml {
            if let Some(state) =
                first_element_attribute(&interface_xml, "interface", "link", "state")?
            {
                return Ok(state != "down");
            }
        }

        // Default to link up if not specified
        Ok(true)
    }

    /// Get hugepages memory backing configuration for a VM
    pub fn get_hugepages_settings(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
    ) -> Result<HugepagesSettings, AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;

        // Check for <memoryBacking><hugepages> element
        let enabled = xml.contains("<hugepages");
        let mut size: Option<u64> = None;

        if enabled {
            // Try to extract page size if specified
            // <page size='2048' unit='KiB'/>
            if let Some(page_start) = xml.find("<page size='") {
                let rest = &xml[page_start + 12..];
                if let Some(end) = rest.find("'") {
                    if let Ok(page_size) = rest[..end].parse::<u64>() {
                        // Check unit and convert to KiB if needed
                        if rest.contains("unit='KiB'") || rest.contains("unit='kib'") {
                            size = Some(page_size);
                        } else if rest.contains("unit='MiB'") || rest.contains("unit='mib'") {
                            size = Some(page_size * 1024);
                        } else if rest.contains("unit='GiB'") || rest.contains("unit='gib'") {
                            size = Some(page_size * 1024 * 1024);
                        }
                    }
                }
            }
        }

        Ok(HugepagesSettings { enabled, size })
    }

    /// Set hugepages memory backing for a VM (requires VM to be stopped)
    pub fn set_hugepages(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        enabled: bool,
        size: Option<u64>, // Size in KiB
    ) -> Result<(), AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let info = domain.get_info().map_err(map_libvirt_error)?;
        if info.state == sys::VIR_DOMAIN_RUNNING {
            return Err(AppError::InvalidConfig(
                "Cannot change hugepages while VM is running. Stop the VM first.".to_string(),
            ));
        }

        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;
        let memory_backing = enabled.then(|| match size {
            Some(page_size) => format!(
                "<memoryBacking><hugepages><page size='{page_size}' unit='KiB'/></hugepages></memoryBacking>"
            ),
            None => "<memoryBacking><hugepages/></memoryBacking>".to_string(),
        });
        let new_xml =
            replace_direct_child(&xml, "domain", "memoryBacking", memory_backing.as_deref())?;

        // Redefine the domain with updated XML
        Domain::define_xml(conn, &new_xml)
            .map_err(|e| AppError::Other(format!("Failed to update hugepages settings: {}", e)))?;

        tracing::info!(
            "Successfully {} hugepages{} for VM {}",
            if enabled { "enabled" } else { "disabled" },
            if let Some(s) = size {
                format!(" ({}KB)", s)
            } else {
                String::new()
            },
            vm_id
        );
        Ok(())
    }

    /// Get available hugepage sizes on the host system
    pub fn get_host_hugepage_info() -> Result<Vec<HugepageInfo>, AppError> {
        let mut info = Vec::new();

        // Check /sys/kernel/mm/hugepages for available hugepage sizes
        let hugepages_path = std::path::Path::new("/sys/kernel/mm/hugepages");
        if hugepages_path.exists() {
            if let Ok(entries) = std::fs::read_dir(hugepages_path) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    // Format: hugepages-2048kB, hugepages-1048576kB
                    if let Some(rest) = name.strip_prefix("hugepages-") {
                        if let Some(kb_str) = rest.strip_suffix("kB") {
                            if let Ok(size_kb) = kb_str.parse::<u64>() {
                                let entry_path = entry.path();

                                // Read nr_hugepages (total available)
                                let nr_path = entry_path.join("nr_hugepages");
                                let total: u64 = std::fs::read_to_string(&nr_path)
                                    .unwrap_or_default()
                                    .trim()
                                    .parse()
                                    .unwrap_or(0);

                                // Read free_hugepages
                                let free_path = entry_path.join("free_hugepages");
                                let free: u64 = std::fs::read_to_string(&free_path)
                                    .unwrap_or_default()
                                    .trim()
                                    .parse()
                                    .unwrap_or(0);

                                info.push(HugepageInfo {
                                    size_kb,
                                    total,
                                    free,
                                    size_human: Self::format_size_kb(size_kb),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Sort by size
        info.sort_by(|a, b| a.size_kb.cmp(&b.size_kb));

        Ok(info)
    }

    /// Helper to format size in human-readable form
    fn format_size_kb(size_kb: u64) -> String {
        if size_kb >= 1048576 {
            format!("{} GB", size_kb / 1048576)
        } else if size_kb >= 1024 {
            format!("{} MB", size_kb / 1024)
        } else {
            format!("{} KB", size_kb)
        }
    }

    /// List available evdev input devices on the host
    pub fn list_evdev_devices() -> Result<Vec<EvdevDevice>, AppError> {
        let mut devices = Vec::new();
        let by_id_path = std::path::Path::new("/dev/input/by-id");

        if !by_id_path.exists() {
            return Ok(devices);
        }

        if let Ok(entries) = std::fs::read_dir(by_id_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let path = entry.path();

                // Get the actual device path
                let device_path = if let Ok(target) = std::fs::read_link(&path) {
                    // Resolve relative path (e.g., ../event10)
                    let parent = path.parent().unwrap_or(std::path::Path::new("/dev/input"));
                    parent.join(target).canonicalize().unwrap_or(path.clone())
                } else {
                    path.clone()
                };

                // Determine device type from name
                let device_type = if name.contains("kbd") || name.contains("keyboard") {
                    "keyboard"
                } else if name.contains("mouse") {
                    "mouse"
                } else if name.contains("joystick") || name.contains("gamepad") {
                    "joystick"
                } else {
                    "other"
                };

                // Extract a friendly name
                let friendly_name = name
                    .replace("usb-", "")
                    .replace("-event-kbd", "")
                    .replace("-event-mouse", "")
                    .replace("-if01-event-mouse", "")
                    .replace("-if02-event-kbd", "")
                    .replace("-if02-event-mouse", "")
                    .replace("_", " ");

                devices.push(EvdevDevice {
                    id: name.clone(),
                    path: device_path.to_string_lossy().to_string(),
                    name: friendly_name,
                    device_type: device_type.to_string(),
                });
            }
        }

        // Sort by type then name
        devices.sort_by(|a, b| {
            a.device_type
                .cmp(&b.device_type)
                .then_with(|| a.name.cmp(&b.name))
        });

        Ok(devices)
    }

    /// Attach an evdev input device to a VM for low-latency passthrough
    pub fn attach_evdev(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        device_path: &str,
        grab_all: bool, // If true, grab all input from this device
    ) -> Result<(), AppError> {
        tracing::info!("Attaching an evdev device to a VM");

        // Validate device path exists
        if !std::path::Path::new(device_path).exists() {
            return Err(AppError::InvalidConfig(format!(
                "Evdev device not found: {}",
                device_path
            )));
        }

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        if is_running {
            return Err(AppError::InvalidConfig(
                "Cannot attach evdev device to running VM. Please stop the VM first.".to_string(),
            ));
        }

        // Build evdev input XML
        // Determine input type from path
        let input_type = if device_path.contains("kbd") || device_path.contains("keyboard") {
            "keyboard"
        } else if device_path.contains("mouse") {
            "mouse"
        } else {
            "passthrough"
        };

        let grab_attr = if grab_all { " grab='all'" } else { "" };

        let escaped_device_path = escaped_attribute(device_path, "evdev device path")?;
        let evdev_xml = format!(
            r#"<input type='{}' bus='virtio'>
  <source dev='{}'{}/>
</input>"#,
            input_type, escaped_device_path, grab_attr
        );

        domain
            .attach_device_flags(&evdev_xml, sys::VIR_DOMAIN_AFFECT_CONFIG)
            .map_err(|e| AppError::Other(format!("Failed to attach evdev device: {}", e)))?;

        tracing::info!("Successfully attached an evdev device");
        Ok(())
    }

    /// Get list of evdev devices attached to a VM
    pub fn get_vm_evdev_devices(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
    ) -> Result<Vec<String>, AppError> {
        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;
        descendant_attribute_values(&xml, "input", "source", "dev")
    }

    /// Remove an evdev device from a VM
    pub fn detach_evdev(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
        device_path: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Detaching an evdev device from a VM");

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let is_running = domain.is_active().map_err(map_libvirt_error)?;

        if is_running {
            return Err(AppError::InvalidConfig(
                "Cannot detach evdev device from running VM. Please stop the VM first.".to_string(),
            ));
        }

        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;
        let new_xml =
            remove_element_with_descendant_attribute(&xml, "input", "source", "dev", device_path)?;

        // Redefine the domain
        Domain::define_xml(conn, &new_xml)
            .map_err(|e| AppError::Other(format!("Failed to detach evdev device: {}", e)))?;

        tracing::info!("Successfully detached an evdev device");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{deletion_undefine_flags, LIBVIRT_TPM_UNDEFINE_MIN_VERSION};
    use virt::sys;

    #[test]
    fn deletion_flags_remove_uefi_and_managed_save_state_on_supported_libvirt_versions() {
        let flags = deletion_undefine_flags(LIBVIRT_TPM_UNDEFINE_MIN_VERSION - 1);

        assert_ne!(flags & sys::VIR_DOMAIN_UNDEFINE_NVRAM, 0);
        assert_ne!(flags & sys::VIR_DOMAIN_UNDEFINE_MANAGED_SAVE, 0);
        assert_eq!(flags & sys::VIR_DOMAIN_UNDEFINE_TPM, 0);
        assert_eq!(flags & sys::VIR_DOMAIN_UNDEFINE_SNAPSHOTS_METADATA, 0);
    }

    #[test]
    fn deletion_flags_remove_tpm_state_when_the_daemon_supports_it() {
        let flags = deletion_undefine_flags(LIBVIRT_TPM_UNDEFINE_MIN_VERSION);

        assert_ne!(flags & sys::VIR_DOMAIN_UNDEFINE_NVRAM, 0);
        assert_ne!(flags & sys::VIR_DOMAIN_UNDEFINE_TPM, 0);
        assert_eq!(flags & sys::VIR_DOMAIN_UNDEFINE_SNAPSHOTS_METADATA, 0);
    }

    #[test]
    fn deletion_disk_discovery_excludes_attached_iso_media() {
        let xml = r#"
            <domain type='kvm'>
              <name>windows-test</name>
              <devices>
                <disk type='file' device='disk'>
                  <source file='/var/lib/libvirt/images/windows-test.qcow2'/>
                  <target dev='vda' bus='virtio'/>
                </disk>
                <disk type='file' device='cdrom'>
                  <source file='/home/user/Downloads/windows.iso'/>
                  <target dev='sda' bus='sata'/>
                  <readonly/>
                </disk>
              </devices>
            </domain>
        "#;

        assert_eq!(
            super::VmService::extract_disk_paths(xml),
            vec!["/var/lib/libvirt/images/windows-test.qcow2"]
        );
    }
}

/// Migration information for a VM
#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MigrationInfo {
    pub can_live_migrate: bool,
    pub memory_mb: u64,
    pub disk_size_bytes: u64,
    pub estimated_downtime_ms: u64,
}

/// Evdev input device information
#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EvdevDevice {
    pub id: String,
    pub path: String,
    pub name: String,
    pub device_type: String, // keyboard, mouse, joystick, other
}

/// Direct kernel boot settings for a VM
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct KernelBootSettings {
    pub enabled: bool,
    pub kernel_path: Option<String>,
    pub initrd_path: Option<String>,
    pub kernel_args: Option<String>,
    pub dtb_path: Option<String>,
}

/// CPU model configuration for a VM
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CpuModelConfig {
    /// CPU mode: host-passthrough, host-model, custom
    pub mode: String,
    /// CPU model name (only used when mode is "custom")
    pub model: Option<String>,
}

/// Hugepages memory backing settings
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct HugepagesSettings {
    pub enabled: bool,
    /// Page size in KiB (2048 = 2MB, 1048576 = 1GB)
    pub size: Option<u64>,
}

/// Information about available hugepages on host
#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HugepageInfo {
    /// Size in KiB
    pub size_kb: u64,
    /// Total number of hugepages allocated
    pub total: u64,
    /// Number of free hugepages
    pub free: u64,
    /// Human-readable size (e.g., "2 MB", "1 GB")
    pub size_human: String,
}

/// USB redirection configuration info
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct UsbRedirectionInfo {
    pub enabled: bool,
    pub channel_count: u32,
    pub has_spice_graphics: bool,
}

impl VmService {
    /// Check if a VM can be migrated (has compatible storage, etc.)
    pub fn check_migration_compatibility(
        libvirt: &impl ConnectionProvider,
        vm_id: &str,
    ) -> Result<(bool, Vec<String>), AppError> {
        let conn = libvirt.get_connection();
        let mut warnings: Vec<String> = Vec::new();
        let mut can_migrate = true;

        // Look up the domain by UUID
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        // Get domain XML to check for migration blockers
        let xml = domain
            .get_xml_desc(0)
            .map_err(|e| AppError::LibvirtError(format!("Failed to get VM XML: {}", e)))?;

        // Simple XML parsing to check for migration blockers
        // Check for local file-based disks
        for line in xml.lines() {
            let trimmed = line.trim();

            // Check for local disk files
            if trimmed.starts_with("<source file='") {
                if let Some(start) = trimmed.find("file='") {
                    let path_start = start + 6;
                    if let Some(end) = trimmed[path_start..].find("'") {
                        let path = &trimmed[path_start..path_start + end];
                        // Local files may need shared storage for migration
                        if path.starts_with("/var/lib/libvirt/images/")
                            || path.starts_with("/home/")
                            || (!path.contains("://") && !path.starts_with("/dev/"))
                        {
                            warnings.push(
                                "A local disk may require shared storage before migration"
                                    .to_string(),
                            );
                        }
                    }
                }
            }

            // Check for PCI host device passthrough
            if trimmed.contains("<hostdev mode='subsystem' type='pci'") {
                can_migrate = false;
                warnings
                    .push("PCI passthrough device detected - prevents live migration".to_string());
            }

            // Check for USB host device passthrough
            if trimmed.contains("<hostdev mode='subsystem' type='usb'") {
                warnings.push("USB passthrough device detected - may affect migration".to_string());
            }

            // Check for TPM
            if trimmed.starts_with("<tpm ") {
                warnings.push("TPM device detected - state may not transfer correctly".to_string());
            }

            // Check for MDEV passthrough
            if trimmed.contains("<hostdev mode='subsystem' type='mdev'") {
                can_migrate = false;
                warnings
                    .push("MDEV passthrough device detected - prevents live migration".to_string());
            }
        }

        Ok((can_migrate, warnings))
    }

    /// Get list of available migration destinations from saved connections
    pub fn get_migration_targets(
        connections: &crate::services::connection_service::ConnectionService,
    ) -> Result<Vec<(String, String)>, AppError> {
        let saved = connections.get_saved_connections()?;

        // Filter to remote connections only (not local)
        let targets: Vec<(String, String)> = saved
            .into_iter()
            .filter(|c| {
                c.connection_type != crate::services::connection_service::ConnectionType::Local
            })
            .map(|c| (c.id.clone(), c.build_uri()))
            .collect();

        Ok(targets)
    }
}
