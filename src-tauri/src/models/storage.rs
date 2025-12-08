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
}

fn default_volume_format() -> String {
    "qcow2".to_string()
}
