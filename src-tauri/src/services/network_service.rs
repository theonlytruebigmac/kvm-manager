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
            format!(
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
            )
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
}
