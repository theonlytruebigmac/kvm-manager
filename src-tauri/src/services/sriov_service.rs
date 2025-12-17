use std::fs;
use std::path::Path;
use std::process::Command;

use virt::domain::Domain;
use crate::models::pci::{SriovPf, SriovVf, SriovVfConfig};
use crate::services::libvirt::LibvirtService;
use crate::utils::error::AppError;

/// SR-IOV service for managing Virtual Functions
pub struct SriovService;

impl SriovService {
    /// List all SR-IOV capable Physical Functions
    pub fn list_sriov_pfs() -> Result<Vec<SriovPf>, AppError> {
        tracing::debug!("Listing SR-IOV capable PFs");

        let mut pfs = Vec::new();
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
            let device_path = format!("/sys/bus/pci/devices/{}", address);

            // Check if this device supports SR-IOV (has sriov_totalvfs)
            let max_vfs_path = format!("{}/sriov_totalvfs", device_path);
            if !Path::new(&max_vfs_path).exists() {
                continue;
            }

            // Read max VFs
            let max_vfs = fs::read_to_string(&max_vfs_path)
                .ok()
                .and_then(|s| s.trim().parse::<u32>().ok())
                .unwrap_or(0);

            if max_vfs == 0 {
                continue;
            }

            // Read current number of VFs
            let num_vfs = fs::read_to_string(format!("{}/sriov_numvfs", device_path))
                .ok()
                .and_then(|s| s.trim().parse::<u32>().ok())
                .unwrap_or(0);

            // Get device/vendor names
            let vendor = fs::read_to_string(format!("{}/vendor", device_path))
                .unwrap_or_else(|_| "0x0000".to_string())
                .trim()
                .to_string();
            let device_id = fs::read_to_string(format!("{}/device", device_path))
                .unwrap_or_else(|_| "0x0000".to_string())
                .trim()
                .to_string();

            // Get vendor/device names from lspci
            let (vendor_name, device_name) = Self::get_pci_names(&address);

            // Get network interface name
            let interface_name = Self::get_pf_interface(&device_path);

            // Get driver
            let driver = fs::read_link(format!("{}/driver", device_path))
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()));

            // Get link speed if this is a network device
            let link_speed = interface_name.as_ref().and_then(|iface| {
                fs::read_to_string(format!("/sys/class/net/{}/speed", iface))
                    .ok()
                    .and_then(|s| s.trim().parse::<i32>().ok())
                    .filter(|&s| s > 0)
                    .map(|s| {
                        if s >= 1000 {
                            format!("{} Gb/s", s / 1000)
                        } else {
                            format!("{} Mb/s", s)
                        }
                    })
            });

            // Calculate available VFs
            let vfs_in_use = Self::count_vfs_in_use(&address);
            let available_vfs = num_vfs.saturating_sub(vfs_in_use);

            pfs.push(SriovPf {
                address,
                device_name: device_name.unwrap_or_else(|| format!("Device {}", device_id)),
                vendor: vendor_name.unwrap_or_else(|| format!("Vendor {}", vendor)),
                interface_name,
                max_vfs,
                num_vfs,
                available_vfs,
                driver,
                link_speed,
            });
        }

        tracing::info!("Found {} SR-IOV capable PFs", pfs.len());
        Ok(pfs)
    }

    /// List Virtual Functions for a Physical Function
    pub fn list_vfs(pf_address: &str, libvirt: &LibvirtService) -> Result<Vec<SriovVf>, AppError> {
        tracing::debug!("Listing VFs for PF: {}", pf_address);

        let mut vfs = Vec::new();
        let device_path = format!("/sys/bus/pci/devices/{}", pf_address);

        // Get number of VFs
        let num_vfs = fs::read_to_string(format!("{}/sriov_numvfs", device_path))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0);

        // Get PF interface name for reading VF configurations
        let pf_interface = Self::get_pf_interface(&device_path);

        for vf_index in 0..num_vfs {
            // Read VF PCI address
            let vf_link = format!("{}/virtfn{}", device_path, vf_index);
            let vf_address = match fs::read_link(&vf_link) {
                Ok(p) => p.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                Err(_) => continue,
            };

            if vf_address.is_empty() {
                continue;
            }

            // Get VF details
            let vf_device_path = format!("/sys/bus/pci/devices/{}", vf_address);

            // Get IOMMU group
            let iommu_group = fs::read_link(format!("{}/iommu_group", vf_device_path))
                .ok()
                .and_then(|p| {
                    p.file_name()
                        .and_then(|n| n.to_string_lossy().parse::<u32>().ok())
                });

            // Get VF MAC and VLAN if interface exists
            let (mac_address, vlan_id) = pf_interface
                .as_ref()
                .map(|iface| Self::get_vf_config(iface, vf_index))
                .unwrap_or((None, None));

            // Check if VF is attached to a VM
            let (in_use, attached_to_vm) = Self::get_vf_attachment(libvirt, &vf_address)?;

            vfs.push(SriovVf {
                address: vf_address,
                vf_index,
                parent_pf: pf_address.to_string(),
                mac_address,
                vlan_id,
                in_use,
                attached_to_vm,
                iommu_group,
            });
        }

        Ok(vfs)
    }

    /// Enable VFs on a Physical Function
    pub fn enable_vfs(pf_address: &str, num_vfs: u32) -> Result<(), AppError> {
        tracing::info!("Enabling {} VFs on PF: {}", num_vfs, pf_address);

        let sriov_numvfs_path = format!("/sys/bus/pci/devices/{}/sriov_numvfs", pf_address);

        // First disable existing VFs
        fs::write(&sriov_numvfs_path, "0")
            .map_err(|e| AppError::Other(format!("Failed to disable VFs: {}. Try running with root permissions.", e)))?;

        // Enable requested number of VFs
        if num_vfs > 0 {
            fs::write(&sriov_numvfs_path, num_vfs.to_string())
                .map_err(|e| AppError::Other(format!("Failed to enable {} VFs: {}", num_vfs, e)))?;
        }

        tracing::info!("Enabled {} VFs on {}", num_vfs, pf_address);
        Ok(())
    }

    /// Configure a Virtual Function
    pub fn configure_vf(config: &SriovVfConfig) -> Result<(), AppError> {
        tracing::info!("Configuring VF {} on {}", config.vf_index, config.pf_interface);

        // Use ip link command to configure VF
        let mut args = vec![
            "link".to_string(),
            "set".to_string(),
            config.pf_interface.clone(),
            "vf".to_string(),
            config.vf_index.to_string(),
        ];

        // Add MAC address if specified
        if let Some(ref mac) = config.mac_address {
            args.push("mac".to_string());
            args.push(mac.clone());
        }

        // Add VLAN if specified
        if let Some(vlan) = config.vlan_id {
            args.push("vlan".to_string());
            args.push(vlan.to_string());
        }

        // Add spoof check
        args.push("spoofchk".to_string());
        args.push(if config.spoof_check { "on" } else { "off" }.to_string());

        // Add trust mode
        args.push("trust".to_string());
        args.push(if config.trust { "on" } else { "off" }.to_string());

        let output = Command::new("ip")
            .args(&args)
            .output()
            .map_err(|e| AppError::Other(format!("Failed to run ip command: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Other(format!("Failed to configure VF: {}", stderr)));
        }

        Ok(())
    }

    /// Attach a VF to a VM using libvirt
    pub fn attach_vf_to_vm(
        libvirt: &LibvirtService,
        vm_id: &str,
        vf_address: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Attaching VF {} to VM {}", vf_address, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|e| AppError::VmNotFound(format!("{}: {}", vm_id, e)))?;

        // Parse PCI address (format: 0000:01:00.0)
        let parts: Vec<&str> = vf_address.split(&[':', '.'][..]).collect();
        if parts.len() != 4 {
            return Err(AppError::InvalidConfig(format!("Invalid PCI address format: {}", vf_address)));
        }

        // Build hostdev XML for network VF
        let device_xml = format!(
            r#"<hostdev mode='subsystem' type='pci' managed='yes'>
  <source>
    <address domain='0x{}' bus='0x{}' slot='0x{}' function='0x{}'/>
  </source>
</hostdev>"#,
            parts[0], parts[1], parts[2], parts[3]
        );

        domain.attach_device(&device_xml)
            .map_err(|e| AppError::LibvirtError(format!("Failed to attach VF: {}", e)))?;

        tracing::info!("VF {} attached to VM {}", vf_address, vm_id);
        Ok(())
    }

    /// Detach a VF from a VM
    pub fn detach_vf_from_vm(
        libvirt: &LibvirtService,
        vm_id: &str,
        vf_address: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Detaching VF {} from VM {}", vf_address, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|e| AppError::VmNotFound(format!("{}: {}", vm_id, e)))?;

        // Parse PCI address
        let parts: Vec<&str> = vf_address.split(&[':', '.'][..]).collect();
        if parts.len() != 4 {
            return Err(AppError::InvalidConfig(format!("Invalid PCI address format: {}", vf_address)));
        }

        let device_xml = format!(
            r#"<hostdev mode='subsystem' type='pci' managed='yes'>
  <source>
    <address domain='0x{}' bus='0x{}' slot='0x{}' function='0x{}'/>
  </source>
</hostdev>"#,
            parts[0], parts[1], parts[2], parts[3]
        );

        domain.detach_device(&device_xml)
            .map_err(|e| AppError::LibvirtError(format!("Failed to detach VF: {}", e)))?;

        tracing::info!("VF {} detached from VM {}", vf_address, vm_id);
        Ok(())
    }

    // Helper functions

    fn get_pci_names(address: &str) -> (Option<String>, Option<String>) {
        // Use lspci to get vendor/device names
        let output = Command::new("lspci")
            .args(["-s", address, "-v"])
            .output()
            .ok();

        if let Some(output) = output {
            if output.status.success() {
                let lspci_output = String::from_utf8_lossy(&output.stdout);
                // Parse first line: "01:00.0 Ethernet controller: Intel Corporation ..."
                if let Some(first_line) = lspci_output.lines().next() {
                    if let Some(colon_pos) = first_line.find(": ") {
                        let description = &first_line[colon_pos + 2..];
                        // Try to split vendor and device
                        if let Some(space_pos) = description.find(" ") {
                            // Check for common patterns
                            let parts: Vec<&str> = description.splitn(2, " ").collect();
                            if parts.len() == 2 {
                                return (Some(parts[0].to_string()), Some(parts[1].to_string()));
                            }
                        }
                        return (Some(description.to_string()), None);
                    }
                }
            }
        }

        (None, None)
    }

    fn get_pf_interface(device_path: &str) -> Option<String> {
        // Look in net directory for interface name
        let net_path = format!("{}/net", device_path);
        if let Ok(entries) = fs::read_dir(&net_path) {
            for entry in entries.flatten() {
                return Some(entry.file_name().to_string_lossy().to_string());
            }
        }
        None
    }

    fn get_vf_config(pf_interface: &str, vf_index: u32) -> (Option<String>, Option<u16>) {
        // Use ip link show to get VF configuration
        let output = Command::new("ip")
            .args(["link", "show", pf_interface])
            .output()
            .ok();

        let mut mac_address = None;
        let mut vlan_id = None;

        if let Some(output) = output {
            if output.status.success() {
                let ip_output = String::from_utf8_lossy(&output.stdout);
                // Look for line with "vf N"
                let vf_pattern = format!("vf {}", vf_index);
                for line in ip_output.lines() {
                    if line.contains(&vf_pattern) {
                        // Extract MAC
                        if let Some(mac_pos) = line.find("MAC ") {
                            let mac_part = &line[mac_pos + 4..];
                            if let Some(end) = mac_part.find(|c: char| c == ',' || c.is_whitespace()) {
                                mac_address = Some(mac_part[..end].to_string());
                            }
                        }
                        // Extract VLAN
                        if let Some(vlan_pos) = line.find("vlan ") {
                            let vlan_part = &line[vlan_pos + 5..];
                            if let Some(end) = vlan_part.find(|c: char| c == ',' || c.is_whitespace()) {
                                if let Ok(vid) = vlan_part[..end].parse::<u16>() {
                                    vlan_id = Some(vid);
                                }
                            }
                        }
                    }
                }
            }
        }

        (mac_address, vlan_id)
    }

    fn count_vfs_in_use(pf_address: &str) -> u32 {
        // Count VFs that have a driver bound or are attached to a VM
        let device_path = format!("/sys/bus/pci/devices/{}", pf_address);
        let num_vfs = fs::read_to_string(format!("{}/sriov_numvfs", device_path))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0);

        let mut in_use = 0;
        for vf_index in 0..num_vfs {
            let vf_link = format!("{}/virtfn{}", device_path, vf_index);
            if let Ok(vf_path) = fs::read_link(&vf_link) {
                let vf_address = vf_path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let vf_device_path = format!("/sys/bus/pci/devices/{}", vf_address);
                // Check if VF has a driver bound (other than vfio-pci which means it's assigned to a VM)
                if Path::new(&format!("{}/driver", vf_device_path)).exists() {
                    in_use += 1;
                }
            }
        }

        in_use
    }

    fn get_vf_attachment(libvirt: &LibvirtService, vf_address: &str) -> Result<(bool, Option<String>), AppError> {
        let conn = libvirt.get_connection();

        // Check all running domains for this VF
        let flags = virt::sys::VIR_CONNECT_LIST_DOMAINS_ACTIVE;
        let domains = conn.list_all_domains(flags)
            .map_err(|e| AppError::LibvirtError(format!("Failed to list domains: {}", e)))?;

        for domain in domains {
            if let Ok(xml) = domain.get_xml_desc(0) {
                if xml.contains(vf_address) {
                    if let Ok(name) = domain.get_name() {
                        return Ok((true, Some(name)));
                    }
                }
            }
        }

        Ok((false, None))
    }
}
