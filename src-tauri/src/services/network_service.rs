use crate::models::network::{Network as NetworkModel, NetworkConfig};
use crate::services::libvirt::ConnectionProvider;
use crate::utils::error::{map_libvirt_error, AppError};
use crate::utils::xml::{
    escaped_attribute, first_element_attribute, validate_identifier, validate_text,
    xml_text_element,
};
use std::net::{Ipv4Addr, Ipv6Addr};
use virt::network::Network;
use virt::sys;

/// NetworkService provides network management operations
pub struct NetworkService;

impl NetworkService {
    /// List all virtual networks (active and inactive)
    pub fn list_networks(libvirt: &impl ConnectionProvider) -> Result<Vec<NetworkModel>, AppError> {
        tracing::debug!("Listing all networks");

        let conn = libvirt.get_connection();
        let mut networks = Vec::new();

        // Get all networks (both active and inactive)
        let flags = sys::VIR_CONNECT_LIST_NETWORKS_ACTIVE | sys::VIR_CONNECT_LIST_NETWORKS_INACTIVE;
        let libvirt_networks = conn.list_all_networks(flags).map_err(map_libvirt_error)?;

        for network in libvirt_networks {
            match Self::network_to_model(&network) {
                Ok(net) => networks.push(net),
                Err(_) => {
                    tracing::warn!("A network could not be read");
                    continue;
                }
            }
        }

        tracing::info!("Found {} networks", networks.len());
        Ok(networks)
    }

    /// Get a single network by name
    pub fn get_network(
        libvirt: &impl ConnectionProvider,
        network_name: &str,
    ) -> Result<NetworkModel, AppError> {
        tracing::debug!("Getting network: {}", network_name);

        let conn = libvirt.get_connection();
        let network = Network::lookup_by_name(conn, network_name)
            .map_err(|_| AppError::NetworkNotFound(network_name.to_string()))?;

        Self::network_to_model(&network)
    }

    /// Convert a libvirt Network to our NetworkModel
    fn network_to_model(network: &Network) -> Result<NetworkModel, AppError> {
        let name = network.get_name().map_err(map_libvirt_error)?;

        let uuid = network.get_uuid_string().map_err(map_libvirt_error)?;

        let active = network.is_active().map_err(map_libvirt_error)?;

        let autostart = network.get_autostart().map_err(map_libvirt_error)?;

        // Get network XML to extract bridge name and IP range
        let xml = network.get_xml_desc(0).map_err(map_libvirt_error)?;

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
        let bridge = first_element_attribute(xml, "network", "bridge", "name")?
            .unwrap_or_else(|| "virbr0".to_string());
        let ip_address = first_element_attribute(xml, "network", "ip", "address")?;
        let netmask = first_element_attribute(xml, "network", "ip", "netmask")?;

        let ip_range = match (ip_address, netmask) {
            (Some(addr), Some(mask)) => Some(format!("{}/{}", addr, mask)),
            (Some(addr), None) => Some(addr),
            _ => None,
        };

        Ok((bridge, ip_range))
    }

    /// Create a new virtual network
    pub fn create_network(
        libvirt: &impl ConnectionProvider,
        config: NetworkConfig,
    ) -> Result<String, AppError> {
        tracing::info!("Creating network: {}", config.name);

        let conn = libvirt.get_connection();

        let xml = Self::network_definition(&config)?;

        tracing::debug!("Generated network definition");

        // Define the network (create network configuration)
        let network = Network::define_xml(conn, &xml).map_err(map_libvirt_error)?;

        // Set autostart if requested
        if config.autostart {
            network.set_autostart(true).map_err(map_libvirt_error)?;
        }

        // Start the network
        network.create().map_err(map_libvirt_error)?;

        let uuid = network.get_uuid_string().map_err(map_libvirt_error)?;

        tracing::info!(
            "Network created successfully: {} (UUID: {})",
            config.name,
            uuid
        );
        Ok(uuid)
    }

    /// Builds a libvirt network definition after validating each value at the configuration
    /// boundary. Keeping this pure makes rejection testable before any libvirt mutation begins.
    pub fn network_definition(config: &NetworkConfig) -> Result<String, AppError> {
        validate_network_name(&config.name, "network name")?;
        validate_network_name(&config.bridge_name, "bridge name")?;
        let forward_mode = match config.forward_mode.as_str() {
            "nat" | "route" | "bridge" | "isolated" => config.forward_mode.as_str(),
            _ => return Err(AppError::InvalidConfig("Invalid forward mode".to_string())),
        };

        let mut xml = String::from("<network>\n  ");
        xml.push_str(&xml_text_element("name", &config.name)?);
        xml.push_str("\n  <bridge name='");
        xml.push_str(&escaped_attribute(&config.bridge_name, "bridge name")?);
        xml.push('\'');
        if forward_mode != "bridge" {
            xml.push_str(" stp='on' delay='0'");
        }
        xml.push_str("/>\n");

        if forward_mode != "isolated" {
            xml.push_str("  <forward mode='");
            xml.push_str(forward_mode);
            xml.push_str("'/>\n");
        }

        if forward_mode != "bridge" {
            let ipv4_address = parse_ipv4(&config.ip_address, "IPv4 address")?;
            let netmask = parse_ipv4(&config.netmask, "IPv4 netmask")?;
            let dhcp_start = parse_ipv4(&config.dhcp_start, "IPv4 DHCP start")?;
            let dhcp_end = parse_ipv4(&config.dhcp_end, "IPv4 DHCP end")?;
            if u32::from(dhcp_start) > u32::from(dhcp_end) {
                return Err(AppError::InvalidConfig(
                    "IPv4 DHCP range start must not follow its end".to_string(),
                ));
            }

            xml.push_str(&format!(
                "  <ip address='{}' netmask='{}'><dhcp><range start='{}' end='{}'/></dhcp></ip>\n",
                ipv4_address, netmask, dhcp_start, dhcp_end
            ));

            if config.ipv6_enabled {
                let address = config.ipv6_address.as_deref().ok_or_else(|| {
                    AppError::InvalidConfig(
                        "IPv6 address is required when IPv6 is enabled".to_string(),
                    )
                })?;
                let prefix = config.ipv6_prefix.ok_or_else(|| {
                    AppError::InvalidConfig(
                        "IPv6 prefix is required when IPv6 is enabled".to_string(),
                    )
                })?;
                if prefix > 128 {
                    return Err(AppError::InvalidConfig(
                        "IPv6 prefix must be at most 128".to_string(),
                    ));
                }
                let ipv6_address = parse_ipv6(address, "IPv6 address")?;
                let (dhcp_start, dhcp_end) = match (
                    config.ipv6_dhcp_start.as_deref(),
                    config.ipv6_dhcp_end.as_deref(),
                ) {
                    (None, None) => (None, None),
                    (Some(start), Some(end)) => {
                        let start = parse_ipv6(start, "IPv6 DHCP start")?;
                        let end = parse_ipv6(end, "IPv6 DHCP end")?;
                        if u128::from(start) > u128::from(end) {
                            return Err(AppError::InvalidConfig(
                                "IPv6 DHCP range start must not follow its end".to_string(),
                            ));
                        }
                        (Some(start), Some(end))
                    }
                    _ => {
                        return Err(AppError::InvalidConfig(
                            "Both IPv6 DHCP range bounds are required".to_string(),
                        ))
                    }
                };

                xml.push_str(&format!(
                    "  <ip family='ipv6' address='{}' prefix='{}'",
                    ipv6_address, prefix
                ));
                if let (Some(start), Some(end)) = (dhcp_start, dhcp_end) {
                    xml.push_str(&format!(
                        "><dhcp><range start='{}' end='{}'/></dhcp></ip>\n",
                        start, end
                    ));
                } else {
                    xml.push_str("/>\n");
                }
            }
        }

        xml.push_str("</network>");
        crate::utils::xml::validate_document_root(&xml, "network")?;
        Ok(xml)
    }

    /// Delete a virtual network
    pub fn delete_network(
        libvirt: &impl ConnectionProvider,
        network_name: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Deleting network: {}", network_name);

        let conn = libvirt.get_connection();
        let network = Network::lookup_by_name(conn, network_name)
            .map_err(|_| AppError::NetworkNotFound(network_name.to_string()))?;

        // Check if network is active
        let active = network.is_active().map_err(map_libvirt_error)?;

        // Destroy (stop) the network if it's active
        if active {
            network.destroy().map_err(map_libvirt_error)?;
        }

        // Undefine the network (delete configuration)
        network.undefine().map_err(map_libvirt_error)?;

        tracing::info!("Network deleted successfully: {}", network_name);
        Ok(())
    }

    /// Start a network
    pub fn start_network(
        libvirt: &impl ConnectionProvider,
        network_name: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Starting network: {}", network_name);

        let conn = libvirt.get_connection();
        let network = Network::lookup_by_name(conn, network_name)
            .map_err(|_| AppError::NetworkNotFound(network_name.to_string()))?;

        let active = network.is_active().map_err(map_libvirt_error)?;

        if active {
            return Err(AppError::InvalidNetworkState(
                "Network is already active".to_string(),
            ));
        }

        network.create().map_err(map_libvirt_error)?;

        tracing::info!("Network started successfully: {}", network_name);
        Ok(())
    }

    /// Stop a network
    pub fn stop_network(
        libvirt: &impl ConnectionProvider,
        network_name: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Stopping network: {}", network_name);

        let conn = libvirt.get_connection();
        let network = Network::lookup_by_name(conn, network_name)
            .map_err(|_| AppError::NetworkNotFound(network_name.to_string()))?;

        let active = network.is_active().map_err(map_libvirt_error)?;

        if !active {
            return Err(AppError::InvalidNetworkState(
                "Network is not active".to_string(),
            ));
        }

        network.destroy().map_err(map_libvirt_error)?;

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
            protocol,
            host_port,
            guest_ip,
            guest_port
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
            return Err(AppError::Other(
                "iptables could not add the requested forwarding rule".to_string(),
            ));
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
            return Err(AppError::Other(
                "iptables could not add the requested forwarding rule".to_string(),
            ));
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
            protocol,
            host_port,
            guest_ip,
            guest_port
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
            tracing::warn!("iptables DNAT delete failed; the rule may not exist");
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
            tracing::warn!("iptables FORWARD delete failed; the rule may not exist");
        }

        tracing::info!("Port forward rule removed");
        Ok(())
    }

    /// Set network autostart
    pub fn set_network_autostart(
        libvirt: &impl ConnectionProvider,
        network_name: &str,
        autostart: bool,
    ) -> Result<(), AppError> {
        tracing::info!(
            "Setting network {} autostart to {}",
            network_name,
            autostart
        );

        let conn = libvirt.get_connection();
        let network = Network::lookup_by_name(conn, network_name)
            .map_err(|_| AppError::NetworkNotFound(network_name.to_string()))?;

        network
            .set_autostart(autostart)
            .map_err(map_libvirt_error)?;

        tracing::info!("Network autostart set successfully");
        Ok(())
    }

    /// Get DHCP leases for a network by reading dnsmasq lease file
    pub fn get_dhcp_leases(
        libvirt: &impl ConnectionProvider,
        network_name: &str,
    ) -> Result<Vec<DhcpLease>, AppError> {
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
                    let hostname = if parts[3] != "*" {
                        Some(parts[3].to_string())
                    } else {
                        None
                    };
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
            tracing::debug!("Lease data is not available");
        }

        tracing::debug!(
            "Found {} DHCP leases for network {}",
            leases.len(),
            network_name
        );
        Ok(leases)
    }

    /// Get detailed network information including DHCP config
    pub fn get_network_details(
        libvirt: &impl ConnectionProvider,
        network_name: &str,
    ) -> Result<NetworkDetails, AppError> {
        tracing::debug!("Getting detailed network info for: {}", network_name);

        let conn = libvirt.get_connection();
        let network = Network::lookup_by_name(conn, network_name)
            .map_err(|_| AppError::NetworkNotFound(network_name.to_string()))?;

        let xml = network.get_xml_desc(0).map_err(map_libvirt_error)?;

        // Parse network details from XML events rather than quote-style-specific substrings.
        let forward_mode = first_element_attribute(&xml, "network", "forward", "mode")?
            .unwrap_or_else(|| "isolated".to_string());

        let ip_address = first_element_attribute(&xml, "network", "ip", "address")?;
        let netmask = first_element_attribute(&xml, "network", "ip", "netmask")?;
        let dhcp_start = first_element_attribute(&xml, "network", "range", "start")?;
        let dhcp_end = first_element_attribute(&xml, "network", "range", "end")?;

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
}

fn validate_network_name(value: &str, field: &str) -> Result<(), AppError> {
    validate_identifier(value, field)?;
    validate_text(value, field)?;
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.'))
    {
        return Err(AppError::InvalidConfig(format!(
            "{} contains unsupported characters",
            field
        )));
    }
    Ok(())
}

fn parse_ipv4(value: &str, field: &str) -> Result<Ipv4Addr, AppError> {
    value
        .parse()
        .map_err(|_| AppError::InvalidConfig(format!("{} is invalid", field)))
}

fn parse_ipv6(value: &str, field: &str) -> Result<Ipv6Addr, AppError> {
    value
        .parse()
        .map_err(|_| AppError::InvalidConfig(format!("{} is invalid", field)))
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
