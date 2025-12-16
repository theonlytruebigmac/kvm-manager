use serde::{Deserialize, Serialize};

/// Network filter model representing a libvirt nwfilter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NwFilter {
    /// Filter UUID
    pub uuid: String,
    /// Filter name
    pub name: String,
    /// Number of rules in this filter
    pub rule_count: usize,
    /// Filter chain type (e.g., "root", "ipv4", "arp", etc.)
    pub chain: Option<String>,
    /// Priority (if specified)
    pub priority: Option<i32>,
}

/// Filter rule direction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleDirection {
    In,
    Out,
    InOut,
}

impl Default for RuleDirection {
    fn default() -> Self {
        RuleDirection::InOut
    }
}

/// Filter rule action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    Accept,
    Drop,
    Reject,
    Return,
    Continue,
}

impl Default for RuleAction {
    fn default() -> Self {
        RuleAction::Accept
    }
}

/// Network filter rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NwFilterRule {
    /// Rule direction
    pub direction: RuleDirection,
    /// Rule action
    pub action: RuleAction,
    /// Rule priority (0-1000)
    pub priority: Option<i32>,
    /// Protocol (tcp, udp, icmp, all, etc.)
    pub protocol: Option<String>,
    /// Source IP address or CIDR
    pub src_ip: Option<String>,
    /// Source MAC address
    pub src_mac: Option<String>,
    /// Destination IP address or CIDR
    pub dest_ip: Option<String>,
    /// Destination MAC address
    pub dest_mac: Option<String>,
    /// Source port or port range
    pub src_port: Option<String>,
    /// Destination port or port range
    pub dest_port: Option<String>,
    /// Comment/description
    pub comment: Option<String>,
}

/// Configuration for creating a new network filter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NwFilterConfig {
    /// Filter name
    pub name: String,
    /// Filter chain (root, ipv4, ipv6, arp, rarp, etc.)
    pub chain: Option<String>,
    /// Filter priority (-1000 to 1000)
    pub priority: Option<i32>,
    /// Rules to include
    pub rules: Vec<NwFilterRule>,
    /// Reference to other filters to include
    pub filter_refs: Vec<String>,
}
