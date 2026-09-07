use crate::models::nwfilter::{NwFilter, NwFilterConfig, NwFilterRule, RuleAction, RuleDirection};
use crate::services::libvirt::ConnectionProvider;
use crate::utils::error::{map_libvirt_error, AppError};
use crate::utils::xml::{
    count_elements, escaped_attribute, first_element_attribute, validate_identifier, validate_text,
};
use std::net::IpAddr;
use virt::nwfilter::NWFilter;

/// NwFilterService provides network filter management operations
pub struct NwFilterService;

impl NwFilterService {
    /// List all network filters
    pub fn list_filters(libvirt: &impl ConnectionProvider) -> Result<Vec<NwFilter>, AppError> {
        tracing::debug!("Listing all network filters");

        let conn = libvirt.get_connection();
        let filters = conn.list_all_nw_filters(0).map_err(map_libvirt_error)?;

        let mut result = Vec::new();
        for filter in filters {
            match Self::filter_to_model(&filter) {
                Ok(model) => result.push(model),
                Err(_) => {
                    tracing::warn!("A network filter could not be read");
                    continue;
                }
            }
        }

        tracing::info!("Found {} network filters", result.len());
        Ok(result)
    }

    /// Get a network filter by name
    pub fn get_filter(libvirt: &impl ConnectionProvider, name: &str) -> Result<NwFilter, AppError> {
        tracing::debug!("Getting network filter: {}", name);

        let conn = libvirt.get_connection();
        let filter = NWFilter::lookup_by_name(conn, name).map_err(map_libvirt_error)?;

        Self::filter_to_model(&filter)
    }

    /// Get the XML definition of a network filter
    pub fn get_filter_xml(
        libvirt: &impl ConnectionProvider,
        name: &str,
    ) -> Result<String, AppError> {
        tracing::debug!("Getting XML for network filter: {}", name);

        let conn = libvirt.get_connection();
        let filter = NWFilter::lookup_by_name(conn, name).map_err(map_libvirt_error)?;

        filter.get_xml_desc(0).map_err(map_libvirt_error)
    }

    /// Create a new network filter from config
    pub fn create_filter(
        libvirt: &impl ConnectionProvider,
        config: NwFilterConfig,
    ) -> Result<NwFilter, AppError> {
        tracing::info!("Creating network filter: {}", config.name);

        let xml = Self::config_to_xml(&config)?;
        tracing::debug!("Generated network filter definition");

        let conn = libvirt.get_connection();
        let filter = NWFilter::define_xml(conn, &xml).map_err(map_libvirt_error)?;

        Self::filter_to_model(&filter)
    }

    /// Create a network filter from raw XML
    pub fn create_filter_from_xml(
        libvirt: &impl ConnectionProvider,
        xml: &str,
    ) -> Result<NwFilter, AppError> {
        tracing::info!("Creating network filter from XML");

        crate::utils::xml::validate_document_root(xml, "filter")?;

        let conn = libvirt.get_connection();
        let filter = NWFilter::define_xml(conn, xml).map_err(map_libvirt_error)?;

        Self::filter_to_model(&filter)
    }

    /// Delete a network filter
    pub fn delete_filter(libvirt: &impl ConnectionProvider, name: &str) -> Result<(), AppError> {
        tracing::info!("Deleting network filter: {}", name);

        let conn = libvirt.get_connection();
        let filter = NWFilter::lookup_by_name(conn, name).map_err(map_libvirt_error)?;

        filter.undefine().map_err(map_libvirt_error)?;

        tracing::info!("Network filter deleted: {}", name);
        Ok(())
    }

    /// Convert a libvirt NWFilter to our model
    fn filter_to_model(filter: &NWFilter) -> Result<NwFilter, AppError> {
        let uuid = filter.get_uuid_string().map_err(map_libvirt_error)?;

        let name = filter.get_name().map_err(map_libvirt_error)?;

        // Get XML to extract more info
        let xml = filter.get_xml_desc(0).map_err(map_libvirt_error)?;

        let rule_count = Self::count_rules(&xml)?;
        let chain = Self::extract_chain(&xml)?;
        let priority = Self::extract_priority(&xml)?;

        Ok(NwFilter {
            uuid,
            name,
            rule_count,
            chain,
            priority,
        })
    }

    /// Count rules in XML
    fn count_rules(xml: &str) -> Result<usize, AppError> {
        count_elements(xml, "filter", "rule")
    }

    /// Extract chain type from XML
    fn extract_chain(xml: &str) -> Result<Option<String>, AppError> {
        first_element_attribute(xml, "filter", "filter", "chain")
    }

    /// Extract priority from XML
    fn extract_priority(xml: &str) -> Result<Option<i32>, AppError> {
        first_element_attribute(xml, "filter", "filter", "priority")?
            .map(|value| {
                value
                    .parse()
                    .map_err(|_| AppError::InvalidConfig("Invalid filter priority".to_string()))
            })
            .transpose()
    }

    /// Convert config to XML
    pub fn config_to_xml(config: &NwFilterConfig) -> Result<String, AppError> {
        validate_filter_identifier(&config.name, "filter name")?;
        if config.rules.len() > 1024 || config.filter_refs.len() > 1024 {
            return Err(AppError::InvalidConfig(
                "Filter contains too many rules or references".to_string(),
            ));
        }

        let mut xml = String::from("<?xml version='1.0' encoding='UTF-8'?>\n<filter name='");
        xml.push_str(&escaped_attribute(&config.name, "filter name")?);
        xml.push('\'');

        if let Some(chain) = &config.chain {
            validate_filter_identifier(chain, "filter chain")?;
            xml.push_str(" chain='");
            xml.push_str(&escaped_attribute(chain, "filter chain")?);
            xml.push('\'');
        }

        if let Some(priority) = config.priority {
            if !(-1000..=1000).contains(&priority) {
                return Err(AppError::InvalidConfig(
                    "Filter priority must be between -1000 and 1000".to_string(),
                ));
            }
            xml.push_str(&format!(" priority='{}'", priority));
        }

        xml.push_str(">\n");

        for filter_ref in &config.filter_refs {
            validate_filter_identifier(filter_ref, "filter reference")?;
            xml.push_str("  <filterref filter='");
            xml.push_str(&escaped_attribute(filter_ref, "filter reference")?);
            xml.push_str("'/>\n");
        }

        for rule in &config.rules {
            xml.push_str(&Self::rule_to_xml(rule)?);
        }

        xml.push_str("</filter>\n");
        crate::utils::xml::validate_document_root(&xml, "filter")?;
        Ok(xml)
    }

    /// Convert a rule to XML
    fn rule_to_xml(rule: &NwFilterRule) -> Result<String, AppError> {
        let direction = match rule.direction {
            RuleDirection::In => "in",
            RuleDirection::Out => "out",
            RuleDirection::InOut => "inout",
        };

        let action = match rule.action {
            RuleAction::Accept => "accept",
            RuleAction::Drop => "drop",
            RuleAction::Reject => "reject",
            RuleAction::Return => "return",
            RuleAction::Continue => "continue",
        };

        let mut xml = format!("  <rule action='{}' direction='{}'", action, direction);

        if let Some(priority) = rule.priority {
            if !(-1000..=1000).contains(&priority) {
                return Err(AppError::InvalidConfig(
                    "Rule priority must be between -1000 and 1000".to_string(),
                ));
            }
            xml.push_str(&format!(" priority='{}'", priority));
        }

        xml.push_str(">\n");

        // Determine protocol element
        let protocol = rule.protocol.as_deref().unwrap_or("all");
        if !SUPPORTED_PROTOCOLS.contains(&protocol) {
            return Err(AppError::InvalidConfig(
                "Unsupported filter protocol".to_string(),
            ));
        }

        if protocol == "all" {
            if rule.src_ip.is_some()
                || rule.dest_ip.is_some()
                || rule.src_mac.is_some()
                || rule.dest_mac.is_some()
                || rule.src_port.is_some()
                || rule.dest_port.is_some()
            {
                return Err(AppError::InvalidConfig(
                    "An all-protocol rule cannot include protocol-specific fields".to_string(),
                ));
            }
            if let Some(comment) = &rule.comment {
                xml.push_str("    <!-- ");
                xml.push_str(&safe_xml_comment(comment)?);
                xml.push_str(" -->\n");
            }
        } else {
            xml.push_str(&format!("    <{}", protocol));

            if let Some(src_ip) = &rule.src_ip {
                xml.push_str(" srcipaddr='");
                xml.push_str(&escaped_attribute(
                    &validated_ip_or_cidr(src_ip, "source IP")?,
                    "source IP",
                )?);
                xml.push('\'');
            }
            if let Some(dest_ip) = &rule.dest_ip {
                xml.push_str(" dstipaddr='");
                xml.push_str(&escaped_attribute(
                    &validated_ip_or_cidr(dest_ip, "destination IP")?,
                    "destination IP",
                )?);
                xml.push('\'');
            }
            if let Some(src_mac) = &rule.src_mac {
                xml.push_str(" srcmacaddr='");
                xml.push_str(&escaped_attribute(
                    &validated_mac(src_mac, "source MAC")?,
                    "source MAC",
                )?);
                xml.push('\'');
            }
            if let Some(dest_mac) = &rule.dest_mac {
                xml.push_str(" dstmacaddr='");
                xml.push_str(&escaped_attribute(
                    &validated_mac(dest_mac, "destination MAC")?,
                    "destination MAC",
                )?);
                xml.push('\'');
            }
            if let Some(src_port) = &rule.src_port {
                let (start, end) = validated_port_range(src_port, "source port")?;
                xml.push_str(&format!(" srcportstart='{}'", start));
                if let Some(end) = end {
                    xml.push_str(&format!(" srcportend='{}'", end));
                }
            }
            if let Some(dest_port) = &rule.dest_port {
                let (start, end) = validated_port_range(dest_port, "destination port")?;
                xml.push_str(&format!(" dstportstart='{}'", start));
                if let Some(end) = end {
                    xml.push_str(&format!(" dstportend='{}'", end));
                }
            }
            if let Some(comment) = &rule.comment {
                validate_text(comment, "rule comment")?;
                xml.push_str(" comment='");
                xml.push_str(&escaped_attribute(comment, "rule comment")?);
                xml.push('\'');
            }

            xml.push_str("/>\n");
        }

        xml.push_str("  </rule>\n");
        Ok(xml)
    }
}

const SUPPORTED_PROTOCOLS: &[&str] = &[
    "all", "arp", "rarp", "ipv4", "ipv6", "tcp", "udp", "udplite", "sctp", "icmp", "igmp", "esp",
    "ah", "mac", "vlan", "stp",
];

fn validate_filter_identifier(value: &str, field: &str) -> Result<(), AppError> {
    validate_identifier(value, field)?;
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

fn validated_ip_or_cidr(value: &str, field: &str) -> Result<String, AppError> {
    let (address, prefix) = match value.split_once('/') {
        Some((address, prefix)) if !prefix.contains('/') => (address, Some(prefix)),
        Some(_) => return Err(AppError::InvalidConfig(format!("{} is invalid", field))),
        None => (value, None),
    };
    let address: IpAddr = address
        .parse()
        .map_err(|_| AppError::InvalidConfig(format!("{} is invalid", field)))?;
    if let Some(prefix) = prefix {
        let prefix: u8 = prefix
            .parse()
            .map_err(|_| AppError::InvalidConfig(format!("{} is invalid", field)))?;
        let maximum = match address {
            IpAddr::V4(_) => 32,
            IpAddr::V6(_) => 128,
        };
        if prefix > maximum {
            return Err(AppError::InvalidConfig(format!("{} is invalid", field)));
        }
        Ok(format!("{}/{}", address, prefix))
    } else {
        Ok(address.to_string())
    }
}

fn validated_mac(value: &str, field: &str) -> Result<String, AppError> {
    let parts = value.split(':').collect::<Vec<_>>();
    if parts.len() != 6
        || parts.iter().any(|part| {
            part.len() != 2 || !part.chars().all(|character| character.is_ascii_hexdigit())
        })
    {
        return Err(AppError::InvalidConfig(format!("{} is invalid", field)));
    }
    Ok(parts.join(":"))
}

fn validated_port_range(value: &str, field: &str) -> Result<(u16, Option<u16>), AppError> {
    let (start, end) = match value.split_once('-') {
        Some((start, end)) if !end.contains('-') => (start, Some(end)),
        Some(_) => return Err(AppError::InvalidConfig(format!("{} is invalid", field))),
        None => (value, None),
    };
    let start: u16 = start
        .parse()
        .map_err(|_| AppError::InvalidConfig(format!("{} is invalid", field)))?;
    let end = end
        .map(|value| {
            value
                .parse()
                .map_err(|_| AppError::InvalidConfig(format!("{} is invalid", field)))
        })
        .transpose()?;
    if end.is_some_and(|end| start > end) {
        return Err(AppError::InvalidConfig(format!("{} is invalid", field)));
    }
    Ok((start, end))
}

fn safe_xml_comment(value: &str) -> Result<String, AppError> {
    validate_text(value, "rule comment")?;
    if value.contains("--") || value.ends_with('-') {
        return Err(AppError::InvalidConfig(
            "Rule comment contains unsupported XML comment content".to_string(),
        ));
    }
    Ok(value.to_string())
}
