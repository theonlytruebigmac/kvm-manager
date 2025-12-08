use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub name: String,
    pub description: Option<String>,
    pub creation_time: i64,
    pub state: SnapshotState,
    pub parent: Option<String>,
    pub is_current: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SnapshotState {
    DiskSnapshot,
    Running,
    Paused,
    Shutoff,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotConfig {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub include_memory: bool,
}
