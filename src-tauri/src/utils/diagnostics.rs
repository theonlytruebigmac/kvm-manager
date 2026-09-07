use crate::models::operation::{MutationOutcome, OperationContext};
use serde::Serialize;

const DEFAULT_REASON_CODE: &str = "unspecified";
const ALLOWED_REASON_CODES: &[&str] = &[
    "unavailable",
    "invalid_input",
    "conflict",
    "unauthorized",
    "integration_failure",
    "unsupported",
    "partial",
    "unknown",
    "reconciliation_required",
];

/// The complete, allowlisted payload for one terminal privileged-operation event.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct OperationDiagnosticEvent {
    pub event: &'static str,
    pub operation_id: String,
    pub operation_kind: String,
    pub connection_id: String,
    pub resource_kind: String,
    pub resource_id: String,
    pub outcome: MutationOutcome,
    pub reason_code: String,
}

/// Builds the only diagnostic payload that terminal privileged operations may emit. It
/// deliberately omits labels, capabilities, timestamps, raw errors, and every user-supplied
/// configuration value that could contain a protected value.
pub fn terminal_operation_event(
    context: &OperationContext,
    outcome: MutationOutcome,
    reason_code: Option<&str>,
) -> OperationDiagnosticEvent {
    OperationDiagnosticEvent {
        event: "privileged_operation_finished",
        operation_id: safe_identifier(&context.operation_id),
        operation_kind: format!("{:?}", context.operation_kind).to_ascii_lowercase(),
        connection_id: safe_identifier(&context.connection_id),
        resource_kind: context
            .target
            .as_ref()
            .map(|target| safe_identifier(&target.resource_kind))
            .unwrap_or_else(|| "none".to_string()),
        resource_id: context
            .target
            .as_ref()
            .map(|target| safe_identifier(&target.stable_id))
            .unwrap_or_else(|| "none".to_string()),
        outcome,
        reason_code: normalized_reason_code(reason_code),
    }
}

/// Returns the exact allowlisted representation used by captured-output tests and support-safe
/// diagnostic sinks. This is intentionally separate from arbitrary tracing formatting.
pub fn captured_terminal_operation_event(
    context: &OperationContext,
    outcome: MutationOutcome,
    reason_code: Option<&str>,
) -> String {
    serde_json::to_string(&terminal_operation_event(context, outcome, reason_code))
        .unwrap_or_else(|_| "{\"event\":\"diagnostic_serialization_failed\"}".to_string())
}

/// Emits only the reviewed, non-sensitive fields permitted for privileged operations.
pub fn operation_finished(
    context: &OperationContext,
    outcome: MutationOutcome,
    reason_code: Option<&str>,
) {
    let event = terminal_operation_event(context, outcome, reason_code);
    tracing::info!(
        event = event.event,
        operation_id = %event.operation_id,
        operation_kind = %event.operation_kind,
        connection_id = %event.connection_id,
        resource_kind = %event.resource_kind,
        resource_id = %event.resource_id,
        outcome = ?event.outcome,
        reason_code = %event.reason_code,
        "Privileged operation finished"
    );
}

fn normalized_reason_code(value: Option<&str>) -> String {
    value
        .filter(|value| ALLOWED_REASON_CODES.contains(value))
        .unwrap_or(DEFAULT_REASON_CODE)
        .to_string()
}

fn safe_identifier(value: &str) -> String {
    if !value.is_empty()
        && value.len() <= 128
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | ':')
        })
    {
        value.to_string()
    } else {
        "redacted".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::operation::{ConnectionScope, OperationKind};

    #[test]
    fn accepts_only_safe_operation_context_fields() {
        let context = OperationContext {
            operation_id: "op-1".to_string(),
            operation_kind: OperationKind::Query,
            connection_id: "local".to_string(),
            connection_label: "Local".to_string(),
            connection_scope: ConnectionScope::LocalSystem,
            capabilities: Vec::new(),
            target: None,
            captured_at: "2026-09-06T00:00:00Z".to_string(),
        };
        operation_finished(&context, MutationOutcome::Rejected, Some("unavailable"));
    }

    #[test]
    fn drops_unrecognized_reason_codes_and_unsafe_identifiers() {
        let context = OperationContext {
            operation_id: "op unsafe value".to_string(),
            operation_kind: OperationKind::Mutation,
            connection_id: "local".to_string(),
            connection_label: "Local".to_string(),
            connection_scope: ConnectionScope::LocalSystem,
            capabilities: Vec::new(),
            target: None,
            captured_at: "2026-09-06T00:00:00Z".to_string(),
        };
        let event =
            terminal_operation_event(&context, MutationOutcome::Rejected, Some("raw secret"));
        assert_eq!(event.operation_id, "redacted");
        assert_eq!(event.reason_code, DEFAULT_REASON_CODE);
    }
}
