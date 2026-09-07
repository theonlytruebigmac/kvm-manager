use crate::models::operation::{
    MutationOutcome, OperationContext, RecoveryAction, RecoveryActionKind, TargetIdentity,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Stable, user-safe failure categories returned over IPC.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SafeFailureCode {
    Unavailable,
    InvalidInput,
    Conflict,
    Unauthorized,
    Integration,
    Unsupported,
    Partial,
    Internal,
}

/// A failure envelope deliberately designed not to expose paths, credentials, guest content, raw
/// resource definitions, or backend error chains.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SafeFailure {
    pub code: SafeFailureCode,
    pub summary: String,
    pub operation_id: Option<String>,
    pub connection_id: Option<String>,
    pub target: Option<TargetIdentity>,
    pub outcome: MutationOutcome,
    pub retryable: bool,
    pub recovery_action: Option<RecoveryAction>,
}

impl SafeFailure {
    pub fn new(code: SafeFailureCode, summary: impl Into<String>) -> Self {
        Self {
            code,
            summary: summary.into(),
            operation_id: None,
            connection_id: None,
            target: None,
            outcome: MutationOutcome::Rejected,
            retryable: false,
            recovery_action: None,
        }
    }

    pub fn with_context(mut self, context: &OperationContext) -> Self {
        self.operation_id = Some(context.operation_id.clone());
        self.connection_id = Some(context.connection_id.clone());
        self.target = context.target.clone();
        self
    }
}

/// Application error types
#[derive(Error, Debug)]
pub enum AppError {
    #[error("The selected connection is unavailable.")]
    Unavailable(String),

    #[error("The requested desktop window is unavailable.")]
    WindowFailure(String),

    #[error("The host integration could not complete this operation.")]
    LibvirtError(String),

    #[error("The selected virtual machine is no longer available.")]
    VmNotFound(String),

    #[error("The selected virtual machine is not in a state that permits this operation.")]
    InvalidVmState(String),

    #[error("The selected network is no longer available.")]
    NetworkNotFound(String),

    #[error("The selected network is not in a state that permits this operation.")]
    InvalidNetworkState(String),

    #[error("The selected resource is no longer available.")]
    NotFound(String),

    #[error("libvirtd is not running")]
    LibvirtdNotRunning,

    #[error("Permission denied. Add user to libvirt group")]
    PermissionDenied,

    #[error("The supplied configuration is invalid.")]
    InvalidConfig(String),

    #[error("This operation is not available for the selected connection.")]
    Unsupported(String),

    #[error("The local system could not complete this operation.")]
    IoError(#[from] std::io::Error),

    #[error("The template operation could not be completed safely.")]
    TemplateError(String),

    #[error("The scheduled operation could not be completed safely.")]
    ScheduleError(String),

    #[error("The alert operation could not be completed safely.")]
    AlertError(String),

    #[error("The operation could not be completed safely.")]
    Other(String),

    #[error("The operation may have left resources that require inspection.")]
    Partial(String),
}

/// Map virt::error::Error to user-friendly AppError
pub fn map_libvirt_error(err: virt::error::Error) -> AppError {
    // Simply wrap the libvirt error message for now
    // We can add more sophisticated error mapping later
    AppError::LibvirtError(err.message().to_string())
}
/// Convert AppError to String for Tauri command results
impl From<AppError> for String {
    fn from(err: AppError) -> String {
        err.to_string()
    }
}

impl From<AppError> for SafeFailure {
    fn from(error: AppError) -> Self {
        match error {
            AppError::Unavailable(_) => SafeFailure::new(
                SafeFailureCode::Unavailable,
                "The selected connection is unavailable. Reconnect and retry.",
            ),
            AppError::WindowFailure(_) => SafeFailure::new(
                SafeFailureCode::Integration,
                "The requested desktop window could not be opened. Retry the operation.",
            ),
            AppError::PermissionDenied => SafeFailure::new(
                SafeFailureCode::Unauthorized,
                "The selected operation is not authorized on this host.",
            ),
            AppError::InvalidConfig(_) => SafeFailure::new(
                SafeFailureCode::InvalidInput,
                "The supplied configuration is invalid.",
            ),
            AppError::Unsupported(_) => SafeFailure::new(
                SafeFailureCode::Unsupported,
                "This operation is not available for the selected connection.",
            ),
            AppError::VmNotFound(_) | AppError::NetworkNotFound(_) | AppError::NotFound(_) => {
                SafeFailure::new(
                    SafeFailureCode::Conflict,
                    "The selected resource is no longer available.",
                )
            }
            AppError::LibvirtdNotRunning => SafeFailure::new(
                SafeFailureCode::Unavailable,
                "The required libvirt service is unavailable.",
            ),
            AppError::InvalidVmState(_) | AppError::InvalidNetworkState(_) => SafeFailure::new(
                SafeFailureCode::Conflict,
                "The selected resource is not in a state that permits this operation.",
            ),
            AppError::IoError(_) | AppError::LibvirtError(_) => SafeFailure::new(
                SafeFailureCode::Integration,
                "The host integration could not complete this operation.",
            ),
            AppError::TemplateError(_)
            | AppError::ScheduleError(_)
            | AppError::AlertError(_)
            | AppError::Other(_) => SafeFailure::new(
                SafeFailureCode::Internal,
                "The operation could not be completed safely.",
            ),
            AppError::Partial(_) => SafeFailure {
                code: SafeFailureCode::Partial,
                summary: "The operation did not complete cleanly. Inspect the affected resources before retrying."
                    .to_string(),
                operation_id: None,
                connection_id: None,
                target: None,
                outcome: MutationOutcome::Partial,
                retryable: false,
                recovery_action: Some(RecoveryAction {
                    kind: RecoveryActionKind::Inspect,
                    label: "Inspect affected resources".to_string(),
                    requires_confirmation: false,
                    expected_connection_id: None,
                }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_configuration_maps_to_a_safe_summary() {
        let failure = SafeFailure::from(AppError::InvalidConfig(
            "/private/path contains a password".to_string(),
        ));
        assert_eq!(failure.code, SafeFailureCode::InvalidInput);
        assert_eq!(failure.summary, "The supplied configuration is invalid.");
        assert!(!failure.summary.contains("/private/path"));
    }

    #[test]
    fn display_boundary_never_reflects_stored_host_or_secret_details() {
        let raw = "/private/host/path token=protected-value";
        for error in [
            AppError::LibvirtError(raw.to_string()),
            AppError::InvalidConfig(raw.to_string()),
            AppError::VmNotFound(raw.to_string()),
            AppError::InvalidVmState(raw.to_string()),
            AppError::NetworkNotFound(raw.to_string()),
            AppError::InvalidNetworkState(raw.to_string()),
            AppError::NotFound(raw.to_string()),
            AppError::TemplateError(raw.to_string()),
            AppError::ScheduleError(raw.to_string()),
            AppError::AlertError(raw.to_string()),
            AppError::Other(raw.to_string()),
            AppError::Partial(raw.to_string()),
        ] {
            assert!(!error.to_string().contains(raw));
        }
        let io_error = AppError::IoError(std::io::Error::other(raw));
        assert!(!io_error.to_string().contains(raw));
    }

    #[test]
    fn partial_state_requires_inspection_without_exposing_backend_detail() {
        let failure = SafeFailure::from(AppError::Partial("/host/path secret=value".to_string()));
        assert_eq!(failure.code, SafeFailureCode::Partial);
        assert_eq!(failure.outcome, MutationOutcome::Partial);
        assert_eq!(
            failure.recovery_action.expect("inspection action").kind,
            RecoveryActionKind::Inspect
        );
        assert!(!failure.summary.contains("/host/path"));
    }
}
