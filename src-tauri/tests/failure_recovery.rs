use kvm_manager_app_lib::models::operation::OperationKind;
use kvm_manager_app_lib::services::connection_service::ConnectionService;
use kvm_manager_app_lib::utils::error::{AppError, SafeFailure, SafeFailureCode};

#[test]
fn unavailable_connection_returns_a_safe_classification() {
    let service = ConnectionService::new();
    let error = match service.resolve_operation(OperationKind::Query, None) {
        Ok(_) => panic!("an inactive connection must not resolve"),
        Err(error) => error,
    };
    let failure = SafeFailure::from(error);

    assert_eq!(failure.code, SafeFailureCode::Unavailable);
    assert_eq!(
        failure.summary,
        "The selected connection is unavailable. Reconnect and retry."
    );
    assert!(!failure.retryable);
}

#[test]
fn malformed_and_non_utf8_host_responses_do_not_leak_or_panic() {
    let malformed = serde_json::from_str::<serde_json::Value>("{not-json").unwrap_err();
    let invalid_utf8 = String::from_utf8(vec![0xff, 0xfe]).unwrap_err();

    for error in [
        AppError::Other(malformed.to_string()),
        AppError::Other(invalid_utf8.to_string()),
        AppError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "missing-command",
        )),
    ] {
        let failure = SafeFailure::from(error);
        assert!(!failure.summary.contains("not-json"));
        assert!(!failure.summary.contains("missing-command"));
    }
}

#[test]
fn failed_window_creation_returns_a_safe_failure() {
    let failure = SafeFailure::from(AppError::WindowFailure(
        "window backend rejected a protected host path".to_string(),
    ));

    assert_eq!(failure.code, SafeFailureCode::Integration);
    assert_eq!(
        failure.summary,
        "The requested desktop window could not be opened. Retry the operation."
    );
    assert!(!failure.summary.contains("protected host path"));
}
