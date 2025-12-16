use virt::storage_pool::StoragePool;
use virt::storage_vol::StorageVol;
use virt::sys;
use crate::models::storage::{StoragePool as StoragePoolModel, Volume, VolumeConfig, PoolState, PoolType};
use crate::services::libvirt::LibvirtService;
use crate::utils::error::{AppError, map_libvirt_error};

/// StorageService provides storage pool and volume management operations
pub struct StorageService;

impl StorageService {
    /// List all storage pools (active and inactive)
    pub fn list_storage_pools(libvirt: &LibvirtService) -> Result<Vec<StoragePoolModel>, AppError> {
        tracing::debug!("Listing all storage pools");

        let conn = libvirt.get_connection();
        let mut pools = Vec::new();

        // Get all storage pools (both active and inactive)
        let flags = sys::VIR_CONNECT_LIST_STORAGE_POOLS_ACTIVE | sys::VIR_CONNECT_LIST_STORAGE_POOLS_INACTIVE;
        let storage_pools = conn.list_all_storage_pools(flags)
            .map_err(map_libvirt_error)?;

        for pool in storage_pools {
            match Self::pool_to_model(&pool) {
                Ok(pool_model) => pools.push(pool_model),
                Err(e) => {
                    tracing::warn!("Failed to convert storage pool to model: {}", e);
                    continue;
                }
            }
        }

        tracing::info!("Found {} storage pools", pools.len());
        Ok(pools)
    }

    /// Convert a libvirt StoragePool to our StoragePool model
    fn pool_to_model(pool: &StoragePool) -> Result<StoragePoolModel, AppError> {
        let uuid = pool.get_uuid_string()
            .map_err(map_libvirt_error)?;

        let name = pool.get_name()
            .map_err(map_libvirt_error)?;

        let is_active = pool.is_active()
            .map_err(map_libvirt_error)?;

        let state = if is_active {
            PoolState::Active
        } else {
            PoolState::Inactive
        };

        // Get pool info for capacity and allocation
        let info = pool.get_info()
            .map_err(map_libvirt_error)?;

        let capacity_bytes = info.capacity;
        let allocation_bytes = info.allocation;
        let available_bytes = info.available;

        // Get pool XML to extract type and path
        let xml = pool.get_xml_desc(0)
            .map_err(map_libvirt_error)?;

        let pool_type = Self::extract_pool_type(&xml)?;
        let path = Self::extract_pool_path(&xml)?;

        let autostart = pool.get_autostart()
            .map_err(map_libvirt_error)?;

        Ok(StoragePoolModel {
            id: uuid,
            name,
            state,
            pool_type,
            capacity_bytes,
            allocation_bytes,
            available_bytes,
            path,
            autostart,
        })
    }

    /// Extract pool type from XML
    fn extract_pool_type(xml: &str) -> Result<PoolType, AppError> {
        // Simple XML parsing to get the pool type attribute
        if let Some(start) = xml.find("<pool type='") {
            let start_pos = start + 12;
            if let Some(end) = xml[start_pos..].find('\'') {
                let type_str = &xml[start_pos..start_pos + end];
                return match type_str {
                    "dir" => Ok(PoolType::Dir),
                    "fs" => Ok(PoolType::Fs),
                    "netfs" => Ok(PoolType::Netfs),
                    "logical" => Ok(PoolType::Logical),
                    "disk" => Ok(PoolType::Disk),
                    "iscsi" => Ok(PoolType::Iscsi),
                    "scsi" => Ok(PoolType::Scsi),
                    "mpath" => Ok(PoolType::Mpath),
                    "rbd" => Ok(PoolType::Rbd),
                    "sheepdog" => Ok(PoolType::Sheepdog),
                    "gluster" => Ok(PoolType::Gluster),
                    "zfs" => Ok(PoolType::Zfs),
                    _ => Ok(PoolType::Dir), // Default to dir
                };
            }
        }
        Ok(PoolType::Dir) // Default to dir if not found
    }

    /// Extract pool path from XML
    fn extract_pool_path(xml: &str) -> Result<String, AppError> {
        // Simple XML parsing to get the path
        if let Some(start) = xml.find("<path>") {
            let start_pos = start + 6;
            if let Some(end) = xml[start_pos..].find("</path>") {
                return Ok(xml[start_pos..start_pos + end].to_string());
            }
        }
        Ok(String::new())
    }

    /// List all volumes in a storage pool
    pub fn list_volumes(libvirt: &LibvirtService, pool_id: &str) -> Result<Vec<Volume>, AppError> {
        tracing::debug!("Listing volumes in pool: {}", pool_id);

        let conn = libvirt.get_connection();
        let pool = StoragePool::lookup_by_uuid_string(conn, pool_id)
            .map_err(|_| AppError::LibvirtError(format!("Storage pool not found: {}", pool_id)))?;

        let pool_name = pool.get_name()
            .map_err(map_libvirt_error)?;

        let mut volumes = Vec::new();

        // List all volumes in the pool
        let volume_names = pool.list_volumes()
            .map_err(map_libvirt_error)?;

        for vol_name in volume_names {
            let volume = StorageVol::lookup_by_name(&pool, &vol_name)
                .map_err(map_libvirt_error)?;

            match Self::volume_to_model(&volume, &pool_name) {
                Ok(vol_model) => volumes.push(vol_model),
                Err(e) => {
                    tracing::warn!("Failed to convert volume to model: {}", e);
                    continue;
                }
            }
        }

        tracing::info!("Found {} volumes in pool {}", volumes.len(), pool_name);
        Ok(volumes)
    }

    /// Convert a libvirt StorageVol to our Volume model
    fn volume_to_model(volume: &StorageVol, pool_name: &str) -> Result<Volume, AppError> {
        let name = volume.get_name()
            .map_err(map_libvirt_error)?;

        let path = volume.get_path()
            .map_err(map_libvirt_error)?;

        let info = volume.get_info()
            .map_err(map_libvirt_error)?;

        let capacity_bytes = info.capacity;
        let allocation_bytes = info.allocation;

        // Get volume XML to extract format
        let xml = volume.get_xml_desc(0)
            .map_err(map_libvirt_error)?;

        let format = Self::extract_volume_format(&xml)?;

        Ok(Volume {
            name,
            path,
            pool_name: pool_name.to_string(),
            capacity_bytes,
            allocation_bytes,
            format,
        })
    }

    /// Extract volume format from XML
    fn extract_volume_format(xml: &str) -> Result<String, AppError> {
        // Simple XML parsing to get the format type
        if let Some(start) = xml.find("<format type='") {
            let start_pos = start + 14;
            if let Some(end) = xml[start_pos..].find('\'') {
                return Ok(xml[start_pos..start_pos + end].to_string());
            }
        }
        Ok("raw".to_string()) // Default to raw if not found
    }

    /// Check if a volume is encrypted and get encryption info
    pub fn get_volume_encryption_info(
        libvirt: &LibvirtService,
        pool_id: &str,
        volume_name: &str,
    ) -> Result<crate::models::storage::VolumeEncryptionInfo, AppError> {
        let conn = libvirt.get_connection();
        let pool = StoragePool::lookup_by_uuid_string(conn, pool_id)
            .map_err(|_| AppError::LibvirtError(format!("Storage pool not found: {}", pool_id)))?;

        let volume = StorageVol::lookup_by_name(&pool, volume_name)
            .map_err(|_| AppError::LibvirtError(format!("Volume not found: {}", volume_name)))?;

        let xml = volume.get_xml_desc(0)
            .map_err(map_libvirt_error)?;

        // Check for encryption element
        let encrypted = xml.contains("<encryption");

        let format = if encrypted {
            if xml.contains("format='luks'") {
                Some("luks".to_string())
            } else if xml.contains("format='qcow'") {
                Some("qcow".to_string())
            } else {
                Some("unknown".to_string())
            }
        } else {
            None
        };

        // Extract secret UUID if present
        let secret_uuid = if let Some(start) = xml.find("<secret type='passphrase' uuid='") {
            let start_pos = start + 32;
            if let Some(end) = xml[start_pos..].find('\'') {
                Some(xml[start_pos..start_pos + end].to_string())
            } else {
                None
            }
        } else {
            None
        };

        Ok(crate::models::storage::VolumeEncryptionInfo {
            encrypted,
            format,
            secret_uuid,
        })
    }

    /// Create a new volume in a storage pool
    pub fn create_volume(
        libvirt: &LibvirtService,
        pool_id: &str,
        config: VolumeConfig,
    ) -> Result<String, AppError> {
        tracing::info!("Creating volume {} in pool {} (encrypted: {})", config.name, pool_id, config.encrypted);

        let conn = libvirt.get_connection();
        let pool = StoragePool::lookup_by_uuid_string(conn, pool_id)
            .map_err(|_| AppError::LibvirtError(format!("Storage pool not found: {}", pool_id)))?;

        // Check if pool is active
        let is_active = pool.is_active()
            .map_err(map_libvirt_error)?;

        if !is_active {
            return Err(AppError::LibvirtError("Storage pool is not active".to_string()));
        }

        // Convert GB to bytes
        let capacity_bytes = config.capacity_gb * 1024 * 1024 * 1024;

        // Handle encryption
        let (encryption_xml, secret_uuid) = if config.encrypted {
            let passphrase = config.passphrase.as_ref()
                .ok_or_else(|| AppError::InvalidConfig("Passphrase required for encrypted volume".to_string()))?;

            if passphrase.len() < 8 {
                return Err(AppError::InvalidConfig("Passphrase must be at least 8 characters".to_string()));
            }

            // Create a libvirt secret for the passphrase
            let secret_uuid = Self::create_volume_secret(libvirt, &config.name, passphrase)?;

            let enc_xml = format!(
                r#"
    <encryption format='luks'>
      <secret type='passphrase' uuid='{}'/>
    </encryption>"#,
                secret_uuid
            );

            (enc_xml, Some(secret_uuid))
        } else {
            (String::new(), None)
        };

        // Create volume XML
        let volume_xml = format!(
            r#"<volume>
  <name>{}</name>
  <capacity unit='bytes'>{}</capacity>
  <target>
    <format type='{}'/>{}</target>
</volume>"#,
            config.name, capacity_bytes, config.format, encryption_xml
        );

        tracing::debug!("Volume XML: {}", volume_xml);

        // Create the volume
        let volume = match StorageVol::create_xml(&pool, &volume_xml, 0) {
            Ok(v) => v,
            Err(e) => {
                // Clean up secret if volume creation fails
                if let Some(ref uuid) = secret_uuid {
                    let _ = Self::delete_secret(libvirt, uuid);
                }
                return Err(AppError::LibvirtError(format!("Failed to create volume: {}", e)));
            }
        };

        let path = volume.get_path()
            .map_err(map_libvirt_error)?;

        tracing::info!("Volume created successfully: {} at {} (encrypted: {})", config.name, path, config.encrypted);
        Ok(path)
    }

    /// Create a libvirt secret for volume encryption
    fn create_volume_secret(
        libvirt: &LibvirtService,
        volume_name: &str,
        passphrase: &str,
    ) -> Result<String, AppError> {
        use uuid::Uuid;

        let secret_uuid = Uuid::new_v4().to_string();
        let conn = libvirt.get_connection();

        // Create secret XML
        let secret_xml = format!(
            r#"<secret ephemeral='no' private='yes'>
  <uuid>{}</uuid>
  <description>LUKS passphrase for volume {}</description>
  <usage type='volume'>
    <volume>{}</volume>
  </usage>
</secret>"#,
            secret_uuid, volume_name, volume_name
        );

        tracing::debug!("Creating secret for volume: {}", volume_name);

        // Define the secret
        let secret = virt::secret::Secret::define_xml(conn, &secret_xml, 0)
            .map_err(|e| AppError::LibvirtError(format!("Failed to create secret: {}", e)))?;

        // Set the secret value (passphrase)
        secret.set_value(passphrase.as_bytes(), 0)
            .map_err(|e| AppError::LibvirtError(format!("Failed to set secret value: {}", e)))?;

        tracing::info!("Created encryption secret: {}", secret_uuid);
        Ok(secret_uuid)
    }

    /// Delete a libvirt secret
    fn delete_secret(libvirt: &LibvirtService, uuid: &str) -> Result<(), AppError> {
        let conn = libvirt.get_connection();

        if let Ok(secret) = virt::secret::Secret::lookup_by_uuid_string(conn, uuid) {
            secret.undefine()
                .map_err(|e| AppError::LibvirtError(format!("Failed to delete secret: {}", e)))?;
            tracing::info!("Deleted secret: {}", uuid);
        }

        Ok(())
    }

    /// Delete a volume from a storage pool
    pub fn delete_volume(
        libvirt: &LibvirtService,
        pool_id: &str,
        volume_name: &str,
    ) -> Result<(), AppError> {
        tracing::info!("Deleting volume {} from pool {}", volume_name, pool_id);

        let conn = libvirt.get_connection();
        let pool = StoragePool::lookup_by_uuid_string(conn, pool_id)
            .map_err(|_| AppError::LibvirtError(format!("Storage pool not found: {}", pool_id)))?;

        // Lookup the volume
        let volume = StorageVol::lookup_by_name(&pool, volume_name)
            .map_err(|_| AppError::LibvirtError(format!("Volume not found: {}", volume_name)))?;

        // Delete the volume
        volume.delete(0)
            .map_err(|e| AppError::LibvirtError(format!("Failed to delete volume: {}", e)))?;

        tracing::info!("Volume deleted successfully: {}", volume_name);
        Ok(())
    }

    /// Create a new storage pool
    pub fn create_storage_pool(
        libvirt: &LibvirtService,
        config: crate::models::storage::StoragePoolConfig,
    ) -> Result<String, AppError> {
        tracing::info!("Creating storage pool: {} (type: {})", config.name, config.pool_type);

        let conn = libvirt.get_connection();

        // Build XML based on pool type
        let pool_xml = match config.pool_type.as_str() {
            "dir" => Self::build_dir_pool_xml(&config)?,
            "logical" => Self::build_logical_pool_xml(&config)?,
            "netfs" => Self::build_netfs_pool_xml(&config)?,
            "iscsi" => Self::build_iscsi_pool_xml(&config)?,
            "gluster" => Self::build_gluster_pool_xml(&config)?,
            "rbd" => Self::build_rbd_pool_xml(&config)?,
            _ => return Err(AppError::InvalidConfig(format!("Unsupported pool type: {}", config.pool_type))),
        };

        tracing::debug!("Storage pool XML:\n{}", pool_xml);

        // Define the pool
        let pool = StoragePool::define_xml(conn, &pool_xml, 0)
            .map_err(|e| AppError::LibvirtError(format!("Failed to define storage pool: {}", e)))?;

        // Build the pool (create directory structure, etc.)
        pool.build(0)
            .map_err(|e| AppError::LibvirtError(format!("Failed to build storage pool: {}", e)))?;

        // Set autostart if requested
        if config.autostart {
            pool.set_autostart(true)
                .map_err(map_libvirt_error)?;
        }

        // Start the pool
        pool.create(0)
            .map_err(|e| AppError::LibvirtError(format!("Failed to start storage pool: {}", e)))?;

        let uuid = pool.get_uuid_string()
            .map_err(map_libvirt_error)?;

        tracing::info!("Storage pool created successfully: {} (UUID: {})", config.name, uuid);
        Ok(uuid)
    }

    /// Build XML for directory-based storage pool
    fn build_dir_pool_xml(config: &crate::models::storage::StoragePoolConfig) -> Result<String, AppError> {
        let xml = format!(
            r#"<pool type='dir'>
  <name>{}</name>
  <target>
    <path>{}</path>
    <permissions>
      <mode>0711</mode>
    </permissions>
  </target>
</pool>"#,
            config.name, config.target_path
        );
        Ok(xml)
    }

    /// Build XML for LVM logical volume storage pool
    fn build_logical_pool_xml(config: &crate::models::storage::StoragePoolConfig) -> Result<String, AppError> {
        if config.source_devices.is_empty() {
            return Err(AppError::InvalidConfig("Logical pool requires at least one source device".to_string()));
        }

        let devices_xml = config.source_devices.iter()
            .map(|dev| format!("    <device path='{}'/>\n", dev))
            .collect::<String>();

        let xml = format!(
            r#"<pool type='logical'>
  <name>{}</name>
  <source>
{}  </source>
  <target>
    <path>{}</path>
  </target>
</pool>"#,
            config.name, devices_xml, config.target_path
        );
        Ok(xml)
    }

    /// Build XML for network filesystem storage pool
    fn build_netfs_pool_xml(config: &crate::models::storage::StoragePoolConfig) -> Result<String, AppError> {
        let host = config.source_host.as_ref()
            .ok_or_else(|| AppError::InvalidConfig("Network pool requires source_host".to_string()))?;
        let source_path = config.source_path.as_ref()
            .ok_or_else(|| AppError::InvalidConfig("Network pool requires source_path".to_string()))?;

        let xml = format!(
            r#"<pool type='netfs'>
  <name>{}</name>
  <source>
    <host name='{}'/>
    <dir path='{}'/>
    <format type='nfs'/>
  </source>
  <target>
    <path>{}</path>
  </target>
</pool>"#,
            config.name, host, source_path, config.target_path
        );
        Ok(xml)
    }

    /// Build XML for iSCSI storage pool
    fn build_iscsi_pool_xml(config: &crate::models::storage::StoragePoolConfig) -> Result<String, AppError> {
        let host = config.source_host.as_ref()
            .ok_or_else(|| AppError::InvalidConfig("iSCSI pool requires source_host".to_string()))?;
        let target = config.iscsi_target.as_ref()
            .ok_or_else(|| AppError::InvalidConfig("iSCSI pool requires iscsi_target".to_string()))?;

        // Optional initiator IQN
        let initiator_xml = match &config.initiator_iqn {
            Some(iqn) => format!("\n    <initiator>\n      <iqn name='{}'/>\n    </initiator>", iqn),
            None => String::new(),
        };

        let xml = format!(
            r#"<pool type='iscsi'>
  <name>{}</name>
  <source>
    <host name='{}'/>
    <device path='{}'/>{}
  </source>
  <target>
    <path>{}</path>
  </target>
</pool>"#,
            config.name, host, target, initiator_xml, config.target_path
        );
        Ok(xml)
    }

    /// Build XML for GlusterFS storage pool
    fn build_gluster_pool_xml(config: &crate::models::storage::StoragePoolConfig) -> Result<String, AppError> {
        let host = config.source_host.as_ref()
            .ok_or_else(|| AppError::InvalidConfig("Gluster pool requires source_host".to_string()))?;
        let volume = config.gluster_volume.as_ref()
            .ok_or_else(|| AppError::InvalidConfig("Gluster pool requires gluster_volume".to_string()))?;

        // Optional subdir path within the Gluster volume
        let dir_xml = match &config.source_path {
            Some(path) if !path.is_empty() => format!("\n    <dir path='{}'/>", path),
            _ => String::new(),
        };

        let xml = format!(
            r#"<pool type='gluster'>
  <name>{}</name>
  <source>
    <host name='{}'/>
    <name>{}</name>{}
  </source>
</pool>"#,
            config.name, host, volume, dir_xml
        );
        Ok(xml)
    }

    /// Build XML for Ceph RBD storage pool
    fn build_rbd_pool_xml(config: &crate::models::storage::StoragePoolConfig) -> Result<String, AppError> {
        let rbd_pool = config.rbd_pool.as_ref()
            .ok_or_else(|| AppError::InvalidConfig("RBD pool requires rbd_pool name".to_string()))?;

        if config.ceph_monitors.is_empty() {
            return Err(AppError::InvalidConfig("RBD pool requires at least one Ceph monitor".to_string()));
        }

        // Build monitor hosts XML
        let monitors_xml = config.ceph_monitors.iter()
            .map(|mon| format!("    <host name='{}'/>\n", mon))
            .collect::<String>();

        // Optional authentication
        let auth_xml = match (&config.ceph_auth_user, &config.ceph_auth_secret) {
            (Some(user), Some(secret)) => format!(
                "\n  <auth type='ceph' username='{}'>\n    <secret uuid='{}'/>\n  </auth>",
                user, secret
            ),
            (Some(user), None) => format!(
                "\n  <auth type='ceph' username='{}'/>",
                user
            ),
            _ => String::new(),
        };

        let xml = format!(
            r#"<pool type='rbd'>
  <name>{}</name>
  <source>
{}    <name>{}</name>
  </source>{}
</pool>"#,
            config.name, monitors_xml, rbd_pool, auth_xml
        );
        Ok(xml)
    }

    /// Resize a volume in a storage pool
    pub fn resize_volume(
        libvirt: &LibvirtService,
        pool_id: &str,
        volume_name: &str,
        new_capacity_gb: u64,
    ) -> Result<(), AppError> {
        tracing::info!("Resizing volume {} in pool {} to {}GB", volume_name, pool_id, new_capacity_gb);

        let conn = libvirt.get_connection();
        let pool = StoragePool::lookup_by_uuid_string(conn, pool_id)
            .map_err(|_| AppError::LibvirtError(format!("Storage pool not found: {}", pool_id)))?;

        // Lookup the volume
        let volume = StorageVol::lookup_by_name(&pool, volume_name)
            .map_err(|_| AppError::LibvirtError(format!("Volume not found: {}", volume_name)))?;

        // Get current capacity
        let info = volume.get_info()
            .map_err(map_libvirt_error)?;

        let current_capacity_gb = info.capacity / (1024 * 1024 * 1024);

        if new_capacity_gb <= current_capacity_gb {
            return Err(AppError::InvalidConfig(
                format!("New capacity ({}GB) must be greater than current capacity ({}GB)",
                        new_capacity_gb, current_capacity_gb)
            ));
        }

        // Convert GB to bytes
        let new_capacity_bytes = new_capacity_gb * 1024 * 1024 * 1024;

        // Resize the volume
        volume.resize(new_capacity_bytes, 0)
            .map_err(|e| AppError::LibvirtError(format!("Failed to resize volume: {}", e)))?;

        tracing::info!("Volume resized successfully: {} ({}GB -> {}GB)",
                      volume_name, current_capacity_gb, new_capacity_gb);
        Ok(())
    }

    /// Upload a file to a storage volume
    /// This creates or overwrites a volume with the contents of a local file
    pub fn upload_volume(
        libvirt: &LibvirtService,
        pool_id: &str,
        volume_name: &str,
        source_path: &str,
        format: Option<&str>,
    ) -> Result<Volume, AppError> {
        use std::fs;
        use std::path::Path;

        tracing::info!("Uploading file {} to volume {} in pool {}", source_path, volume_name, pool_id);

        let source = Path::new(source_path);
        if !source.exists() {
            return Err(AppError::InvalidConfig(format!("Source file not found: {}", source_path)));
        }

        let metadata = fs::metadata(source)
            .map_err(|e| AppError::Other(format!("Failed to read source file metadata: {}", e)))?;
        let file_size = metadata.len();

        // Determine format from file extension or parameter
        let vol_format = format.unwrap_or_else(|| {
            match source.extension().and_then(|e| e.to_str()) {
                Some("qcow2") => "qcow2",
                Some("raw") | Some("img") => "raw",
                Some("vmdk") => "vmdk",
                Some("vdi") => "vdi",
                Some("vpc") | Some("vhd") => "vpc",
                Some("iso") => "raw",
                _ => "raw",
            }
        });

        let conn = libvirt.get_connection();
        let pool = StoragePool::lookup_by_uuid_string(conn, pool_id)
            .map_err(|_| AppError::LibvirtError(format!("Storage pool not found: {}", pool_id)))?;

        let pool_name = pool.get_name()
            .map_err(map_libvirt_error)?;

        // Refresh the pool to see current state
        pool.refresh(0).ok();

        // Check if volume already exists and delete it
        if let Ok(existing_vol) = StorageVol::lookup_by_name(&pool, volume_name) {
            tracing::info!("Deleting existing volume {} before upload", volume_name);
            existing_vol.delete(0)
                .map_err(|e| AppError::LibvirtError(format!("Failed to delete existing volume: {}", e)))?;
        }

        // Create volume with appropriate size
        let volume_xml = format!(
            r#"<volume type='file'>
  <name>{}</name>
  <capacity unit='bytes'>{}</capacity>
  <target>
    <format type='{}'/>
  </target>
</volume>"#,
            volume_name, file_size, vol_format
        );

        let volume = StorageVol::create_xml(&pool, &volume_xml, 0)
            .map_err(|e| AppError::LibvirtError(format!("Failed to create volume: {}", e)))?;

        // Get the volume path
        let vol_path = volume.get_path()
            .map_err(map_libvirt_error)?;

        // Copy the file to the volume location
        fs::copy(source, &vol_path)
            .map_err(|e| AppError::Other(format!("Failed to copy file to volume: {}", e)))?;

        tracing::info!("Successfully uploaded {} bytes to volume {}", file_size, volume_name);

        // Convert to our Volume model
        Self::volume_to_model(&volume, &pool_name)
    }

    /// Download a volume to a local file
    pub fn download_volume(
        libvirt: &LibvirtService,
        pool_id: &str,
        volume_name: &str,
        dest_path: &str,
    ) -> Result<u64, AppError> {
        use std::fs;
        use std::path::Path;

        tracing::info!("Downloading volume {} from pool {} to {}", volume_name, pool_id, dest_path);

        let conn = libvirt.get_connection();
        let pool = StoragePool::lookup_by_uuid_string(conn, pool_id)
            .map_err(|_| AppError::LibvirtError(format!("Storage pool not found: {}", pool_id)))?;

        let volume = StorageVol::lookup_by_name(&pool, volume_name)
            .map_err(|_| AppError::LibvirtError(format!("Volume not found: {}", volume_name)))?;

        let vol_path = volume.get_path()
            .map_err(map_libvirt_error)?;

        let source = Path::new(&vol_path);
        if !source.exists() {
            return Err(AppError::Other(format!("Volume file not found: {}", vol_path)));
        }

        let dest = Path::new(dest_path);

        // Ensure destination directory exists
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::Other(format!("Failed to create destination directory: {}", e)))?;
        }

        // Copy the volume to destination
        let bytes_copied = fs::copy(source, dest)
            .map_err(|e| AppError::Other(format!("Failed to copy volume to destination: {}", e)))?;

        tracing::info!("Successfully downloaded {} bytes from volume {}", bytes_copied, volume_name);
        Ok(bytes_copied)
    }

    /// Get the path of a volume (useful for direct file operations)
    pub fn get_volume_path(
        libvirt: &LibvirtService,
        pool_id: &str,
        volume_name: &str,
    ) -> Result<String, AppError> {
        let conn = libvirt.get_connection();
        let pool = StoragePool::lookup_by_uuid_string(conn, pool_id)
            .map_err(|_| AppError::LibvirtError(format!("Storage pool not found: {}", pool_id)))?;

        let volume = StorageVol::lookup_by_name(&pool, volume_name)
            .map_err(|_| AppError::LibvirtError(format!("Volume not found: {}", volume_name)))?;

        volume.get_path().map_err(map_libvirt_error)
    }
}
