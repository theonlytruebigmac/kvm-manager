//! Power management handlers (shutdown/reboot)

use kvmmanager_agent_common::*;
use serde_json::json;
use tokio::process::Command;
use tracing::{debug, warn};

pub async fn handle_shutdown(request: JsonRpcRequest, id: RequestId) -> JsonRpcResponse {
    debug!("Handling shutdown");

    // Parse parameters
    let params: PowerParams = match request.params {
        Some(params) => serde_json::from_value(params).unwrap_or_default(),
        None => PowerParams {
            timeout_seconds: 60,
            force: false,
        },
    };

    // Execute shutdown command
    let args: Vec<String> = if params.force {
        vec!["now".to_string()]
    } else {
        vec![format!("+{}", params.timeout_seconds / 60)]
    };

    let result = Command::new("shutdown")
        .args(&args)
        .output()
        .await;

    match result {
        Ok(output) => {
            if output.status.success() {
                let result = PowerResult {
                    initiated: true,
                    message: "Shutdown initiated".to_string(),
                };
                JsonRpcResponse::success(json!(result), id)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("Shutdown command failed: {}", stderr);
                JsonRpcResponse::error(
                    JsonRpcError::command_failed(format!("Shutdown failed: {}", stderr)),
                    id,
                )
            }
        }
        Err(e) => {
            warn!("Failed to execute shutdown: {}", e);
            JsonRpcResponse::error(
                JsonRpcError::command_failed(format!("Failed to execute shutdown: {}", e)),
                id,
            )
        }
    }
}

pub async fn handle_reboot(request: JsonRpcRequest, id: RequestId) -> JsonRpcResponse {
    debug!("Handling reboot");

    // Parse parameters
    let params: PowerParams = match request.params {
        Some(params) => serde_json::from_value(params).unwrap_or_default(),
        None => PowerParams {
            timeout_seconds: 60,
            force: false,
        },
    };

    // Execute reboot command
    let args: Vec<String> = if params.force {
        vec!["now".to_string()]
    } else {
        vec![format!("+{}", params.timeout_seconds / 60)]
    };

    let result = Command::new("reboot")
        .args(&args)
        .output()
        .await;

    match result {
        Ok(output) => {
            if output.status.success() {
                let result = PowerResult {
                    initiated: true,
                    message: "Reboot initiated".to_string(),
                };
                JsonRpcResponse::success(json!(result), id)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("Reboot command failed: {}", stderr);
                JsonRpcResponse::error(
                    JsonRpcError::command_failed(format!("Reboot failed: {}", stderr)),
                    id,
                )
            }
        }
        Err(e) => {
            warn!("Failed to execute reboot: {}", e);
            JsonRpcResponse::error(
                JsonRpcError::command_failed(format!("Failed to execute reboot: {}", e)),
                id,
            )
        }
    }
}
