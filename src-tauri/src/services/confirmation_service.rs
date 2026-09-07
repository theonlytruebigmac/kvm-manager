use crate::models::operation::{ConfirmationPreview, ConfirmationToken, OperationContext};
use crate::utils::error::AppError;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

const DEFAULT_TTL_SECONDS: i64 = 120;

/// Holds short-lived, process-local confirmation state. Tokens never encode a target, connection,
/// or effect; all of that state remains on the backend and is compared at command entry.
pub struct ConfirmationService {
    pending: Mutex<HashMap<String, PendingConfirmation>>,
}

impl Default for ConfirmationService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
struct PendingConfirmation {
    confirmation: ConfirmationToken,
}

impl ConfirmationService {
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
        }
    }

    pub fn issue(
        &self,
        context: &OperationContext,
        effect_digest: &str,
    ) -> Result<ConfirmationPreview, AppError> {
        self.issue_at(context, effect_digest, Utc::now(), DEFAULT_TTL_SECONDS)
    }

    pub fn issue_at(
        &self,
        context: &OperationContext,
        effect_digest: &str,
        issued_at: DateTime<Utc>,
        ttl_seconds: i64,
    ) -> Result<ConfirmationPreview, AppError> {
        let target = context.target.clone().ok_or_else(|| {
            AppError::InvalidConfig("Destructive confirmation requires an exact target".to_string())
        })?;
        validate_effect_digest(effect_digest)?;
        if ttl_seconds <= 0 {
            return Err(AppError::InvalidConfig(
                "Destructive confirmation lifetime is invalid".to_string(),
            ));
        }

        let token = Uuid::new_v4().to_string();
        let confirmation = ConfirmationToken {
            operation_kind: context.operation_kind.clone(),
            connection_id: context.connection_id.clone(),
            target,
            effect_digest: effect_digest.to_string(),
            expires_at: (issued_at + Duration::seconds(ttl_seconds)).to_rfc3339(),
        };
        let mut pending = self.pending.lock().map_err(|_| {
            AppError::Other("The confirmation store is temporarily unavailable".to_string())
        })?;
        pending.insert(
            token.clone(),
            PendingConfirmation {
                confirmation: confirmation.clone(),
            },
        );
        Ok(ConfirmationPreview {
            token,
            confirmation,
        })
    }

    pub fn verify_and_consume(
        &self,
        token: &str,
        context: &OperationContext,
        effect_digest: &str,
    ) -> Result<(), AppError> {
        self.verify_and_consume_at(token, context, effect_digest, Utc::now())
    }

    pub fn verify_and_consume_at(
        &self,
        token: &str,
        context: &OperationContext,
        effect_digest: &str,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        validate_effect_digest(effect_digest)?;
        let mut pending = self.pending.lock().map_err(|_| {
            AppError::Other("The confirmation store is temporarily unavailable".to_string())
        })?;
        let Some(pending_confirmation) = pending.remove(token) else {
            return Err(AppError::InvalidConfig(
                "The destructive confirmation is missing, expired, or already used".to_string(),
            ));
        };
        let confirmation = pending_confirmation.confirmation;
        let expires_at = DateTime::parse_from_rfc3339(&confirmation.expires_at)
            .map_err(|_| AppError::Other("Stored confirmation expiry is invalid".to_string()))?
            .with_timezone(&Utc);
        if now >= expires_at {
            return Err(AppError::InvalidConfig(
                "The destructive confirmation has expired".to_string(),
            ));
        }
        if confirmation.operation_kind != context.operation_kind
            || confirmation.connection_id != context.connection_id
            || context.target.as_ref() != Some(&confirmation.target)
            || confirmation.effect_digest != effect_digest
        {
            return Err(AppError::InvalidConfig(
                "The destructive target or selected connection changed".to_string(),
            ));
        }
        Ok(())
    }
}

fn validate_effect_digest(effect_digest: &str) -> Result<(), AppError> {
    if effect_digest.len() < 16
        || effect_digest.len() > 128
        || !effect_digest
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(AppError::InvalidConfig(
            "Destructive effect digest is invalid".to_string(),
        ));
    }
    Ok(())
}
