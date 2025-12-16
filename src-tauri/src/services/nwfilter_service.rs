use virt::nwfilter::NWFilter;
use crate::models::nwfilter::{NwFilter, NwFilterConfig, NwFilterRule, RuleDirection, RuleAction};
use crate::services::libvirt::LibvirtService;
use crate::utils::error::{AppError, map_libvirt_error};

/// NwFilterService provides network filter management operations
pub struct NwFilterService;

impl NwFilterService {
    /// List all network filters
    pub fn list_filters(libvirt: &LibvirtService) -> Result<Vec<NwFilter>, AppError> {
        tracing::debug!("Listing all network filters");

        let conn = libvirt.get_connection();
        let filters = conn.list_all_nw_filters(0)
            .map_err(map_libvirt_error)?;

        let mut result = Vec::new();
        for filter in filters {
            match Self::filter_to_model(&filter) {
                Ok(model) => result.push(model),
                Err(e) => {
                    tracing::warn!("Failed to convert nwfilter to model: {}", e);
                    continue;
                }
            }
        }

        tracing::info!("Found {} network filters", result.len());
        Ok(result)
    }

    /// Get a network filter by name
    pub fn get_filter(libvirt: &LibvirtService, name: &str) -> Result<NwFilter, AppError> {
        tracing::debug!("Getting network filter: {}", name);

        let conn = libvirt.get_connection();
        let filter = NWFilter::lookup_by_name(conn, name)
            .map_err(map_libvirt_error)?;

        Self::filter_to_model(&filter)
    }

    /// Get the XML definition of a network filter
    pub fn get_filter_xml(libvirt: &LibvirtService, name: &str) -> Result<String, AppError> {
        tracing::debug!("Getting XML for network filter: {}", name);

        let conn = libvirt.get_connection();
        let filter = NWFilter::lookup_by_name(conn, name)
            .map_err(map_libvirt_error)?;

        filter.get_xml_desc(0).map_err(map_libvirt_error)
    }

    /// Create a new network filter from config
    pub fn create_filter(libvirt: &LibvirtService, config: NwFilterConfig) -> Result<NwFilter, AppError> {
        tracing::info!("Creating network filter: {}", config.name);

        let xml = Self::config_to_xml(&config)?;
        tracing::debug!("Generated nwfilter XML: {}", xml);

        let conn = libvirt.get_connection();
        let filter = NWFilter::define_xml(conn, &xml)
            .map_err(map_libvirt_error)?;

        Self::filter_to_model(&filter)
    }

    /// Create a network filter from raw XML
    pub fn create_filter_from_xml(libvirt: &LibvirtService, xml: &str) -> Result<NwFilter, AppError> {
        tracing::info!("Creating network filter from XML");

        let conn = libvirt.get_connection();
        let filter = NWFilter::define_xml(conn, xml)
            .map_err(map_libvirt_error)?;

        Self::filter_to_model(&filter)
    }

    /// Delete a network filter
    pub fn delete_filter(libvirt: &LibvirtService, name: &str) -> Result<(), AppError> {
        tracing::info!("Deleting network filter: {}", name);

        let conn = libvirt.get_connection();
        let filter = NWFilter::lookup_by_name(conn, name)
            .map_err(map_libvirt_error)?;

        filter.undefine().map_err(map_libvirt_error)?;

        tracing::info!("Network filter deleted: {}", name);
        Ok(())
    }

    /// Convert a libvirt NWFilter to our model
    fn filter_to_model(filter: &NWFilter) -> Result<NwFilter, AppError> {
        let uuid = filter.get_uuid_string()
            .map_err(map_libvirt_error)?;

        let name = filter.get_name()
            .map_err(map_libvirt_error)?;

        // Get XML to extract more info
        let xml = filter.get_xml_desc(0)
            .map_err(map_libvirt_error)?;

        let rule_count = Self::count_rules(&xml);
        let chain = Self::extract_chain(&xml);
        let priority = Self::extract_priority(&xml);

        Ok(NwFilter {
            uuid,
            name,
            rule_count,
            chain,
            priority,
        })
    }

    /// Count rules in XML
    fn count_rules(xml: &str) -> usize {
        xml.matches("<rule ").count()
    }

    /// Extract chain type from XML
    fn extract_chain(xml: &str) -> Option<String> {
        // Look for chain='...' in the filter element
        if let Some(start) = xml.find("chain='") {
            let start_pos = start + 7;
            if let Some(end) = xml[start_pos..].find('\'') {
                return Some(xml[start_pos..start_pos + end].to_string());
            }
        }
        None
    }

    /// Extract priority from XML
    fn extract_priority(xml: &str) -> Option<i32> {
        // Look for priority='...' in the filter element
        if let Some(start) = xml.find("priority='") {
            let start_pos = start + 10;
            if let Some(end) = xml[start_pos..].find('\'') {
                return xml[start_pos..start_pos + end].parse().ok();
            }
        }
        None
    }

    /// Convert config to XML
    fn config_to_xml(config: &NwFilterConfig) -> Result<String, AppError> {
        let mut xml = String::from("<?xml version='1.0' encoding='UTF-8'?>\n");

        // Start filter element
        xml.push_str(&format!("<filter name='{}'", config.name));

        if let Some(chain) = &config.chain {
            xml.push_str(&format!(" chain='{}'", chain));
        }

        if let Some(priority) = config.priority {
            xml.push_str(&format!(" priority='{}'", priority));
        }

        xml.push_str(">\n");

        // Add filter references
        for filter_ref in &config.filter_refs {
            xml.push_str(&format!("  <filterref filter='{}'/>\n", filter_ref));
        }

        // Add rules
        for rule in &config.rules {
            xml.push_str(&Self::rule_to_xml(rule)?);
        }

        xml.push_str("</filter>\n");
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
            xml.push_str(&format!(" priority='{}'", priority));
        }

        xml.push_str(">\n");

        // Determine protocol element
        let protocol = rule.protocol.as_deref().unwrap_or("all");

        if protocol == "all" {
            // Generic all rule - use comment only
            if let Some(comment) = &rule.comment {
                xml.push_str(&format!("    <!-- {} -->\n", comment));
            }
        } else {
            // Protocol-specific element
            xml.push_str(&format!("    <{}", protocol));

            if let Some(src_ip) = &rule.src_ip {
                xml.push_str(&format!(" srcipaddr='{}'", src_ip));
            }
            if let Some(dest_ip) = &rule.dest_ip {
                xml.push_str(&format!(" dstipaddr='{}'", dest_ip));
            }
            if let Some(src_mac) = &rule.src_mac {
                xml.push_str(&format!(" srcmacaddr='{}'", src_mac));
            }
            if let Some(dest_mac) = &rule.dest_mac {
                xml.push_str(&format!(" dstmacaddr='{}'", dest_mac));
            }
            if let Some(src_port) = &rule.src_port {
                xml.push_str(&format!(" srcportstart='{}'", src_port));
            }
            if let Some(dest_port) = &rule.dest_port {
                xml.push_str(&format!(" dstportstart='{}'", dest_port));
            }
            if let Some(comment) = &rule.comment {
                xml.push_str(&format!(" comment='{}'", comment));
            }

            xml.push_str("/>\n");
        }

        xml.push_str("  </rule>\n");
        Ok(xml)
    }
}
