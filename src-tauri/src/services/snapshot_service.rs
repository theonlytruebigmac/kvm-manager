use virt::domain::Domain;
use virt::domain_snapshot::DomainSnapshot;
use crate::models::snapshot::{Snapshot, SnapshotConfig, SnapshotState};
use crate::services::libvirt::LibvirtService;
use crate::utils::error::{AppError, map_libvirt_error};

/// SnapshotService provides VM snapshot management operations
pub struct SnapshotService;

impl SnapshotService {
    /// List all snapshots for a VM
    pub fn list_snapshots(libvirt: &LibvirtService, vm_id: &str) -> Result<Vec<Snapshot>, AppError> {
        tracing::debug!("Listing snapshots for VM: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let snapshot_count = DomainSnapshot::num(&domain, 0)
            .map_err(map_libvirt_error)?;

        if snapshot_count == 0 {
            return Ok(Vec::new());
        }

        let domain_snapshots = domain.list_all_snapshots(0)
            .map_err(map_libvirt_error)?;

        let mut snapshots = Vec::new();

        for snap in domain_snapshots {
            if let Ok(snapshot) = Self::snapshot_to_model(&snap, &domain) {
                snapshots.push(snapshot);
            }
        }

        // Sort by creation time, newest first
        snapshots.sort_by(|a, b| b.creation_time.cmp(&a.creation_time));

        Ok(snapshots)
    }

    /// Create a new snapshot
    pub fn create_snapshot(
        libvirt: &LibvirtService,
        vm_id: &str,
        config: SnapshotConfig,
    ) -> Result<String, AppError> {
        tracing::info!("Creating snapshot '{}' for VM: {}", config.name, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let description = config.description.unwrap_or_default();

        // Build snapshot XML
        let xml = if config.include_memory {
            format!(
                r#"<domainsnapshot>
  <name>{}</name>
  <description>{}</description>
  <memory snapshot='internal'/>
</domainsnapshot>"#,
                config.name, description
            )
        } else {
            format!(
                r#"<domainsnapshot>
  <name>{}</name>
  <description>{}</description>
  <memory snapshot='no'/>
</domainsnapshot>"#,
                config.name, description
            )
        };

        let snapshot = DomainSnapshot::create_xml(&domain, &xml, 0)
            .map_err(map_libvirt_error)?;

        let snap_name = snapshot.get_name()
            .map_err(map_libvirt_error)?;

        tracing::info!("Snapshot created successfully: {}", snap_name);
        Ok(snap_name)
    }

    /// Delete a snapshot
    pub fn delete_snapshot(
        libvirt: &LibvirtService,
        vm_id: &str,
        snapshot_name: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Deleting snapshot '{}' from VM: {}", snapshot_name, vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let snapshot = DomainSnapshot::lookup_by_name(&domain, snapshot_name, 0)
            .map_err(|_| AppError::LibvirtError(format!("Snapshot not found: {}", snapshot_name)))?;

        snapshot.delete(0)
            .map_err(map_libvirt_error)?;

        tracing::info!("Snapshot deleted successfully: {}", snapshot_name);
        Ok(())
    }

    /// Revert VM to a snapshot
    pub fn revert_snapshot(
        libvirt: &LibvirtService,
        vm_id: &str,
        snapshot_name: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Reverting VM {} to snapshot: {}", vm_id, snapshot_name);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let snapshot = DomainSnapshot::lookup_by_name(&domain, snapshot_name, 0)
            .map_err(|_| AppError::LibvirtError(format!("Snapshot not found: {}", snapshot_name)))?;

        snapshot.revert(0)
            .map_err(map_libvirt_error)?;

        tracing::info!("VM reverted to snapshot successfully: {}", snapshot_name);
        Ok(())
    }

    /// Convert libvirt snapshot to our model
    fn snapshot_to_model(snap: &DomainSnapshot, _domain: &Domain) -> Result<Snapshot, AppError> {
        let name = snap.get_name()
            .map_err(map_libvirt_error)?;

        let xml = snap.get_xml_desc(0)
            .map_err(map_libvirt_error)?;

        // Parse creation time from XML
        let creation_time = Self::parse_creation_time(&xml);

        // Parse description from XML
        let description = Self::parse_description(&xml);

        // Parse parent from XML
        let parent = Self::parse_parent(&xml);

        // Determine snapshot state
        let state = Self::parse_state(&xml);

        // Check if this is the current snapshot
        let is_current = snap.is_current(0).unwrap_or(false);        Ok(Snapshot {
            name,
            description,
            creation_time,
            state,
            parent,
            is_current,
        })
    }

    fn parse_creation_time(xml: &str) -> i64 {
        // Look for: <creationTime>1234567890</creationTime>
        if let Some(start) = xml.find("<creationTime>") {
            let time_section = &xml[start + 14..];
            if let Some(end) = time_section.find("</creationTime>") {
                if let Ok(time) = time_section[..end].parse::<i64>() {
                    return time;
                }
            }
        }
        0
    }

    fn parse_description(xml: &str) -> Option<String> {
        if let Some(start) = xml.find("<description>") {
            let desc_section = &xml[start + 13..];
            if let Some(end) = desc_section.find("</description>") {
                let desc = desc_section[..end].trim().to_string();
                if !desc.is_empty() {
                    return Some(desc);
                }
            }
        }
        None
    }

    fn parse_parent(xml: &str) -> Option<String> {
        if let Some(start) = xml.find("<parent>") {
            let parent_section = &xml[start + 8..];
            if let Some(name_start) = parent_section.find("<name>") {
                let name_section = &parent_section[name_start + 6..];
                if let Some(name_end) = name_section.find("</name>") {
                    return Some(name_section[..name_end].to_string());
                }
            }
        }
        None
    }

    fn parse_state(xml: &str) -> SnapshotState {
        if xml.contains("<state>running</state>") {
            SnapshotState::Running
        } else if xml.contains("<state>paused</state>") {
            SnapshotState::Paused
        } else if xml.contains("<state>shutoff</state>") || xml.contains("<state>shut off</state>") {
            SnapshotState::Shutoff
        } else if xml.contains("<memory snapshot='no'/>") || xml.contains("<memory snapshot='external'/>") {
            SnapshotState::DiskSnapshot
        } else {
            SnapshotState::DiskSnapshot
        }
    }
}
