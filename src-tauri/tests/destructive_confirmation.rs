use chrono::{Duration, TimeZone, Utc};
use kvm_manager_app_lib::models::operation::{
    ConnectionScope, OperationContext, OperationKind, TargetIdentity,
};
use kvm_manager_app_lib::services::confirmation_service::ConfirmationService;

const EFFECT: &str = "0123456789abcdef0123456789abcdef";

fn context(connection_id: &str, target_id: &str) -> OperationContext {
    OperationContext {
        operation_id: "op-confirm-1".to_string(),
        operation_kind: OperationKind::Mutation,
        connection_id: connection_id.to_string(),
        connection_label: "Safe test connection".to_string(),
        connection_scope: ConnectionScope::Test,
        capabilities: Vec::new(),
        target: Some(TargetIdentity {
            resource_kind: "vm".to_string(),
            stable_id: target_id.to_string(),
            display_name: Some("safe-test-vm".to_string()),
        }),
        captured_at: "2026-09-06T00:00:00Z".to_string(),
    }
}

#[test]
fn confirmation_rejects_a_stale_connection_selection() {
    let service = ConfirmationService::new();
    let issued = service
        .issue_at(
            &context("fixture-a", "vm-1"),
            EFFECT,
            Utc.with_ymd_and_hms(2026, 9, 6, 0, 0, 0).unwrap(),
            60,
        )
        .unwrap();
    assert!(service
        .verify_and_consume_at(
            &issued.token,
            &context("fixture-b", "vm-1"),
            EFFECT,
            Utc.with_ymd_and_hms(2026, 9, 6, 0, 0, 1).unwrap(),
        )
        .is_err());
}

#[test]
fn confirmation_rejects_a_changed_target() {
    let service = ConfirmationService::new();
    let issued = service
        .issue(&context("fixture-a", "vm-1"), EFFECT)
        .unwrap();
    assert!(service
        .verify_and_consume(&issued.token, &context("fixture-a", "vm-2"), EFFECT)
        .is_err());
}

#[test]
fn confirmation_expires_and_cannot_be_reused() {
    let service = ConfirmationService::new();
    let issued_at = Utc.with_ymd_and_hms(2026, 9, 6, 0, 0, 0).unwrap();
    let issued = service
        .issue_at(&context("fixture-a", "vm-1"), EFFECT, issued_at, 1)
        .unwrap();
    assert!(service
        .verify_and_consume_at(
            &issued.token,
            &context("fixture-a", "vm-1"),
            EFFECT,
            issued_at + Duration::seconds(1),
        )
        .is_err());

    let valid = service
        .issue_at(&context("fixture-a", "vm-1"), EFFECT, issued_at, 60)
        .unwrap();
    assert!(service
        .verify_and_consume_at(
            &valid.token,
            &context("fixture-a", "vm-1"),
            EFFECT,
            issued_at
        )
        .is_ok());
    assert!(service
        .verify_and_consume_at(
            &valid.token,
            &context("fixture-a", "vm-1"),
            EFFECT,
            issued_at
        )
        .is_err());
}
