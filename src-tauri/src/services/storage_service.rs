use crate::models::host::{StorageChoice, StorageReadiness, StorageReadinessState};
use crate::models::operation::{RecoveryAction, RecoveryActionKind};
use crate::models::storage::{
    PoolState, PoolType, StoragePool as StoragePoolModel, Volume, VolumeConfig,
};
use crate::services::libvirt::ConnectionProvider;
use crate::utils::error::{map_libvirt_error, AppError};
use crate::utils::xml::{
    escaped_attribute, first_element_attribute, first_element_text, validate_identifier,
    validate_text, xml_text_element,
};
use virt::storage_pool::StoragePool;
use virt::storage_vol::StorageVol;
use virt::stream::Stream;
use virt::sys;

/// StorageService provides storage pool and volume management operations
pub struct StorageService;

impl StorageService {
    pub fn disk_bytes(disk_size_gb: u64) -> Result<u64, AppError> {
        if disk_size_gb == 0 {
            return Err(AppError::InvalidConfig(
                "Disk size must be greater than zero".to_string(),
            ));
        }
        disk_size_gb
            .checked_mul(1024 * 1024 * 1024)
            .ok_or_else(|| AppError::InvalidConfig("Disk size is too large".to_string()))
    }

    pub fn pool_choice(pool: &StoragePoolModel, required_bytes: Option<u64>) -> StorageChoice {
        let active = pool.state == PoolState::Active;
        let capacity_ok = required_bytes
            .map(|required| pool.available_bytes >= required)
            .unwrap_or(true);
        let reason = if !active {
            Some("This storage pool is inactive.".to_string())
        } else if !capacity_ok {
            Some("This storage pool does not have enough available capacity.".to_string())
        } else {
            None
        };
        StorageChoice {
            id: pool.id.clone(),
            name: pool.name.clone(),
            state: pool.state.clone(),
            pool_type: pool.pool_type.clone(),
            capacity_bytes: pool.capacity_bytes,
            allocation_bytes: pool.allocation_bytes,
            available_bytes: pool.available_bytes,
            autostart: pool.autostart,
            eligible: active && capacity_ok,
            reason,
        }
    }

    pub fn assess_pools(
        connection_id: &str,
        pools: &[StoragePoolModel],
        required_bytes: Option<u64>,
        selected_pool_id: Option<&str>,
    ) -> StorageReadiness {
        let choices: Vec<_> = pools
            .iter()
            .map(|pool| Self::pool_choice(pool, required_bytes))
            .collect();
        let eligible_count = choices.iter().filter(|choice| choice.eligible).count();
        let selection_valid = selected_pool_id
            .and_then(|id| choices.iter().find(|choice| choice.id == id))
            .map(|choice| choice.eligible)
            .unwrap_or(false);
        let state = if selection_valid {
            StorageReadinessState::Ready
        } else if eligible_count > 0 {
            StorageReadinessState::SelectionRequired
        } else if choices
            .iter()
            .any(|choice| choice.state == PoolState::Active)
        {
            StorageReadinessState::InsufficientCapacity
        } else {
            StorageReadinessState::Unavailable
        };
        let recovery_action = (state != StorageReadinessState::Ready).then(|| RecoveryAction {
            kind: if eligible_count > 0 {
                RecoveryActionKind::Reselect
            } else {
                RecoveryActionKind::Inspect
            },
            label: if eligible_count > 0 {
                "Select an eligible storage pool.".to_string()
            } else {
                "Inspect or create storage for this connection.".to_string()
            },
            requires_confirmation: false,
            expected_connection_id: Some(connection_id.to_string()),
        });
        StorageReadiness {
            connection_id: connection_id.to_string(),
            required_bytes,
            selected_pool_id: selected_pool_id.map(str::to_string),
            pools: choices,
            state,
            recovery_action,
        }
    }

    pub fn storage_readiness(
        libvirt: &impl ConnectionProvider,
        connection_id: &str,
        required_bytes: Option<u64>,
        selected_pool_id: Option<&str>,
    ) -> Result<StorageReadiness, AppError> {
        let pools = Self::list_storage_pools(libvirt)?;
        Ok(Self::assess_pools(
            connection_id,
            &pools,
            required_bytes,
            selected_pool_id,
        ))
    }
    /// List all storage pools (active and inactive)
    pub fn list_storage_pools(
        libvirt: &impl ConnectionProvider,
    ) -> Result<Vec<StoragePoolModel>, AppError> {
        tracing::debug!("Listing all storage pools");

        let conn = libvirt.get_connection();
        let mut pools = Vec::new();

        // Get all storage pools (both active and inactive)
        let flags = sys::VIR_CONNECT_LIST_STORAGE_POOLS_ACTIVE
            | sys::VIR_CONNECT_LIST_STORAGE_POOLS_INACTIVE;
        let storage_pools = conn
            .list_all_storage_pools(flags)
            .map_err(map_libvirt_error)?;

        for pool in storage_pools {
            match Self::pool_to_model(&pool) {
                Ok(pool_model) => pools.push(pool_model),
                Err(_) => {
                    tracing::warn!("A storage pool could not be read");
                    continue;
                }
            }
        }

        tracing::info!("Found {} storage pools", pools.len());
        Ok(pools)
    }

    /// Convert a libvirt StoragePool to our StoragePool model
    fn pool_to_model(pool: &StoragePool) -> Result<StoragePoolModel, AppError> {
        let uuid = pool.get_uuid_string().map_err(map_libvirt_error)?;

        let name = pool.get_name().map_err(map_libvirt_error)?;

        let is_active = pool.is_active().map_err(map_libvirt_error)?;

        let state = if is_active {
            PoolState::Active
        } else {
            PoolState::Inactive
        };

        // Get pool info for capacity and allocation
        let info = pool.get_info().map_err(map_libvirt_error)?;

        let capacity_bytes = info.capacity;
        let allocation_bytes = info.allocation;
        let available_bytes = info.available;

        // Get pool XML to extract type and path
        let xml = pool.get_xml_desc(0).map_err(map_libvirt_error)?;

        let pool_type = Self::extract_pool_type(&xml)?;
        let path = Self::extract_pool_path(&xml)?;

        let autostart = pool.get_autostart().map_err(map_libvirt_error)?;

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
        match first_element_attribute(xml, "pool", "pool", "type")?.as_deref() {
            Some("dir") | None => Ok(PoolType::Dir),
            Some("fs") => Ok(PoolType::Fs),
            Some("netfs") => Ok(PoolType::Netfs),
            Some("logical") => Ok(PoolType::Logical),
            Some("disk") => Ok(PoolType::Disk),
            Some("iscsi") => Ok(PoolType::Iscsi),
            Some("scsi") => Ok(PoolType::Scsi),
            Some("mpath") => Ok(PoolType::Mpath),
            Some("rbd") => Ok(PoolType::Rbd),
            Some("sheepdog") => Ok(PoolType::Sheepdog),
            Some("gluster") => Ok(PoolType::Gluster),
            Some("zfs") => Ok(PoolType::Zfs),
            Some(_) => Ok(PoolType::Dir),
        }
    }

    /// Extract pool path from XML
    fn extract_pool_path(xml: &str) -> Result<String, AppError> {
        Ok(first_element_text(xml, "pool", "path")?.unwrap_or_default())
    }

    /// List all volumes in a storage pool
    pub fn list_volumes(
        libvirt: &impl ConnectionProvider,
        pool_id: &str,
    ) -> Result<Vec<Volume>, AppError> {
        tracing::debug!("Listing volumes in pool: {}", pool_id);

        let conn = libvirt.get_connection();
        let pool = StoragePool::lookup_by_uuid_string(conn, pool_id)
            .map_err(|_| AppError::LibvirtError(format!("Storage pool not found: {}", pool_id)))?;

        let pool_name = pool.get_name().map_err(map_libvirt_error)?;

        let mut volumes = Vec::new();

        // List all volumes in the pool
        let volume_names = pool.list_volumes().map_err(map_libvirt_error)?;

        for vol_name in volume_names {
            let volume = StorageVol::lookup_by_name(&pool, &vol_name).map_err(map_libvirt_error)?;

            match Self::volume_to_model(&volume, &pool_name) {
                Ok(vol_model) => volumes.push(vol_model),
                Err(_) => {
                    tracing::warn!("A storage volume could not be read");
                    continue;
                }
            }
        }

        tracing::info!("Found {} volumes in pool {}", volumes.len(), pool_name);
        Ok(volumes)
    }

    /// Convert a libvirt StorageVol to our Volume model
    fn volume_to_model(volume: &StorageVol, pool_name: &str) -> Result<Volume, AppError> {
        let name = volume.get_name().map_err(map_libvirt_error)?;

        let path = volume.get_path().map_err(map_libvirt_error)?;

        let info = volume.get_info().map_err(map_libvirt_error)?;

        let capacity_bytes = info.capacity;
        let allocation_bytes = info.allocation;

        // Get volume XML to extract format
        let xml = volume.get_xml_desc(0).map_err(map_libvirt_error)?;

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
        Ok(first_element_attribute(xml, "volume", "format", "type")?
            .unwrap_or_else(|| "raw".to_string()))
    }

    /// Check if a volume is encrypted and get encryption info
    pub fn get_volume_encryption_info(
        libvirt: &impl ConnectionProvider,
        pool_id: &str,
        volume_name: &str,
    ) -> Result<crate::models::storage::VolumeEncryptionInfo, AppError> {
        let conn = libvirt.get_connection();
        let pool = StoragePool::lookup_by_uuid_string(conn, pool_id)
            .map_err(|_| AppError::LibvirtError(format!("Storage pool not found: {}", pool_id)))?;

        let volume = StorageVol::lookup_by_name(&pool, volume_name)
            .map_err(|_| AppError::LibvirtError(format!("Volume not found: {}", volume_name)))?;

        let xml = volume.get_xml_desc(0).map_err(map_libvirt_error)?;

        let format = first_element_attribute(&xml, "volume", "encryption", "format")?;
        let encrypted = format.is_some();
        let format = format.map(|value| match value.as_str() {
            "luks" | "qcow" => value,
            _ => "unknown".to_string(),
        });
        let secret_uuid = first_element_attribute(&xml, "volume", "secret", "uuid")?;

        Ok(crate::models::storage::VolumeEncryptionInfo {
            encrypted,
            format,
            secret_uuid,
        })
    }

    /// Create a new volume in a storage pool
    pub fn create_volume(
        libvirt: &impl ConnectionProvider,
        pool_id: &str,
        config: VolumeConfig,
    ) -> Result<String, AppError> {
        tracing::info!(
            "Creating volume {} in pool {} (encrypted: {})",
            config.name,
            pool_id,
            config.encrypted
        );

        let conn = libvirt.get_connection();
        let pool = StoragePool::lookup_by_uuid_string(conn, pool_id)
            .map_err(|_| AppError::LibvirtError(format!("Storage pool not found: {}", pool_id)))?;

        // Check if pool is active
        let is_active = pool.is_active().map_err(map_libvirt_error)?;

        if !is_active {
            return Err(AppError::LibvirtError(
                "Storage pool is not active".to_string(),
            ));
        }

        let capacity_bytes = config
            .capacity_gb
            .checked_mul(1024 * 1024 * 1024)
            .ok_or_else(|| AppError::InvalidConfig("Volume capacity is too large".to_string()))?;

        // Handle encryption
        let (encryption_xml, secret_uuid) = if config.encrypted {
            let passphrase = config.passphrase.as_ref().ok_or_else(|| {
                AppError::InvalidConfig("Passphrase required for encrypted volume".to_string())
            })?;

            if passphrase.len() < 8 {
                return Err(AppError::InvalidConfig(
                    "Passphrase must be at least 8 characters".to_string(),
                ));
            }

            // Create a libvirt secret for the passphrase
            let secret_uuid = Self::create_volume_secret(libvirt, &config.name, passphrase)?;

            (Some(secret_uuid.clone()), Some(secret_uuid))
        } else {
            (None, None)
        };

        let volume_xml =
            Self::volume_definition(&config, capacity_bytes, encryption_xml.as_deref())?;

        tracing::debug!("Generated storage volume definition");

        // Create the volume
        let volume = match StorageVol::create_xml(&pool, &volume_xml, 0) {
            Ok(v) => v,
            Err(_) => {
                // A secret created for this request is owned by the failed operation.  Do not
                // report a rejected create as clean if that compensation cannot complete.
                let secret_removed = secret_uuid
                    .as_deref()
                    .map(|uuid| Self::delete_secret(libvirt, uuid).is_ok())
                    .unwrap_or(true);
                return if secret_removed {
                    Err(AppError::LibvirtError(
                        "Storage volume creation failed".to_string(),
                    ))
                } else {
                    Err(AppError::Partial(
                        "Storage volume creation failed after creating an encryption secret"
                            .to_string(),
                    ))
                };
            }
        };

        let path = match volume.get_path() {
            Ok(path) => path,
            Err(_) => {
                // The caller cannot safely use a volume whose identity could not be returned.
                // Remove both resources created by this request, or make the remaining state
                // explicit so the UI can direct the operator to inspect it.
                let volume_removed = volume.delete(0).is_ok();
                let secret_removed = secret_uuid
                    .as_deref()
                    .map(|uuid| Self::delete_secret(libvirt, uuid).is_ok())
                    .unwrap_or(true);
                return if volume_removed && secret_removed {
                    Err(AppError::LibvirtError(
                        "Storage volume could not be finalized".to_string(),
                    ))
                } else {
                    Err(AppError::Partial(
                        "Storage volume finalization left resources that require inspection"
                            .to_string(),
                    ))
                };
            }
        };

        tracing::info!(
            encrypted = config.encrypted,
            "Storage volume created successfully"
        );
        Ok(path)
    }

    pub fn volume_definition(
        config: &VolumeConfig,
        capacity_bytes: u64,
        secret_uuid: Option<&str>,
    ) -> Result<String, AppError> {
        validate_storage_name(&config.name, "volume name")?;
        validate_volume_format(&config.format)?;
        let mut xml = String::from("<volume>\n  ");
        xml.push_str(&xml_text_element("name", &config.name)?);
        xml.push_str(&format!(
            "\n  <capacity unit='bytes'>{}</capacity>\n  <target>\n    <format type='{}'/>",
            capacity_bytes, config.format
        ));
        if let Some(secret_uuid) = secret_uuid {
            uuid::Uuid::parse_str(secret_uuid).map_err(|_| {
                AppError::InvalidConfig("Encryption secret identifier is invalid".to_string())
            })?;
            xml.push_str(&format!(
                "\n    <encryption format='luks'><secret type='passphrase' uuid='{}'/></encryption>",
                secret_uuid
            ));
        }
        xml.push_str("\n  </target>\n</volume>");
        crate::utils::xml::validate_document_root(&xml, "volume")?;
        Ok(xml)
    }

    fn volume_secret_definition(volume_name: &str, secret_uuid: &str) -> Result<String, AppError> {
        validate_storage_name(volume_name, "volume name")?;
        uuid::Uuid::parse_str(secret_uuid).map_err(|_| {
            AppError::InvalidConfig("Encryption secret identifier is invalid".to_string())
        })?;
        let volume = xml_text_element("volume", volume_name)?;
        let description = xml_text_element(
            "description",
            &format!("LUKS passphrase for volume {volume_name}"),
        )?;
        let xml = format!(
            "<secret ephemeral='no' private='yes'><uuid>{}</uuid>{}<usage type='volume'>{}</usage></secret>",
            secret_uuid, description, volume
        );
        crate::utils::xml::validate_document_root(&xml, "secret")?;
        Ok(xml)
    }

    /// Create a libvirt secret for volume encryption
    fn create_volume_secret(
        libvirt: &impl ConnectionProvider,
        volume_name: &str,
        passphrase: &str,
    ) -> Result<String, AppError> {
        use uuid::Uuid;

        let secret_uuid = Uuid::new_v4().to_string();
        let conn = libvirt.get_connection();

        let secret_xml = Self::volume_secret_definition(volume_name, &secret_uuid)?;

        tracing::debug!("Creating encryption secret for volume");

        // Define the secret
        let secret = virt::secret::Secret::define_xml(conn, &secret_xml, 0)
            .map_err(|e| AppError::LibvirtError(format!("Failed to create secret: {}", e)))?;

        // Set the secret value (passphrase)
        if secret.set_value(passphrase.as_bytes(), 0).is_err() {
            return if secret.undefine().is_ok() {
                Err(AppError::LibvirtError(
                    "Failed to initialize the encryption secret".to_string(),
                ))
            } else {
                Err(AppError::Partial(
                    "An encryption secret may require inspection".to_string(),
                ))
            };
        }

        tracing::info!("Created encryption secret");
        Ok(secret_uuid)
    }

    /// Delete a libvirt secret
    fn delete_secret(libvirt: &impl ConnectionProvider, uuid: &str) -> Result<(), AppError> {
        let conn = libvirt.get_connection();

        if let Ok(secret) = virt::secret::Secret::lookup_by_uuid_string(conn, uuid) {
            secret
                .undefine()
                .map_err(|e| AppError::LibvirtError(format!("Failed to delete secret: {}", e)))?;
            tracing::info!("Deleted encryption secret");
        }

        Ok(())
    }

    /// Delete a volume from a storage pool
    pub fn delete_volume(
        libvirt: &impl ConnectionProvider,
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
        volume
            .delete(0)
            .map_err(|e| AppError::LibvirtError(format!("Failed to delete volume: {}", e)))?;

        tracing::info!("Volume deleted successfully: {}", volume_name);
        Ok(())
    }

    /// Create a new storage pool
    pub fn create_storage_pool(
        libvirt: &impl ConnectionProvider,
        config: crate::models::storage::StoragePoolConfig,
    ) -> Result<String, AppError> {
        tracing::info!(
            "Creating storage pool: {} (type: {})",
            config.name,
            config.pool_type
        );

        let conn = libvirt.get_connection();

        let pool_xml = Self::pool_definition(&config)?;

        tracing::debug!("Generated storage pool definition");

        // Define the pool
        let pool = StoragePool::define_xml(conn, &pool_xml, 0)
            .map_err(|e| AppError::LibvirtError(format!("Failed to define storage pool: {}", e)))?;

        // Build the pool (create directory structure, etc.)
        pool.build(0)
            .map_err(|e| AppError::LibvirtError(format!("Failed to build storage pool: {}", e)))?;

        // Set autostart if requested
        if config.autostart {
            pool.set_autostart(true).map_err(map_libvirt_error)?;
        }

        // Start the pool
        pool.create(0)
            .map_err(|e| AppError::LibvirtError(format!("Failed to start storage pool: {}", e)))?;

        let uuid = pool.get_uuid_string().map_err(map_libvirt_error)?;

        tracing::info!(
            "Storage pool created successfully: {} (UUID: {})",
            config.name,
            uuid
        );
        Ok(uuid)
    }

    pub fn pool_definition(
        config: &crate::models::storage::StoragePoolConfig,
    ) -> Result<String, AppError> {
        match config.pool_type.as_str() {
            "dir" => Self::build_dir_pool_xml(config),
            "logical" => Self::build_logical_pool_xml(config),
            "netfs" => Self::build_netfs_pool_xml(config),
            "iscsi" => Self::build_iscsi_pool_xml(config),
            "gluster" => Self::build_gluster_pool_xml(config),
            "rbd" => Self::build_rbd_pool_xml(config),
            _ => Err(AppError::InvalidConfig(format!(
                "Unsupported pool type: {}",
                config.pool_type
            ))),
        }
    }

    /// Build XML for directory-based storage pool
    fn build_dir_pool_xml(
        config: &crate::models::storage::StoragePoolConfig,
    ) -> Result<String, AppError> {
        let xml = Self::pool_document("dir", config, String::new(), true)?;
        Ok(xml)
    }

    /// Build XML for LVM logical volume storage pool
    fn build_logical_pool_xml(
        config: &crate::models::storage::StoragePoolConfig,
    ) -> Result<String, AppError> {
        if config.source_devices.is_empty() {
            return Err(AppError::InvalidConfig(
                "Logical pool requires at least one source device".to_string(),
            ));
        }

        let mut source = String::new();
        for device in &config.source_devices {
            source.push_str("<device path='");
            source.push_str(&escaped_path(device, "source device")?);
            source.push_str("'/>");
        }
        let xml = Self::pool_document("logical", config, source, true)?;
        Ok(xml)
    }

    /// Build XML for network filesystem storage pool
    fn build_netfs_pool_xml(
        config: &crate::models::storage::StoragePoolConfig,
    ) -> Result<String, AppError> {
        let host = config.source_host.as_ref().ok_or_else(|| {
            AppError::InvalidConfig("Network pool requires source_host".to_string())
        })?;
        let source_path = config.source_path.as_ref().ok_or_else(|| {
            AppError::InvalidConfig("Network pool requires source_path".to_string())
        })?;

        let source = format!(
            "<host name='{}'/><dir path='{}'/><format type='nfs'/>",
            escaped_host(host)?,
            escaped_path(source_path, "network source path")?
        );
        let xml = Self::pool_document("netfs", config, source, true)?;
        Ok(xml)
    }

    /// Build XML for iSCSI storage pool
    fn build_iscsi_pool_xml(
        config: &crate::models::storage::StoragePoolConfig,
    ) -> Result<String, AppError> {
        let host = config.source_host.as_ref().ok_or_else(|| {
            AppError::InvalidConfig("iSCSI pool requires source_host".to_string())
        })?;
        let target = config.iscsi_target.as_ref().ok_or_else(|| {
            AppError::InvalidConfig("iSCSI pool requires iscsi_target".to_string())
        })?;

        let mut source = format!(
            "<host name='{}'/><device path='{}'/>",
            escaped_host(host)?,
            escaped_path(target, "iSCSI target")?
        );
        if let Some(iqn) = &config.initiator_iqn {
            validate_text(iqn, "iSCSI initiator")?;
            source.push_str("<initiator><iqn name='");
            source.push_str(&escaped_attribute(iqn, "iSCSI initiator")?);
            source.push_str("'/></initiator>");
        }
        let xml = Self::pool_document("iscsi", config, source, true)?;
        Ok(xml)
    }

    /// Build XML for GlusterFS storage pool
    fn build_gluster_pool_xml(
        config: &crate::models::storage::StoragePoolConfig,
    ) -> Result<String, AppError> {
        let host = config.source_host.as_ref().ok_or_else(|| {
            AppError::InvalidConfig("Gluster pool requires source_host".to_string())
        })?;
        let volume = config.gluster_volume.as_ref().ok_or_else(|| {
            AppError::InvalidConfig("Gluster pool requires gluster_volume".to_string())
        })?;

        validate_storage_name(volume, "Gluster volume")?;
        let mut source = format!(
            "<host name='{}'/>{}",
            escaped_host(host)?,
            xml_text_element("name", volume)?
        );
        if let Some(path) = config
            .source_path
            .as_deref()
            .filter(|path| !path.is_empty())
        {
            source.push_str("<dir path='");
            source.push_str(&escaped_path(path, "Gluster directory")?);
            source.push_str("'/>");
        }
        let xml = Self::pool_document("gluster", config, source, false)?;
        Ok(xml)
    }

    /// Build XML for Ceph RBD storage pool
    fn build_rbd_pool_xml(
        config: &crate::models::storage::StoragePoolConfig,
    ) -> Result<String, AppError> {
        let rbd_pool = config.rbd_pool.as_ref().ok_or_else(|| {
            AppError::InvalidConfig("RBD pool requires rbd_pool name".to_string())
        })?;

        if config.ceph_monitors.is_empty() {
            return Err(AppError::InvalidConfig(
                "RBD pool requires at least one Ceph monitor".to_string(),
            ));
        }

        validate_storage_name(rbd_pool, "RBD pool")?;
        let mut source = String::new();
        for monitor in &config.ceph_monitors {
            source.push_str("<host name='");
            source.push_str(&escaped_host(monitor)?);
            source.push_str("'/>");
        }
        source.push_str(&xml_text_element("name", rbd_pool)?);
        let auth = match (&config.ceph_auth_user, &config.ceph_auth_secret) {
            (Some(user), Some(secret)) => {
                validate_storage_name(user, "Ceph auth user")?;
                uuid::Uuid::parse_str(secret).map_err(|_| {
                    AppError::InvalidConfig(
                        "Ceph authentication secret identifier is invalid".to_string(),
                    )
                })?;
                format!(
                    "<auth type='ceph' username='{}'><secret uuid='{}'/></auth>",
                    user, secret
                )
            }
            (Some(user), None) => {
                validate_storage_name(user, "Ceph auth user")?;
                format!("<auth type='ceph' username='{}'/>", user)
            }
            (None, Some(_)) => {
                return Err(AppError::InvalidConfig(
                    "Ceph auth user is required with a secret".to_string(),
                ))
            }
            (None, None) => String::new(),
        };
        validate_storage_name(&config.name, "storage pool name")?;
        let xml = format!(
            "<pool type='rbd'>{}<source>{}</source>{}</pool>",
            xml_text_element("name", &config.name)?,
            source,
            auth
        );
        crate::utils::xml::validate_document_root(&xml, "pool")?;
        Ok(xml)
    }

    fn pool_document(
        pool_type: &str,
        config: &crate::models::storage::StoragePoolConfig,
        source: String,
        requires_target: bool,
    ) -> Result<String, AppError> {
        validate_storage_name(&config.name, "storage pool name")?;
        let mut xml = format!(
            "<pool type='{}'>{}",
            pool_type,
            xml_text_element("name", &config.name)?
        );
        if !source.is_empty() {
            xml.push_str("<source>");
            xml.push_str(&source);
            xml.push_str("</source>");
        }
        if requires_target {
            if config.target_path.trim().is_empty() {
                return Err(AppError::InvalidConfig(
                    "Storage pool target path is required".to_string(),
                ));
            }
            xml.push_str("<target>");
            xml.push_str(&xml_text_element(
                "path",
                &validated_path(&config.target_path, "storage pool target path")?,
            )?);
            xml.push_str("</target>");
        }
        xml.push_str("</pool>");
        crate::utils::xml::validate_document_root(&xml, "pool")?;
        Ok(xml)
    }

    /// Resize a volume in a storage pool
    pub fn resize_volume(
        libvirt: &impl ConnectionProvider,
        pool_id: &str,
        volume_name: &str,
        new_capacity_gb: u64,
    ) -> Result<(), AppError> {
        tracing::info!(
            "Resizing volume {} in pool {} to {}GB",
            volume_name,
            pool_id,
            new_capacity_gb
        );

        let conn = libvirt.get_connection();
        let pool = StoragePool::lookup_by_uuid_string(conn, pool_id)
            .map_err(|_| AppError::LibvirtError(format!("Storage pool not found: {}", pool_id)))?;

        // Lookup the volume
        let volume = StorageVol::lookup_by_name(&pool, volume_name)
            .map_err(|_| AppError::LibvirtError(format!("Volume not found: {}", volume_name)))?;

        // Get current capacity
        let info = volume.get_info().map_err(map_libvirt_error)?;

        let current_capacity_gb = info.capacity / (1024 * 1024 * 1024);

        if new_capacity_gb <= current_capacity_gb {
            return Err(AppError::InvalidConfig(format!(
                "New capacity ({}GB) must be greater than current capacity ({}GB)",
                new_capacity_gb, current_capacity_gb
            )));
        }

        // Convert GB to bytes
        let new_capacity_bytes = new_capacity_gb * 1024 * 1024 * 1024;

        // Resize the volume
        volume
            .resize(new_capacity_bytes, 0)
            .map_err(|e| AppError::LibvirtError(format!("Failed to resize volume: {}", e)))?;

        tracing::info!(
            "Volume resized successfully: {} ({}GB -> {}GB)",
            volume_name,
            current_capacity_gb,
            new_capacity_gb
        );
        Ok(())
    }

    /// Upload a file to a storage volume
    /// This creates a volume and transfers through libvirt so the pool owns permissions/policy.
    pub fn upload_volume(
        libvirt: &impl ConnectionProvider,
        pool_id: &str,
        volume_name: &str,
        source_path: &str,
        format: Option<&str>,
    ) -> Result<Volume, AppError> {
        use std::fs;
        use std::io::Read;
        use std::path::Path;

        tracing::info!("Uploading a local file to the requested volume");

        let source = Path::new(source_path);
        if !source.exists() {
            return Err(AppError::InvalidConfig(format!(
                "Source file not found: {}",
                source_path
            )));
        }

        let metadata = fs::metadata(source)
            .map_err(|e| AppError::Other(format!("Failed to read source file metadata: {}", e)))?;
        let file_size = metadata.len();

        // Determine format from file extension or parameter
        let vol_format =
            format.unwrap_or_else(|| match source.extension().and_then(|e| e.to_str()) {
                Some("qcow2") => "qcow2",
                Some("raw") | Some("img") => "raw",
                Some("vmdk") => "vmdk",
                Some("vdi") => "vdi",
                Some("vpc") | Some("vhd") => "vpc",
                Some("iso") => "raw",
                _ => "raw",
            });

        let conn = libvirt.get_connection();
        let pool = StoragePool::lookup_by_uuid_string(conn, pool_id)
            .map_err(|_| AppError::LibvirtError(format!("Storage pool not found: {}", pool_id)))?;

        let pool_name = pool.get_name().map_err(map_libvirt_error)?;

        if !pool.is_active().map_err(map_libvirt_error)? {
            return Err(AppError::Unavailable(
                "The selected storage pool is inactive".to_string(),
            ));
        }
        pool.refresh(0).map_err(map_libvirt_error)?;
        let pool_info = pool.get_info().map_err(map_libvirt_error)?;
        if pool_info.available < file_size {
            return Err(AppError::InvalidConfig(
                "The selected storage pool does not have enough available capacity".to_string(),
            ));
        }

        // Import never overwrites implicitly. This keeps the source and existing pool content safe.
        if StorageVol::lookup_by_name(&pool, volume_name).is_ok() {
            return Err(AppError::InvalidConfig(
                "A volume with that name already exists in the selected pool".to_string(),
            ));
        }

        // Create volume with appropriate size using the same validated definition boundary.
        let volume_xml = Self::volume_definition(
            &VolumeConfig {
                name: volume_name.to_string(),
                capacity_gb: 0,
                format: vol_format.to_string(),
                encrypted: false,
                passphrase: None,
            },
            file_size,
            None,
        )?;

        let volume = StorageVol::create_xml(&pool, &volume_xml, 0)
            .map_err(|e| AppError::LibvirtError(format!("Failed to create volume: {}", e)))?;

        let mut source_file = fs::File::open(source).map_err(|_| {
            AppError::Unavailable("The selected source file cannot be read".to_string())
        })?;
        let stream = Stream::new(conn, 0).map_err(map_libvirt_error)?;
        if volume.upload(&stream, 0, file_size, 0).is_err() {
            let _ = stream.abort();
            let removed = volume.delete(0).is_ok();
            return Err(if removed {
                AppError::LibvirtError("The volume upload could not be started".to_string())
            } else {
                AppError::Partial(
                    "The failed import left a volume that requires inspection".to_string(),
                )
            });
        }
        let transfer_result = (|| -> Result<(), AppError> {
            let mut buffer = vec![0_u8; 1024 * 1024];
            loop {
                let count = source_file.read(&mut buffer).map_err(|_| {
                    AppError::Unavailable("The selected source file could not be read".to_string())
                })?;
                if count == 0 {
                    break;
                }
                let mut sent = 0;
                while sent < count {
                    let written = stream.send(&buffer[sent..count]).map_err(|_| {
                        AppError::LibvirtError("The volume upload failed".to_string())
                    })?;
                    if written == 0 {
                        return Err(AppError::LibvirtError(
                            "The volume upload stopped before completion".to_string(),
                        ));
                    }
                    sent += written;
                }
            }
            stream.finish().map_err(|_| {
                AppError::LibvirtError("The volume upload could not be finalized".to_string())
            })
        })();
        if let Err(error) = transfer_result {
            let removed = volume.delete(0).is_ok();
            return Err(if removed {
                error
            } else {
                AppError::Partial(
                    "The failed import left a volume that requires inspection".to_string(),
                )
            });
        }

        tracing::info!(
            "Successfully uploaded {} bytes to volume {}",
            file_size,
            volume_name
        );

        // Convert to our Volume model
        Self::volume_to_model(&volume, &pool_name)
    }

    /// Download a volume to a local file
    pub fn download_volume(
        libvirt: &impl ConnectionProvider,
        pool_id: &str,
        volume_name: &str,
        dest_path: &str,
    ) -> Result<u64, AppError> {
        use std::fs;
        use std::path::Path;

        tracing::info!("Downloading the requested volume to a local file");

        let conn = libvirt.get_connection();
        let pool = StoragePool::lookup_by_uuid_string(conn, pool_id)
            .map_err(|_| AppError::LibvirtError(format!("Storage pool not found: {}", pool_id)))?;

        let volume = StorageVol::lookup_by_name(&pool, volume_name)
            .map_err(|_| AppError::LibvirtError(format!("Volume not found: {}", volume_name)))?;

        let vol_path = volume.get_path().map_err(map_libvirt_error)?;

        let source = Path::new(&vol_path);
        if !source.exists() {
            return Err(AppError::Other(format!(
                "Volume file not found: {}",
                vol_path
            )));
        }

        let dest = Path::new(dest_path);

        // Ensure destination directory exists
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::Other(format!("Failed to create destination directory: {}", e))
            })?;
        }

        // Copy the volume to destination
        let bytes_copied = fs::copy(source, dest)
            .map_err(|e| AppError::Other(format!("Failed to copy volume to destination: {}", e)))?;

        tracing::info!(
            "Successfully downloaded {} bytes from volume {}",
            bytes_copied,
            volume_name
        );
        Ok(bytes_copied)
    }

    /// Get the path of a volume (useful for direct file operations)
    pub fn get_volume_path(
        libvirt: &impl ConnectionProvider,
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

fn validate_storage_name(value: &str, field: &str) -> Result<(), AppError> {
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

fn validated_path(value: &str, field: &str) -> Result<String, AppError> {
    validate_text(value, field)?;
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(AppError::InvalidConfig(format!("{} is invalid", field)));
    }
    Ok(value.to_string())
}

fn escaped_path(value: &str, field: &str) -> Result<String, AppError> {
    escaped_attribute(&validated_path(value, field)?, field)
}

fn escaped_host(value: &str) -> Result<String, AppError> {
    validate_text(value, "storage host")?;
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(AppError::InvalidConfig(
            "Storage host is invalid".to_string(),
        ));
    }
    escaped_attribute(value, "storage host")
}

fn validate_volume_format(value: &str) -> Result<(), AppError> {
    if !matches!(value, "raw" | "qcow2" | "qcow" | "vmdk" | "vdi" | "vpc") {
        return Err(AppError::InvalidConfig(
            "Unsupported volume format".to_string(),
        ));
    }
    Ok(())
}
