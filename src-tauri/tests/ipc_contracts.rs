use kvm_manager_app_lib::models::operation::{
    ConnectionScope, MutationOutcome, OperationContext, OperationKind, TargetIdentity,
};
use kvm_manager_app_lib::utils::error::{AppError, SafeFailure, SafeFailureCode};

#[test]
fn safe_failure_uses_camel_case_contract_without_source_error_details() {
    let context = OperationContext {
        operation_id: "op-contract".to_string(),
        operation_kind: OperationKind::Mutation,
        connection_id: "remote-a".to_string(),
        connection_label: "Lab host".to_string(),
        connection_scope: ConnectionScope::Remote,
        capabilities: Vec::new(),
        target: Some(TargetIdentity {
            resource_kind: "vm".to_string(),
            stable_id: "fixture-vm".to_string(),
            display_name: Some("Fixture VM".to_string()),
        }),
        captured_at: "2026-09-06T00:00:00Z".to_string(),
    };

    let failure = SafeFailure::from(AppError::InvalidConfig(
        "SENTINEL_PASSWORD_DO_NOT_LOG /private/path".to_string(),
    ))
    .with_context(&context);
    let value = serde_json::to_value(&failure).unwrap();

    assert_eq!(value["code"], "invalid_input");
    assert_eq!(value["operationId"], "op-contract");
    assert_eq!(value["connectionId"], "remote-a");
    assert_eq!(value["target"]["resourceKind"], "vm");
    assert!(!value.to_string().contains("SENTINEL_PASSWORD_DO_NOT_LOG"));
    assert_eq!(failure.outcome, MutationOutcome::Rejected);
    assert_eq!(failure.code, SafeFailureCode::InvalidInput);
}
