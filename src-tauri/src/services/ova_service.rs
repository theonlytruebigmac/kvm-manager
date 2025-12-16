use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};
use crate::utils::error::AppError;

/// OVA/OVF Import Service
/// Handles importing VMs from OVA (Open Virtual Appliance) and OVF (Open Virtualization Format) files
pub struct OvaService;

/// Parsed OVF metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OvfMetadata {
    pub name: String,
    pub description: Option<String>,
    pub os_type: Option<String>,
    pub cpu_count: u32,
    pub memory_mb: u64,
    pub disks: Vec<OvfDisk>,
    pub networks: Vec<OvfNetwork>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OvfDisk {
    pub id: String,
    pub file_name: String,
    pub capacity_bytes: u64,
    pub format: String, // vmdk, vhd, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OvfNetwork {
    pub name: String,
    pub description: Option<String>,
}

/// Import configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OvaImportConfig {
    /// Path to OVA or OVF file
    pub source_path: String,
    /// Target storage pool path
    pub target_pool_path: String,
    /// Optional: Override VM name
    pub vm_name: Option<String>,
    /// Optional: Override memory (MB)
    pub memory_mb: Option<u64>,
    /// Optional: Override CPU count
    pub cpu_count: Option<u32>,
    /// Convert disks to qcow2 (recommended)
    #[serde(default = "default_true")]
    pub convert_to_qcow2: bool,
}

fn default_true() -> bool {
    true
}

impl OvaService {
    /// Extract and parse OVA/OVF file to get metadata
    pub fn get_ova_metadata(source_path: &str) -> Result<OvfMetadata, AppError> {
        let path = Path::new(source_path);

        if !path.exists() {
            return Err(AppError::NotFound(format!("File not found: {}", source_path)));
        }

        // Determine if it's OVA (tarball) or OVF (XML file)
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let ovf_content = match extension.as_str() {
            "ova" => Self::extract_ovf_from_ova(path)?,
            "ovf" => fs::read_to_string(path)
                .map_err(|e| AppError::Other(format!("Failed to read OVF file: {}", e)))?,
            _ => return Err(AppError::InvalidConfig(
                "Unsupported file format. Expected .ova or .ovf".to_string()
            )),
        };

        Self::parse_ovf(&ovf_content)
    }

    /// Extract OVF XML from OVA tarball
    fn extract_ovf_from_ova(ova_path: &Path) -> Result<String, AppError> {
        let file = File::open(ova_path)
            .map_err(|e| AppError::Other(format!("Failed to open OVA file: {}", e)))?;

        let reader = BufReader::new(file);
        let mut archive = tar::Archive::new(reader);

        for entry_result in archive.entries()
            .map_err(|e| AppError::Other(format!("Failed to read OVA archive: {}", e)))?
        {
            let mut entry = entry_result
                .map_err(|e| AppError::Other(format!("Failed to read archive entry: {}", e)))?;

            let path = entry.path()
                .map_err(|e| AppError::Other(format!("Failed to get entry path: {}", e)))?;

            if path.extension().and_then(|e| e.to_str()) == Some("ovf") {
                let mut content = String::new();
                entry.read_to_string(&mut content)
                    .map_err(|e| AppError::Other(format!("Failed to read OVF from archive: {}", e)))?;
                return Ok(content);
            }
        }

        Err(AppError::NotFound("No .ovf file found in OVA archive".to_string()))
    }

    /// Parse OVF XML to extract metadata
    fn parse_ovf(ovf_content: &str) -> Result<OvfMetadata, AppError> {
        // Simple XML parsing - in production you'd want a proper XML parser
        let name = Self::extract_xml_value(ovf_content, "VirtualSystemIdentifier")
            .or_else(|| Self::extract_xml_value(ovf_content, "Name"))
            .unwrap_or_else(|| "Imported VM".to_string());

        let description = Self::extract_xml_value(ovf_content, "Annotation")
            .or_else(|| Self::extract_xml_value(ovf_content, "Description"));

        let os_type = Self::extract_ovf_os_type(ovf_content);

        // Parse virtual hardware
        let cpu_count = Self::extract_resource_value(ovf_content, "3") // ResourceType 3 = CPU
            .unwrap_or(1);

        let memory_mb = Self::extract_resource_value(ovf_content, "4") // ResourceType 4 = Memory
            .map(|v| v as u64)
            .unwrap_or(1024);

        // Parse disks
        let disks = Self::parse_ovf_disks(ovf_content);

        // Parse networks
        let networks = Self::parse_ovf_networks(ovf_content);

        Ok(OvfMetadata {
            name,
            description,
            os_type,
            cpu_count,
            memory_mb,
            disks,
            networks,
        })
    }

    /// Extract OS type from OVF
    fn extract_ovf_os_type(ovf: &str) -> Option<String> {
        // Look for OperatingSystemSection
        if let Some(start) = ovf.find("OperatingSystemSection") {
            let section = &ovf[start..];
            if let Some(end) = section.find("</OperatingSystemSection>") {
                let section_content = &section[..end];
                // Look for Description or osType
                if let Some(desc) = Self::extract_xml_value(section_content, "Description") {
                    return Some(desc);
                }
            }
        }
        None
    }

    /// Extract a simple XML element value
    fn extract_xml_value(xml: &str, tag: &str) -> Option<String> {
        // Try both with and without namespace prefix
        for pattern in [format!("<{}>", tag), format!("<ovf:{}>", tag), format!("<rasd:{}>", tag)] {
            if let Some(start) = xml.find(&pattern) {
                let start_pos = start + pattern.len();
                let remaining = &xml[start_pos..];

                // Find the closing tag
                for close_pattern in [format!("</{}>", tag), format!("</ovf:{}>", tag), format!("</rasd:{}>", tag)] {
                    if let Some(end) = remaining.find(&close_pattern) {
                        let value = remaining[..end].trim().to_string();
                        if !value.is_empty() {
                            return Some(value);
                        }
                    }
                }
            }
        }
        None
    }

    /// Extract resource value by ResourceType
    fn extract_resource_value(ovf: &str, resource_type: &str) -> Option<u32> {
        // Look for Item elements with the specific ResourceType
        let type_pattern = format!(">{}< ", resource_type);
        let type_pattern2 = format!(">{}</", resource_type);

        // Find all Item sections
        let mut pos = 0;
        while let Some(start) = ovf[pos..].find("<Item") {
            let item_start = pos + start;
            if let Some(end) = ovf[item_start..].find("</Item>") {
                let item = &ovf[item_start..item_start + end + 7];

                // Check if this item has the right ResourceType
                if item.contains(&type_pattern) || item.contains(&type_pattern2) {
                    // Extract VirtualQuantity
                    if let Some(qty) = Self::extract_xml_value(item, "VirtualQuantity") {
                        if let Ok(value) = qty.parse::<u32>() {
                            return Some(value);
                        }
                    }
                }

                pos = item_start + end + 7;
            } else {
                break;
            }
        }
        None
    }

    /// Parse disk references from OVF
    fn parse_ovf_disks(ovf: &str) -> Vec<OvfDisk> {
        let mut disks = Vec::new();

        // Look for Disk elements in DiskSection
        let mut pos = 0;
        while let Some(start) = ovf[pos..].find("<Disk ") {
            let disk_start = pos + start;
            if let Some(end) = ovf[disk_start..].find("/>") {
                let disk_elem = &ovf[disk_start..disk_start + end + 2];

                // Extract attributes
                let id = Self::extract_attribute(disk_elem, "diskId")
                    .unwrap_or_else(|| format!("disk-{}", disks.len()));

                let file_ref = Self::extract_attribute(disk_elem, "fileRef")
                    .unwrap_or_default();

                let capacity = Self::extract_attribute(disk_elem, "capacity")
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);

                let format = Self::extract_attribute(disk_elem, "format")
                    .unwrap_or_else(|| "vmdk".to_string());

                // Get actual filename from File element
                let file_name = Self::find_file_by_id(ovf, &file_ref)
                    .unwrap_or(file_ref);

                disks.push(OvfDisk {
                    id,
                    file_name,
                    capacity_bytes: capacity,
                    format,
                });

                pos = disk_start + end + 2;
            } else {
                break;
            }
        }

        disks
    }

    /// Find file reference by ID
    fn find_file_by_id(ovf: &str, file_id: &str) -> Option<String> {
        // Look for File element with matching id
        let pattern = format!("id=\"{}\"", file_id);
        if let Some(start) = ovf.find(&pattern) {
            // Go back to find the File element start
            let before = &ovf[..start];
            if let Some(file_start) = before.rfind("<File ") {
                let file_elem = &ovf[file_start..];
                if let Some(end) = file_elem.find("/>") {
                    let elem = &file_elem[..end];
                    return Self::extract_attribute(elem, "href");
                }
            }
        }
        None
    }

    /// Parse network references from OVF
    fn parse_ovf_networks(ovf: &str) -> Vec<OvfNetwork> {
        let mut networks = Vec::new();

        // Look for Network elements
        let mut pos = 0;
        while let Some(start) = ovf[pos..].find("<Network ") {
            let net_start = pos + start;
            if let Some(end) = ovf[net_start..].find("</Network>").or_else(|| ovf[net_start..].find("/>")) {
                let net_elem = &ovf[net_start..net_start + end];

                let name = Self::extract_attribute(net_elem, "name")
                    .unwrap_or_else(|| format!("network-{}", networks.len()));

                let description = Self::extract_xml_value(net_elem, "Description");

                networks.push(OvfNetwork { name, description });

                pos = net_start + end;
            } else {
                break;
            }
        }

        networks
    }

    /// Extract XML attribute value
    fn extract_attribute(elem: &str, attr: &str) -> Option<String> {
        // Try with and without namespace
        for pattern in [format!("{}=\"", attr), format!("ovf:{}=\"", attr)] {
            if let Some(start) = elem.find(&pattern) {
                let start_pos = start + pattern.len();
                if let Some(end) = elem[start_pos..].find('"') {
                    return Some(elem[start_pos..start_pos + end].to_string());
                }
            }
        }
        None
    }

    /// Import OVA/OVF and convert to libvirt VM
    pub fn import_ova(config: OvaImportConfig) -> Result<PathBuf, AppError> {
        let source_path = Path::new(&config.source_path);

        if !source_path.exists() {
            return Err(AppError::NotFound(format!("Source file not found: {}", config.source_path)));
        }

        let target_path = Path::new(&config.target_pool_path);
        if !target_path.exists() {
            return Err(AppError::NotFound(format!("Target pool path not found: {}", config.target_pool_path)));
        }

        // Get metadata
        let metadata = Self::get_ova_metadata(&config.source_path)?;
        let vm_name = config.vm_name.unwrap_or(metadata.name.clone());

        // Determine extraction directory
        let extension = source_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Extract or locate disk files
        let disk_files = if extension == "ova" {
            Self::extract_ova_disks(source_path, target_path, &metadata.disks)?
        } else {
            // OVF file - disks should be in same directory
            let ovf_dir = source_path.parent()
                .ok_or_else(|| AppError::Other("Cannot determine OVF directory".to_string()))?;
            Self::locate_ovf_disks(ovf_dir, target_path, &metadata.disks)?
        };

        // Convert disks to qcow2 if requested
        let converted_disks = if config.convert_to_qcow2 {
            Self::convert_disks_to_qcow2(&disk_files, target_path, &vm_name)?
        } else {
            disk_files
        };

        // Return the path to the first disk (main disk)
        converted_disks.into_iter().next()
            .ok_or_else(|| AppError::NotFound("No disks found in OVA/OVF".to_string()))
    }

    /// Extract disk files from OVA archive
    fn extract_ova_disks(ova_path: &Path, target_dir: &Path, disks: &[OvfDisk]) -> Result<Vec<PathBuf>, AppError> {
        let file = File::open(ova_path)
            .map_err(|e| AppError::Other(format!("Failed to open OVA: {}", e)))?;

        let reader = BufReader::new(file);
        let mut archive = tar::Archive::new(reader);
        let mut extracted = Vec::new();

        // Get list of disk filenames we're looking for
        let disk_names: Vec<&str> = disks.iter()
            .map(|d| d.file_name.as_str())
            .collect();

        for entry_result in archive.entries()
            .map_err(|e| AppError::Other(format!("Failed to read archive: {}", e)))?
        {
            let mut entry = entry_result
                .map_err(|e| AppError::Other(format!("Failed to read entry: {}", e)))?;

            let entry_path = entry.path()
                .map_err(|e| AppError::Other(format!("Failed to get path: {}", e)))?;

            let file_name_owned = entry_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            let file_name = file_name_owned.as_str();

            // Check if this is a disk file
            if disk_names.iter().any(|d| d.ends_with(file_name) || file_name.ends_with(d)) {
                let dest_path = target_dir.join(file_name);

                tracing::info!("Extracting disk: {} -> {:?}", file_name, dest_path);

                // Drop the path borrow before unpacking
                drop(entry_path);

                entry.unpack(&dest_path)
                    .map_err(|e| AppError::Other(format!("Failed to extract {}: {}", file_name, e)))?;

                extracted.push(dest_path);
            }
        }

        if extracted.is_empty() {
            // Fall back to extracting any VMDK/VHD files
            let file = File::open(ova_path)
                .map_err(|e| AppError::Other(format!("Failed to reopen OVA: {}", e)))?;
            let reader = BufReader::new(file);
            let mut archive = tar::Archive::new(reader);

            for entry_result in archive.entries()
                .map_err(|e| AppError::Other(format!("Failed to read archive: {}", e)))?
            {
                let mut entry = entry_result
                    .map_err(|e| AppError::Other(format!("Failed to read entry: {}", e)))?;

                let entry_path = entry.path()
                    .map_err(|e| AppError::Other(format!("Failed to get path: {}", e)))?;

                let file_name_owned = entry_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                let file_name = file_name_owned.as_str();

                let ext = entry_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_string();

                // Drop the path borrow before unpacking
                drop(entry_path);

                if ["vmdk", "vhd", "vhdx", "vdi", "raw", "qcow2"].contains(&ext.as_str()) {
                    let dest_path = target_dir.join(file_name);

                    tracing::info!("Extracting disk file: {} -> {:?}", file_name, dest_path);

                    entry.unpack(&dest_path)
                        .map_err(|e| AppError::Other(format!("Failed to extract {}: {}", file_name, e)))?;

                    extracted.push(dest_path);
                }
            }
        }

        Ok(extracted)
    }

    /// Locate disk files for OVF (they should be in the same directory)
    fn locate_ovf_disks(ovf_dir: &Path, target_dir: &Path, disks: &[OvfDisk]) -> Result<Vec<PathBuf>, AppError> {
        let mut found = Vec::new();

        for disk in disks {
            let source_path = ovf_dir.join(&disk.file_name);

            if source_path.exists() {
                let dest_path = target_dir.join(&disk.file_name);

                tracing::info!("Copying disk: {:?} -> {:?}", source_path, dest_path);

                fs::copy(&source_path, &dest_path)
                    .map_err(|e| AppError::Other(format!("Failed to copy disk: {}", e)))?;

                found.push(dest_path);
            }
        }

        Ok(found)
    }

    /// Convert disk files to qcow2 using qemu-img
    fn convert_disks_to_qcow2(disks: &[PathBuf], target_dir: &Path, vm_name: &str) -> Result<Vec<PathBuf>, AppError> {
        let mut converted = Vec::new();

        for (i, disk) in disks.iter().enumerate() {
            let suffix = if i == 0 { String::new() } else { format!("-{}", i) };
            let qcow2_name = format!("{}{}.qcow2", vm_name, suffix);
            let dest_path = target_dir.join(&qcow2_name);

            tracing::info!("Converting disk: {:?} -> {:?}", disk, dest_path);

            // Use qemu-img to convert
            let output = Command::new("qemu-img")
                .args([
                    "convert",
                    "-f", "vmdk",  // Source format (auto-detected if wrong)
                    "-O", "qcow2",
                    "-o", "compat=1.1",
                    disk.to_str().unwrap_or(""),
                    dest_path.to_str().unwrap_or(""),
                ])
                .output()
                .map_err(|e| AppError::Other(format!("Failed to run qemu-img: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(AppError::Other(format!("qemu-img conversion failed: {}", stderr)));
            }

            // Remove the original VMDK
            let _ = fs::remove_file(disk);

            converted.push(dest_path);
        }

        Ok(converted)
    }
}
