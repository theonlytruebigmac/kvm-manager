use virt::network::Network;
use virt::sys;
use crate::models::network::{Network as NetworkModel, NetworkConfig};
use crate::services::libvirt::LibvirtService;
use crate::utils::error::{AppError, map_libvirt_error};

/// NetworkService provides network management operations
pub struct NetworkService;

impl NetworkService {
    /// List all virtual networks (active and inactive)
    pub fn list_networks(libvirt: &LibvirtService) -> Result<Vec<NetworkModel>, AppError> {
        tracing::debug!("Listing all networks");

        let conn = libvirt.get_connection();
        let mut networks = Vec::new();

        // Get all networks (both active and inactive)
        let flags = sys::VIR_CONNECT_LIST_NETWORKS_ACTIVE | sys::VIR_CONNECT_LIST_NETWORKS_INACTIVE;
        let libvirt_networks = conn.list_all_networks(flags)
            .map_err(map_libvirt_error)?;

        for network in libvirt_networks {
            match Self::network_to_model(&network) {
                Ok(net) => networks.push(net),
                Err(e) => {
                    tracing::warn!("Failed to convert network to model: {}", e);
                    continue;
                }
            }
        }

        tracing::info!("Found {} networks", networks.len());
        Ok(networks)
    }

    /// Get a single network by name
    pub fn get_network(libvirt: &LibvirtService, network_name: &str) -> Result<NetworkModel, AppError> {
        tracing::debug!("Getting network: {}", network_name);

        let conn = libvirt.get_connection();
        let network = Network::lookup_by_name(conn, network_name)
            .map_err(|_| AppError::NetworkNotFound(network_name.to_string()))?;

        Self::network_to_model(&network)
    }

    /// Convert a libvirt Network to our NetworkModel
    fn network_to_model(network: &Network) -> Result<NetworkModel, AppError> {
        let name = network.get_name()
            .map_err(map_libvirt_error)?;

        let uuid = network.get_uuid_string()
            .map_err(map_libvirt_error)?;

        let active = network.is_active()
            .map_err(map_libvirt_error)?;

        let autostart = network.get_autostart()
            .map_err(map_libvirt_error)?;

        // Get network XML to extract bridge name and IP range
        let xml = network.get_xml_desc(0)
            .map_err(map_libvirt_error)?;

        let (bridge, ip_range) = Self::parse_network_xml(&xml)?;

        Ok(NetworkModel {
            name,
            uuid,
            bridge,
            active,
            autostart,
            ip_range,
        })
    }

    /// Parse network XML to extract bridge name and IP range
    fn parse_network_xml(xml: &str) -> Result<(String, Option<String>), AppError> {
        // Simple XML parsing to extract bridge name and IP range
        let bridge = Self::extract_xml_value(xml, "bridge", "name")
            .unwrap_or_else(|| "virbr0".to_string());

        // Extract IP address and netmask
        let ip_address = Self::extract_xml_value(xml, "ip", "address");
        let netmask = Self::extract_xml_value(xml, "ip", "netmask");

        let ip_range = match (ip_address, netmask) {
            (Some(addr), Some(mask)) => Some(format!("{}/{}", addr, mask)),
            (Some(addr), None) => Some(addr),
            _ => None,
        };

        Ok((bridge, ip_range))
    }

    /// Extract attribute value from XML element (simple parser)
    fn extract_xml_value(xml: &str, element: &str, attribute: &str) -> Option<String> {
        let element_tag = format!("<{}", element);
        if let Some(start) = xml.find(&element_tag) {
            let slice = &xml[start..];
            let attr_pattern = format!("{}=\"", attribute);
            if let Some(attr_start) = slice.find(&attr_pattern) {
                let value_start = attr_start + attr_pattern.len();
                if let Some(value_end) = slice[value_start..].find('"') {
                    return Some(slice[value_start..value_start + value_end].to_string());
                }
            }
        }
        None
    }

    /// Create a new virtual network
    pub fn create_network(libvirt: &LibvirtService, config: NetworkConfig) -> Result<String, AppError> {
        tracing::info!("Creating network: {}", config.name);

        let conn = libvirt.get_connection();

        // Validate forward mode
        let forward_mode = match config.forward_mode.as_str() {
            "nat" | "route" | "bridge" | "isolated" => &config.forward_mode,
            _ => return Err(AppError::InvalidConfig(format!("Invalid forward mode: {}", config.forward_mode))),
        };

        // Generate network XML configuration
        let forward_xml = if forward_mode != "isolated" {
            format!("  <forward mode='{}'/>\n", forward_mode)
        } else {
            String::new()
        };

        // Bridge attributes (stp/delay) only allowed in nat, route, isolated modes, not bridge mode
        let bridge_attrs = if forward_mode == "bridge" {
            String::new()
        } else {
            " stp='on' delay='0'".to_string()
        };

        // Bridge mode networks don't have IP/DHCP config (they bridge to physical networks)
        let ip_config = if forward_mode == "bridge" {
            String::new()
        } else {
            // IPv4 configuration
            let ipv4_config = format!(
                r#"  <ip address='{}' netmask='{}'>
    <dhcp>
      <range start='{}' end='{}'/>
    </dhcp>
  </ip>
"#,
                config.ip_address,
                config.netmask,
                config.dhcp_start,
                config.dhcp_end
            );

            // IPv6 configuration (optional)
            let ipv6_config = if config.ipv6_enabled {
                if let (Some(ref ipv6_addr), Some(prefix)) = (&config.ipv6_address, config.ipv6_prefix) {
                    let dhcp_section = if let (Some(ref start), Some(ref end)) = (&config.ipv6_dhcp_start, &config.ipv6_dhcp_end) {
                        format!(
                            r#"
    <dhcp>
      <range start='{}' end='{}'/>
    </dhcp>"#,
                            start, end
                        )
                    } else {
                        String::new()
                    };

                    format!(
                        r#"  <ip family='ipv6' address='{}' prefix='{}'>{}</ip>
"#,
                        ipv6_addr, prefix, dhcp_section
                    )
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            format!("{}{}", ipv4_config, ipv6_config)
        };

        let xml = format!(
            r#"<network>
  <name>{}</name>
  <bridge name='{}'{}/>
{}{}</network>"#,
            config.name,
            config.bridge_name,
            bridge_attrs,
            forward_xml,
            ip_config
        );

        tracing::debug!("Network XML:\n{}", xml);

        // Define the network (create network configuration)
        let network = Network::define_xml(conn, &xml)
            .map_err(map_libvirt_error)?;

        // Set autostart if requested
        if config.autostart {
            network.set_autostart(true)
                .map_err(map_libvirt_error)?;
        }

        // Start the network
        network.create()
            .map_err(map_libvirt_error)?;

        let uuid = network.get_uuid_string()
            .map_err(map_libvirt_error)?;

        tracing::info!("Network created successfully: {} (UUID: {})", config.name, uuid);
        Ok(uuid)
    }

    /// Delete a virtual network
    pub fn delete_network(libvirt: &LibvirtService, network_name: &str) -> Result<(), AppError> {
        tracing::info!("Deleting network: {}", network_name);

        let conn = libvirt.get_connection();
        let network = Network::lookup_by_name(conn, network_name)
            .map_err(|_| AppError::NetworkNotFound(network_name.to_string()))?;

        // Check if network is active
        let active = network.is_active()
            .map_err(map_libvirt_error)?;

        // Destroy (stop) the network if it's active
        if active {
            network.destroy()
                .map_err(map_libvirt_error)?;
        }

        // Undefine the network (delete configuration)
        network.undefine()
            .map_err(map_libvirt_error)?;

        tracing::info!("Network deleted successfully: {}", network_name);
        Ok(())
    }

    /// Start a network
    pub fn start_network(libvirt: &LibvirtService, network_name: &str) -> Result<(), AppError> {
        tracing::info!("Starting network: {}", network_name);

        let conn = libvirt.get_connection();
        let network = Network::lookup_by_name(conn, network_name)
            .map_err(|_| AppError::NetworkNotFound(network_name.to_string()))?;

        let active = network.is_active()
            .map_err(map_libvirt_error)?;

        if active {
            return Err(AppError::InvalidNetworkState("Network is already active".to_string()));
        }

        network.create()
            .map_err(map_libvirt_error)?;

        tracing::info!("Network started successfully: {}", network_name);
        Ok(())
    }

    /// Stop a network
    pub fn stop_network(libvirt: &LibvirtService, network_name: &str) -> Result<(), AppError> {
        tracing::info!("Stopping network: {}", network_name);

        let conn = libvirt.get_connection();
        let network = Network::lookup_by_name(conn, network_name)
            .map_err(|_| AppError::NetworkNotFound(network_name.to_string()))?;

        let active = network.is_active()
            .map_err(map_libvirt_error)?;

        if !active {
            return Err(AppError::InvalidNetworkState("Network is not active".to_string()));
        }

        network.destroy()
            .map_err(map_libvirt_error)?;

        tracing::info!("Network stopped successfully: {}", network_name);
        Ok(())
    }

    /// Add a port forwarding rule using iptables
    pub fn add_port_forward(
        host_port: u16,
        guest_ip: &str,
        guest_port: u16,
        protocol: &str,
    ) -> Result<(), AppError> {
        tracing::info!(
            "Adding port forward: {}:{} -> {}:{}",
            protocol, host_port, guest_ip, guest_port
        );

        // Validate protocol
        if protocol != "tcp" && protocol != "udp" {
            return Err(AppError::InvalidConfig(format!(
                "Invalid protocol '{}'. Must be 'tcp' or 'udp'",
                protocol
            )));
        }

        // Add DNAT rule using iptables
        let dnat_cmd = format!(
            "iptables -t nat -A PREROUTING -p {} --dport {} -j DNAT --to-destination {}:{}",
            protocol, host_port, guest_ip, guest_port
        );

        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&dnat_cmd)
            .output()
            .map_err(|e| AppError::Other(format!("Failed to execute iptables: {}", e)))?;

        if !output.status.success() {
            return Err(AppError::Other(format!(
                "iptables command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Add FORWARD rule to allow forwarded traffic
        let forward_cmd = format!(
            "iptables -A FORWARD -p {} -d {} --dport {} -j ACCEPT",
            protocol, guest_ip, guest_port
        );

        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&forward_cmd)
            .output()
            .map_err(|e| AppError::Other(format!("Failed to execute iptables: {}", e)))?;

        if !output.status.success() {
            return Err(AppError::Other(format!(
                "iptables FORWARD command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        tracing::info!("Port forward rule added successfully");
        Ok(())
    }

    /// Remove a port forwarding rule
    pub fn remove_port_forward(
        host_port: u16,
        guest_ip: &str,
        guest_port: u16,
        protocol: &str,
    ) -> Result<(), AppError> {
        tracing::info!(
            "Removing port forward: {}:{} -> {}:{}",
            protocol, host_port, guest_ip, guest_port
        );

        // Validate protocol
        if protocol != "tcp" && protocol != "udp" {
            return Err(AppError::InvalidConfig(format!(
                "Invalid protocol '{}'. Must be 'tcp' or 'udp'",
                protocol
            )));
        }

        // Remove DNAT rule
        let dnat_cmd = format!(
            "iptables -t nat -D PREROUTING -p {} --dport {} -j DNAT --to-destination {}:{}",
            protocol, host_port, guest_ip, guest_port
        );

        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&dnat_cmd)
            .output()
            .map_err(|e| AppError::Other(format!("Failed to execute iptables: {}", e)))?;

        if !output.status.success() {
            tracing::warn!(
                "iptables DNAT delete failed (rule may not exist): {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // Remove FORWARD rule
        let forward_cmd = format!(
            "iptables -D FORWARD -p {} -d {} --dport {} -j ACCEPT",
            protocol, guest_ip, guest_port
        );

        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&forward_cmd)
            .output()
            .map_err(|e| AppError::Other(format!("Failed to execute iptables: {}", e)))?;

        if !output.status.success() {
            tracing::warn!(
                "iptables FORWARD delete failed (rule may not exist): {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        tracing::info!("Port forward rule removed");
        Ok(())
    }

    /// Set network autostart
    pub fn set_network_autostart(libvirt: &LibvirtService, network_name: &str, autostart: bool) -> Result<(), AppError> {
        tracing::info!("Setting network {} autostart to {}", network_name, autostart);

        let conn = libvirt.get_connection();
        let network = Network::lookup_by_name(conn, network_name)
            .map_err(|_| AppError::NetworkNotFound(network_name.to_string()))?;

        network.set_autostart(autostart)
            .map_err(map_libvirt_error)?;

        tracing::info!("Network autostart set successfully");
        Ok(())
    }

    /// Get DHCP leases for a network by reading dnsmasq lease file
    pub fn get_dhcp_leases(libvirt: &LibvirtService, network_name: &str) -> Result<Vec<DhcpLease>, AppError> {
        tracing::debug!("Getting DHCP leases for network: {}", network_name);

        let conn = libvirt.get_connection();
        let _network = Network::lookup_by_name(conn, network_name)
            .map_err(|_| AppError::NetworkNotFound(network_name.to_string()))?;

        // Read leases from dnsmasq lease file
        // The file is typically at /var/lib/libvirt/dnsmasq/<network-name>.leases
        let lease_file = format!("/var/lib/libvirt/dnsmasq/{}.leases", network_name);

        let mut leases = Vec::new();

        if let Ok(content) = std::fs::read_to_string(&lease_file) {
            for line in content.lines() {
                // dnsmasq lease format: <expiry> <mac> <ip> <hostname> <client-id>
                // Example: 1234567890 00:16:3e:xx:xx:xx 192.168.122.100 myvm *
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let expiry_time = parts[0].parse::<i64>().unwrap_or(0);
                    let mac = parts[1].to_string();
                    let ip_address = parts[2].to_string();
                    let hostname = if parts[3] != "*" { Some(parts[3].to_string()) } else { None };
                    let client_id = if parts.len() >= 5 && parts[4] != "*" {
                        Some(parts[4].to_string())
                    } else {
                        None
                    };

                    leases.push(DhcpLease {
                        mac,
                        ip_address,
                        hostname,
                        client_id,
                        expiry_time,
                    });
                }
            }
        } else {
            tracing::debug!("Lease file not found or not readable: {}", lease_file);
        }

        tracing::debug!("Found {} DHCP leases for network {}", leases.len(), network_name);
        Ok(leases)
    }

    /// Get detailed network information including DHCP config
    pub fn get_network_details(libvirt: &LibvirtService, network_name: &str) -> Result<NetworkDetails, AppError> {
        tracing::debug!("Getting detailed network info for: {}", network_name);

        let conn = libvirt.get_connection();
        let network = Network::lookup_by_name(conn, network_name)
            .map_err(|_| AppError::NetworkNotFound(network_name.to_string()))?;

        let xml = network.get_xml_desc(0)
            .map_err(map_libvirt_error)?;

        // Parse network details from XML
        let forward_mode = Self::extract_xml_value(&xml, "forward", "mode")
            .unwrap_or_else(|| "isolated".to_string());

        let ip_address = Self::extract_xml_value(&xml, "ip", "address");
        let netmask = Self::extract_xml_value(&xml, "ip", "netmask");
        let dhcp_start = Self::extract_dhcp_range_value(&xml, "start");
        let dhcp_end = Self::extract_dhcp_range_value(&xml, "end");

        let basic = Self::network_to_model(&network)?;

        // Get DHCP leases
        let leases = Self::get_dhcp_leases(libvirt, network_name).unwrap_or_default();

        Ok(NetworkDetails {
            name: basic.name,
            uuid: basic.uuid,
            bridge: basic.bridge,
            active: basic.active,
            autostart: basic.autostart,
            ip_range: basic.ip_range,
            forward_mode,
            ip_address,
            netmask,
            dhcp_start,
            dhcp_end,
            dhcp_leases: leases,
        })
    }

    /// Extract DHCP range start/end from XML
    fn extract_dhcp_range_value(xml: &str, attribute: &str) -> Option<String> {
        // Look for <range start='...' end='...'/> inside <dhcp>
        if let Some(dhcp_start) = xml.find("<dhcp>") {
            let dhcp_section = &xml[dhcp_start..];
            if let Some(range_start) = dhcp_section.find("<range") {
                let range_section = &dhcp_section[range_start..];
                let attr_pattern = format!("{}='", attribute);
                if let Some(attr_start) = range_section.find(&attr_pattern) {
                    let value_start = attr_start + attr_pattern.len();
                    if let Some(value_end) = range_section[value_start..].find('\'') {
                        return Some(range_section[value_start..value_start + value_end].to_string());
                    }
                }
            }
        }
        None
    }
}

/// DHCP Lease information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DhcpLease {
    pub mac: String,
    pub ip_address: String,
    pub hostname: Option<String>,
    pub client_id: Option<String>,
    pub expiry_time: i64,
}

/// Detailed network information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkDetails {
    pub name: String,
    pub uuid: String,
    pub bridge: String,
    pub active: bool,
    pub autostart: bool,
    pub ip_range: Option<String>,
    pub forward_mode: String,
    pub ip_address: Option<String>,
    pub netmask: Option<String>,
    pub dhcp_start: Option<String>,
    pub dhcp_end: Option<String>,
    pub dhcp_leases: Vec<DhcpLease>,
}
