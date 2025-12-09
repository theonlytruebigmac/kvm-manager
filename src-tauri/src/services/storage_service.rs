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

    /// Create a new volume in a storage pool
    pub fn create_volume(
        libvirt: &LibvirtService,
        pool_id: &str,
        config: VolumeConfig,
    ) -> Result<String, AppError> {
        tracing::info!("Creating volume {} in pool {}", config.name, pool_id);

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

        // Create volume XML
        let volume_xml = format!(
            r#"<volume>
  <name>{}</name>
  <capacity unit='bytes'>{}</capacity>
  <target>
    <format type='{}'/>
  </target>
</volume>"#,
            config.name, capacity_bytes, config.format
        );

        tracing::debug!("Volume XML: {}", volume_xml);

        // Create the volume
        let volume = StorageVol::create_xml(&pool, &volume_xml, 0)
            .map_err(|e| AppError::LibvirtError(format!("Failed to create volume: {}", e)))?;

        let path = volume.get_path()
            .map_err(map_libvirt_error)?;

        tracing::info!("Volume created successfully: {} at {}", config.name, path);
        Ok(path)
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
}
