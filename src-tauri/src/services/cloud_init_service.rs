use crate::models::cloud_init::CloudInitConfig;
use crate::utils::error::AppError;
use crate::utils::xml::{escaped_attribute, validate_identifier};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// CloudInitService handles cloud-init ISO generation
pub struct CloudInitService;

impl CloudInitService {
    const ISO_DIRECTORY: &'static str = "/var/lib/libvirt/images";

    fn validate_vm_name(vm_name: &str) -> Result<(), AppError> {
        validate_identifier(vm_name, "VM name")?;
        if !vm_name
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

    pub fn iso_path_for_vm(vm_name: &str) -> Result<PathBuf, AppError> {
        Self::validate_vm_name(vm_name)?;
        Ok(PathBuf::from(Self::ISO_DIRECTORY).join(format!("{vm_name}-cloud-init.iso")))
    }

    /// Builds the cloud-init CD-ROM fragment used in a domain definition. Paths stay attribute
    /// data and cannot add a device or attribute to the generated XML.
    pub fn attachment_definition(
        iso_path: &Path,
        target: &str,
        bus: &str,
    ) -> Result<String, AppError> {
        let iso_path = iso_path.to_str().ok_or_else(|| {
            AppError::InvalidConfig("Cloud-init ISO path is not valid UTF-8".to_string())
        })?;
        let target = escaped_attribute(target, "cloud-init target")?;
        let bus = escaped_attribute(bus, "cloud-init bus")?;
        Ok(format!(
            "    <disk type='file' device='cdrom'>\n      <driver name='qemu' type='raw'/>\n      <source file='{}'/>\n      <target dev='{target}' bus='{bus}'/>\n      <readonly/>\n    </disk>",
            escaped_attribute(iso_path, "cloud-init ISO path")?,
        ))
    }

    /// Generate a cloud-init ISO image
    ///
    /// This creates a NoCloud datasource ISO with user-data, meta-data, and optionally network-config
    /// The ISO can be attached to a VM as a CDROM device for automatic provisioning
    pub fn generate_iso(
        config: &CloudInitConfig,
        vm_name: &str,
        instance_id: &str,
    ) -> Result<PathBuf, AppError> {
        Self::validate_vm_name(vm_name)?;
        uuid::Uuid::parse_str(instance_id).map_err(|_| {
            AppError::InvalidConfig("Cloud-init instance ID must be a UUID".to_string())
        })?;
        tracing::info!(
            operation = "cloud-init-generate",
            "Generating cloud-init ISO"
        );

        // Create temporary directory for cloud-init files
        let temp_path = std::env::temp_dir().join(format!("cloud-init-{instance_id}"));

        if temp_path.exists() {
            fs::remove_dir_all(&temp_path).map_err(AppError::IoError)?;
        }

        fs::create_dir_all(&temp_path).map_err(AppError::IoError)?;

        // Generate user-data
        let user_data = config
            .generate_user_data()
            .map_err(|e| AppError::InvalidConfig(format!("Failed to generate user-data: {}", e)))?;

        let user_data_path = temp_path.join("user-data");
        let mut user_data_file = fs::File::create(&user_data_path).map_err(AppError::IoError)?;
        user_data_file
            .write_all(user_data.as_bytes())
            .map_err(AppError::IoError)?;

        tracing::debug!(
            operation = "cloud-init-user-data-written",
            "Generated cloud-init user-data"
        );

        // Generate meta-data
        let meta_data = config.generate_meta_data(instance_id);
        let meta_data_path = temp_path.join("meta-data");
        let mut meta_data_file = fs::File::create(&meta_data_path).map_err(AppError::IoError)?;
        meta_data_file
            .write_all(meta_data.as_bytes())
            .map_err(AppError::IoError)?;

        tracing::debug!(
            operation = "cloud-init-meta-data-written",
            "Generated cloud-init meta-data"
        );

        // Generate network-config (if specified)
        if let Some(network_config) = config.generate_network_config() {
            let network_config_path = temp_path.join("network-config");
            let mut network_config_file =
                fs::File::create(&network_config_path).map_err(AppError::IoError)?;
            network_config_file
                .write_all(network_config.as_bytes())
                .map_err(AppError::IoError)?;

            tracing::debug!(
                operation = "cloud-init-network-config-written",
                "Generated cloud-init network configuration"
            );
        }

        // Determine ISO output path
        let iso_path = Self::iso_path_for_vm(vm_name)?;

        // Generate ISO using genisoimage or mkisofs
        // Try genisoimage first, then mkisofs as fallback
        let iso_tool = if Self::command_exists("genisoimage") {
            "genisoimage"
        } else if Self::command_exists("mkisofs") {
            "mkisofs"
        } else {
            return Err(AppError::InvalidConfig(
                "Neither genisoimage nor mkisofs found. Install with: sudo apt install genisoimage"
                    .to_string(),
            ));
        };

        tracing::info!(
            operation = "cloud-init-iso-create",
            tool = iso_tool,
            "Creating cloud-init ISO"
        );

        let output = Command::new(iso_tool)
            .args([
                "-output",
                iso_path.to_str().unwrap(),
                "-volid",
                "cidata",
                "-joliet",
                "-rock",
                temp_path.to_str().unwrap(),
            ])
            .output()
            .map_err(AppError::IoError)?;

        if !output.status.success() {
            return Err(AppError::Other(
                "Cloud-init ISO creation failed".to_string(),
            ));
        }

        // Clean up temp directory
        if fs::remove_dir_all(&temp_path).is_err() {
            tracing::warn!("Cloud-init temporary cleanup did not complete");
        }

        tracing::info!(
            operation = "cloud-init-iso-created",
            "Cloud-init ISO created successfully"
        );
        Ok(iso_path)
    }

    /// Check if a command exists in PATH
    fn command_exists(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Delete a cloud-init ISO
    pub fn delete_iso(iso_path: &Path) -> Result<(), AppError> {
        if iso_path.parent() != Some(Path::new(Self::ISO_DIRECTORY))
            || !iso_path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with("-cloud-init.iso"))
        {
            return Err(AppError::InvalidConfig(
                "Cloud-init ISO path is outside the managed directory".to_string(),
            ));
        }
        if iso_path.exists() {
            fs::remove_file(iso_path).map_err(AppError::IoError)?;
            tracing::info!(
                operation = "cloud-init-iso-deleted",
                "Deleted cloud-init ISO"
            );
        }
        Ok(())
    }

    /// Get the path to a VM's cloud-init ISO
    pub fn get_iso_path(vm_name: &str) -> Result<PathBuf, AppError> {
        Self::iso_path_for_vm(vm_name)
    }

    /// Check if a VM has a cloud-init ISO
    pub fn has_iso(vm_name: &str) -> bool {
        Self::get_iso_path(vm_name)
            .map(|path| path.exists())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::CloudInitService;
    use crate::utils::xml::validate_document_root;
    use std::path::Path;

    #[test]
    fn cloud_init_attachment_escapes_the_iso_path() {
        let definition = CloudInitService::attachment_definition(
            Path::new("/images/cloud-init & data.iso"),
            "sdb",
            "sata",
        )
        .unwrap();

        validate_document_root(
            &format!("<domain><devices>{definition}</devices></domain>"),
            "domain",
        )
        .unwrap();
        assert!(definition.contains("cloud-init &amp; data.iso"));
    }

    #[test]
    fn cloud_init_filename_rejects_structure_like_vm_names() {
        assert!(CloudInitService::iso_path_for_vm("name</name><disk").is_err());
    }

    #[test]
    fn cloud_init_cleanup_cannot_target_an_arbitrary_file() {
        assert!(CloudInitService::delete_iso(Path::new("/tmp/not-a-cloud-init.iso")).is_err());
    }
}
