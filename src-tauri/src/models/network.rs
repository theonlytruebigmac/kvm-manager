use serde::{Deserialize, Serialize};

/// Virtual Network model
/// Represents a libvirt virtual network
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Network {
    pub name: String,
    pub uuid: String,
    pub bridge: String,
    pub active: bool,
    pub autostart: bool,
    pub ip_range: Option<String>,
}

/// Network Configuration for creating new networks
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NetworkConfig {
    pub name: String,
    pub bridge_name: String,
    pub forward_mode: String,
    pub ip_address: String,
    pub netmask: String,
    pub dhcp_start: String,
    pub dhcp_end: String,
    #[serde(default)]
    pub autostart: bool,
}
