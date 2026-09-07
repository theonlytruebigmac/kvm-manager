use serde::{Deserialize, Serialize};

use crate::models::operation::{ConnectionScope, RecoveryAction};
use crate::models::storage::{PoolState, PoolType};

/// Supported Linux distribution families used for safe setup guidance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistributionFamily {
    ArchCachyos,
    DebianUbuntu,
    FedoraRhel,
    Opensuse,
    BestEffort,
}

/// Distribution-aware setup and recovery guidance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistributionProfile {
    pub family: DistributionFamily,
    pub display_name: String,
    pub package_manager: String,
    pub supported: bool,
    pub packages: Vec<String>,
    pub service: String,
    pub permission_guidance: String,
    pub firmware_guidance: String,
    pub limitations: Vec<String>,
}

/// State of an independently-probed host capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityState {
    Available,
    Unavailable,
    Warning,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessRepairMode {
    Automated,
    Manual,
    Navigate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadinessRepairAction {
    pub id: String,
    pub mode: ReadinessRepairMode,
    pub title: String,
    pub effect: String,
    pub requires_privilege: bool,
    pub requires_confirmation: bool,
    pub expected_connection_id: String,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessRepairOutcome {
    Applied,
    Rejected,
    Cancelled,
    Failed,
    InspectionRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadinessRepairResult {
    pub action_id: String,
    pub connection_id: String,
    pub outcome: ReadinessRepairOutcome,
    pub summary: String,
}

/// A single host capability and safe remediation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityResult {
    pub kind: String,
    pub state: CapabilityState,
    pub summary: String,
    pub remediation: Option<String>,
    pub details: Option<String>,
    #[serde(default)]
    pub repair_action: Option<ReadinessRepairAction>,
}

/// A non-destructive assessment of the local host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostReadinessReport {
    pub checked_at: String,
    pub connection_uri: String,
    pub distribution: DistributionProfile,
    pub overall_state: String,
    pub capabilities: Vec<CapabilityResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageReadinessState {
    Ready,
    SelectionRequired,
    Unavailable,
    InsufficientCapacity,
}

/// A safe, connection-owned pool choice. Pool paths are intentionally excluded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageChoice {
    pub id: String,
    pub name: String,
    pub state: PoolState,
    pub pool_type: PoolType,
    pub capacity_bytes: u64,
    pub allocation_bytes: u64,
    pub available_bytes: u64,
    pub autostart: bool,
    pub eligible: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageReadiness {
    pub connection_id: String,
    pub required_bytes: Option<u64>,
    pub selected_pool_id: Option<String>,
    pub pools: Vec<StorageChoice>,
    pub state: StorageReadinessState,
    pub recovery_action: Option<RecoveryAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuestRequirements {
    pub firmware: String,
    pub tpm_enabled: bool,
    pub network: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmCreationReadiness {
    pub checked_at: String,
    pub connection_id: String,
    pub connection_label: String,
    pub connection_scope: ConnectionScope,
    pub distribution: DistributionProfile,
    pub overall_state: String,
    pub capabilities: Vec<CapabilityResult>,
    pub storage: StorageReadiness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuestCapabilityReview {
    pub checked_at: String,
    pub connection_id: String,
    pub requirements: GuestRequirements,
    pub capabilities: Vec<CapabilityResult>,
    pub storage: StorageReadiness,
    pub can_create: bool,
}

/// Raw, non-destructive probe values that can be independently mapped to UI diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostProbe {
    pub libvirt_access: bool,
    pub qemu_emulator: bool,
    pub kvm: bool,
    pub uefi: bool,
    pub secure_boot: bool,
    pub forwarding_privilege: bool,
}

/// A discovered UEFI firmware pair suitable for a VM definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareCandidate {
    pub boot_mode: String,
    pub code_path: String,
    pub vars_template_path: Option<String>,
    pub source: String,
}

/// Canonical forwarding state exposed over IPC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortForwardRule {
    pub network_id: String,
    pub protocol: String,
    pub host_port: u16,
    pub guest_address: String,
    pub guest_port: u16,
    pub state: String,
}

/// Host Information
/// Matches the contract defined in .agents/integration/tauri-commands.md
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HostInfo {
    pub hostname: String,
    pub cpu_model: String,
    pub cpu_count: u32,
    pub cpu_threads: u32,
    pub memory_total_mb: u64,
    pub memory_free_mb: u64,
    pub libvirt_version: String,
    pub qemu_version: String,
    pub hypervisor: String,
    pub active_vms: u32,
    pub total_vms: u32,
}

/// Connection Status
#[derive(Serialize, Clone, Debug)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub uri: String,
    pub error: Option<String>,
}

/// VNC Connection Information
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VncInfo {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub websocket_port: Option<u16>,
    #[serde(rename = "type")]
    pub graphics_type: Option<String>,
}
