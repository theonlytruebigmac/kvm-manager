use serde::{Deserialize, Serialize};

/// Storage Pool State enumeration
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PoolState {
    Active,
    Inactive,
    Degraded,
}

/// Storage Pool Type enumeration
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PoolType {
    Dir,
    Fs,
    Netfs,
    Logical,
    Disk,
    Iscsi,
    Scsi,
    Mpath,
    Rbd,
    Sheepdog,
    Gluster,
    Zfs,
}

/// Storage Pool model
/// Represents a libvirt storage pool
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StoragePool {
    pub id: String,
    pub name: String,
    pub state: PoolState,
    pub pool_type: PoolType,
    pub capacity_bytes: u64,
    pub allocation_bytes: u64,
    pub available_bytes: u64,
    pub path: String,
    pub autostart: bool,
}

/// Volume model
/// Represents a storage volume (disk image) within a storage pool
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
    pub name: String,
    pub path: String,
    pub pool_name: String,
    pub capacity_bytes: u64,
    pub allocation_bytes: u64,
    pub format: String,
}

/// Volume Configuration for creating new volumes
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VolumeConfig {
    pub name: String,
    pub capacity_gb: u64,
    #[serde(default = "default_volume_format")]
    pub format: String,
    /// Enable LUKS encryption for this volume
    #[serde(default)]
    pub encrypted: bool,
    /// Encryption passphrase (required if encrypted = true)
    #[serde(default)]
    pub passphrase: Option<String>,
}

/// Encryption info for a volume
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VolumeEncryptionInfo {
    pub encrypted: bool,
    pub format: Option<String>,  // "luks", "qcow"
    pub secret_uuid: Option<String>,
}

fn default_volume_format() -> String {
    "qcow2".to_string()
}

/// Storage Pool Configuration for creating new storage pools
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StoragePoolConfig {
    pub name: String,
    pub pool_type: String, // "dir", "logical", "netfs", "iscsi", "gluster", "rbd", etc.
    pub target_path: String,
    #[serde(default)]
    pub autostart: bool,
    // For logical pools
    #[serde(default)]
    pub source_devices: Vec<String>,
    // For netfs/iSCSI/gluster pools
    #[serde(default)]
    pub source_host: Option<String>,
    #[serde(default)]
    pub source_path: Option<String>,
    // For iSCSI pools
    #[serde(default)]
    pub iscsi_target: Option<String>,
    #[serde(default)]
    pub initiator_iqn: Option<String>,
    // For Gluster pools
    #[serde(default)]
    pub gluster_volume: Option<String>,
    // For RBD (Ceph) pools
    #[serde(default)]
    pub rbd_pool: Option<String>,
    #[serde(default)]
    pub ceph_monitors: Vec<String>,
    #[serde(default)]
    pub ceph_auth_user: Option<String>,
    #[serde(default)]
    pub ceph_auth_secret: Option<String>,
}
