use serde::{Deserialize, Serialize};

/// Identifies the host integration being performed without including its inputs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    Query,
    Mutation,
    Console,
    HostLocal,
    Migration,
    Background,
}

/// Scope of a libvirt connection. This is deliberately less specific than its URI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionScope {
    LocalSystem,
    LocalSession,
    Remote,
    Test,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityState {
    Available,
    Unavailable,
    Degraded,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MutationOutcome {
    Rejected,
    Applied,
    RolledBack,
    Partial,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryActionKind {
    Retry,
    Reconnect,
    Reselect,
    Inspect,
    Reconcile,
    OpenSettings,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryAction {
    pub kind: RecoveryActionKind,
    pub label: String,
    pub requires_confirmation: bool,
    pub expected_connection_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionCapability {
    pub kind: String,
    pub state: CapabilityState,
    pub reason_code: Option<String>,
    pub recovery_action: Option<RecoveryAction>,
    pub checked_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TargetIdentity {
    pub resource_kind: String,
    pub stable_id: String,
    pub display_name: Option<String>,
}

/// The safe, explicit terminal state returned by a mutation. It makes successful and reconciled
/// operations observable without exposing raw integration details.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MutationResult {
    pub operation_id: String,
    pub connection_id: String,
    pub target: TargetIdentity,
    pub outcome: MutationOutcome,
}

impl MutationResult {
    pub fn from_context(
        context: &OperationContext,
        target: TargetIdentity,
        outcome: MutationOutcome,
    ) -> Self {
        Self {
            operation_id: context.operation_id.clone(),
            connection_id: context.connection_id.clone(),
            target,
            outcome,
        }
    }
}

/// The immutable, serializable portion of an operation context. A live libvirt handle is retained
/// separately by the connection service and must never cross IPC or diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OperationContext {
    pub operation_id: String,
    pub operation_kind: OperationKind,
    pub connection_id: String,
    pub connection_label: String,
    pub connection_scope: ConnectionScope,
    pub capabilities: Vec<ConnectionCapability>,
    pub target: Option<TargetIdentity>,
    pub captured_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationToken {
    pub operation_kind: OperationKind,
    pub connection_id: String,
    pub target: TargetIdentity,
    pub effect_digest: String,
    pub expires_at: String,
}

/// A preview returned before a destructive action. The token is opaque and can be consumed only
/// once by the backend confirmation store.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationPreview {
    pub token: String,
    pub confirmation: ConfirmationToken,
}
