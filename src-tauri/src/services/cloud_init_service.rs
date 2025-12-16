use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::models::cloud_init::CloudInitConfig;
use crate::utils::error::AppError;

/// CloudInitService handles cloud-init ISO generation
pub struct CloudInitService;

impl CloudInitService {
    /// Generate a cloud-init ISO image
    ///
    /// This creates a NoCloud datasource ISO with user-data, meta-data, and optionally network-config
    /// The ISO can be attached to a VM as a CDROM device for automatic provisioning
    pub fn generate_iso(
        config: &CloudInitConfig,
        vm_name: &str,
        instance_id: &str,
    ) -> Result<PathBuf, AppError> {
        tracing::info!("Generating cloud-init ISO for VM: {}", vm_name);

        // Create temporary directory for cloud-init files
        let temp_dir = format!("/tmp/cloud-init-{}", instance_id);
        let temp_path = Path::new(&temp_dir);

        if temp_path.exists() {
            fs::remove_dir_all(temp_path)
                .map_err(|e| AppError::IoError(format!("Failed to clean temp directory: {}", e)))?;
        }

        fs::create_dir_all(temp_path)
            .map_err(|e| AppError::IoError(format!("Failed to create temp directory: {}", e)))?;

        // Generate user-data
        let user_data = config.generate_user_data()
            .map_err(|e| AppError::InvalidConfig(format!("Failed to generate user-data: {}", e)))?;

        let user_data_path = temp_path.join("user-data");
        let mut user_data_file = fs::File::create(&user_data_path)
            .map_err(|e| AppError::IoError(format!("Failed to create user-data file: {}", e)))?;
        user_data_file.write_all(user_data.as_bytes())
            .map_err(|e| AppError::IoError(format!("Failed to write user-data: {}", e)))?;

        tracing::debug!("Generated user-data:\n{}", user_data);

        // Generate meta-data
        let meta_data = config.generate_meta_data(instance_id);
        let meta_data_path = temp_path.join("meta-data");
        let mut meta_data_file = fs::File::create(&meta_data_path)
            .map_err(|e| AppError::IoError(format!("Failed to create meta-data file: {}", e)))?;
        meta_data_file.write_all(meta_data.as_bytes())
            .map_err(|e| AppError::IoError(format!("Failed to write meta-data: {}", e)))?;

        tracing::debug!("Generated meta-data:\n{}", meta_data);

        // Generate network-config (if specified)
        if let Some(network_config) = config.generate_network_config() {
            let network_config_path = temp_path.join("network-config");
            let mut network_config_file = fs::File::create(&network_config_path)
                .map_err(|e| AppError::IoError(format!("Failed to create network-config file: {}", e)))?;
            network_config_file.write_all(network_config.as_bytes())
                .map_err(|e| AppError::IoError(format!("Failed to write network-config: {}", e)))?;

            tracing::debug!("Generated network-config:\n{}", network_config);
        }

        // Determine ISO output path
        let iso_name = format!("{}-cloud-init.iso", vm_name);
        let iso_path = PathBuf::from(format!("/var/lib/libvirt/images/{}", iso_name));

        // Generate ISO using genisoimage or mkisofs
        // Try genisoimage first, then mkisofs as fallback
        let iso_tool = if Self::command_exists("genisoimage") {
            "genisoimage"
        } else if Self::command_exists("mkisofs") {
            "mkisofs"
        } else {
            return Err(AppError::InvalidConfig(
                "Neither genisoimage nor mkisofs found. Install with: sudo apt install genisoimage".to_string()
            ));
        };

        tracing::info!("Using {} to create ISO: {}", iso_tool, iso_path.display());

        let output = Command::new(iso_tool)
            .args(&[
                "-output", iso_path.to_str().unwrap(),
                "-volid", "cidata",
                "-joliet",
                "-rock",
                temp_path.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| AppError::IoError(format!("Failed to execute {}: {}", iso_tool, e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::IoError(format!(
                "{} failed: {}",
                iso_tool,
                stderr
            )));
        }

        // Clean up temp directory
        if let Err(e) = fs::remove_dir_all(temp_path) {
            tracing::warn!("Failed to clean up temp directory: {}", e);
        }

        tracing::info!("Cloud-init ISO created successfully: {}", iso_path.display());
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
        if iso_path.exists() {
            fs::remove_file(iso_path)
                .map_err(|e| AppError::IoError(format!("Failed to delete cloud-init ISO: {}", e)))?;
            tracing::info!("Deleted cloud-init ISO: {}", iso_path.display());
        }
        Ok(())
    }

    /// Get the path to a VM's cloud-init ISO
    pub fn get_iso_path(vm_name: &str) -> PathBuf {
        PathBuf::from(format!("/var/lib/libvirt/images/{}-cloud-init.iso", vm_name))
    }

    /// Check if a VM has a cloud-init ISO
    pub fn has_iso(vm_name: &str) -> bool {
        Self::get_iso_path(vm_name).exists()
    }
}
