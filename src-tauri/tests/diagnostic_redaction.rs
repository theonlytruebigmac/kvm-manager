use kvm_manager_app_lib::models::operation::{
    ConnectionScope, MutationOutcome, OperationContext, OperationKind, TargetIdentity,
};
use kvm_manager_app_lib::utils::diagnostics::captured_terminal_operation_event;
use kvm_manager_app_lib::utils::error::{AppError, SafeFailure};

const PROTECTED_SENTINELS: &[&str] = &[
    "SENTINEL_PASSWORD_DO_NOT_LOG",
    "SENTINEL_ENCRYPTION_MATERIAL_DO_NOT_LOG",
    "SENTINEL_SSH_KEY_DO_NOT_LOG",
    "SENTINEL_GUEST_CONTENT_DO_NOT_LOG",
    "SENTINEL_CONSOLE_SECRET_DO_NOT_LOG",
    "<domain>SENTINEL_RAW_DEFINITION_DO_NOT_LOG</domain>",
    "/SENTINEL_HOST_PATH_DO_NOT_LOG",
    "qemu+ssh://SENTINEL_TRANSPORT_DETAIL_DO_NOT_LOG",
    "SENTINEL_SECRET_IDENTIFIER_DO_NOT_LOG",
];

fn context_containing_only_ignored_protected_values() -> OperationContext {
    OperationContext {
        operation_id: "op-redaction-1".to_string(),
        operation_kind: OperationKind::Mutation,
        connection_id: "connection-safe-id".to_string(),
        connection_label: PROTECTED_SENTINELS.join(" "),
        connection_scope: ConnectionScope::Remote,
        capabilities: Vec::new(),
        target: Some(TargetIdentity {
            resource_kind: "volume".to_string(),
            stable_id: "volume-safe-id".to_string(),
            display_name: Some(PROTECTED_SENTINELS.join(" ")),
        }),
        captured_at: PROTECTED_SENTINELS.join(" "),
    }
}

fn assert_safe_capture(outcome: MutationOutcome, reason_code: Option<&str>) {
    let captured = captured_terminal_operation_event(
        &context_containing_only_ignored_protected_values(),
        outcome.clone(),
        reason_code,
    );
    for sentinel in PROTECTED_SENTINELS {
        assert!(
            !captured.contains(sentinel),
            "protected sentinel reached diagnostic output"
        );
    }
    assert!(captured.contains("op-redaction-1"));
    assert!(captured.contains("connection-safe-id"));
    assert!(captured.contains("volume-safe-id"));
    assert!(captured.contains(&serde_json::to_string(&outcome).unwrap()));
}

#[test]
fn successful_operation_diagnostics_exclude_every_protected_category() {
    assert_safe_capture(MutationOutcome::Applied, None);
}

#[test]
fn failed_operation_diagnostics_exclude_every_protected_category_and_untrusted_reason() {
    assert_safe_capture(
        MutationOutcome::Rejected,
        Some("SENTINEL_PASSWORD_DO_NOT_LOG"),
    );
}

#[test]
fn public_failure_envelopes_exclude_every_protected_category() {
    let protected = PROTECTED_SENTINELS.join(" ");
    for error in [
        AppError::LibvirtError(protected.clone()),
        AppError::InvalidConfig(protected.clone()),
        AppError::Other(protected.clone()),
        AppError::IoError(std::io::Error::other(protected.clone())),
    ] {
        let failure = SafeFailure::from(error);
        let serialized = serde_json::to_string(&failure).unwrap();
        for sentinel in PROTECTED_SENTINELS {
            assert!(
                !serialized.contains(sentinel),
                "protected sentinel reached the public failure envelope"
            );
        }
    }
}
