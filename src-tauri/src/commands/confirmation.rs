use crate::models::operation::{
    ConfirmationPreview, OperationContext, OperationKind, TargetIdentity,
};
use crate::state::app_state::AppState;
use crate::utils::error::AppError;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tauri::State;

/// Creates a short-lived preview for a destructive operation. The command-specific mutation must
/// consume the token with the same operation, connection, target and canonical effect details.
#[tauri::command]
pub async fn request_destructive_confirmation(
    state: State<'_, AppState>,
    operation: String,
    resource_kind: String,
    stable_id: String,
    display_name: Option<String>,
    effect: String,
) -> Result<ConfirmationPreview, String> {
    let context =
        destructive_context(&state, &operation, &resource_kind, &stable_id, display_name)?;
    let digest = effect_digest(&operation, &resource_kind, &stable_id, &effect);
    state
        .confirmations
        .issue(&context, &digest)
        .map_err(|error| error.to_string())
}

pub fn require_destructive_confirmation(
    state: &AppState,
    token: &str,
    operation: &str,
    resource_kind: &str,
    stable_id: &str,
    display_name: Option<String>,
    effect: &str,
) -> Result<(), AppError> {
    let context = destructive_context(state, operation, resource_kind, stable_id, display_name)?;
    let digest = effect_digest(operation, resource_kind, stable_id, effect);
    state
        .confirmations
        .verify_and_consume(token, &context, &digest)
}

fn destructive_context(
    state: &AppState,
    operation: &str,
    resource_kind: &str,
    stable_id: &str,
    display_name: Option<String>,
) -> Result<OperationContext, AppError> {
    if operation.is_empty() || resource_kind.is_empty() || stable_id.is_empty() {
        return Err(AppError::InvalidConfig(
            "Destructive confirmation requires an exact operation and target".to_string(),
        ));
    }
    state
        .resolve_operation(
            OperationKind::Mutation,
            Some(TargetIdentity {
                resource_kind: resource_kind.to_string(),
                stable_id: stable_id.to_string(),
                display_name,
            }),
        )
        .map(|resolved| resolved.context)
}

fn effect_digest(operation: &str, resource_kind: &str, stable_id: &str, effect: &str) -> String {
    let mut first = DefaultHasher::new();
    (operation, resource_kind, stable_id, effect).hash(&mut first);
    let mut second = DefaultHasher::new();
    (
        "kvm-manager-confirmation-v1",
        operation,
        resource_kind,
        stable_id,
        effect,
    )
        .hash(&mut second);
    format!("{:016x}{:016x}", first.finish(), second.finish())
}
