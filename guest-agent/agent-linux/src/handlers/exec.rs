//! Command execution handler

use kvmmanager_agent_common::*;
use serde_json::json;
use std::process::Stdio;
use std::time::Instant;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, warn};

use crate::config::Config;

pub async fn handle_exec_command(
    request: JsonRpcRequest,
    id: RequestId,
    config: &Config,
) -> JsonRpcResponse {
    debug!("Handling exec_command");

    // Parse parameters
    let params: ExecCommandParams = match request.params {
        Some(params) => match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse::error(
                    JsonRpcError::invalid_params(format!("Invalid parameters: {}", e)),
                    id,
                );
            }
        },
        None => {
            return JsonRpcResponse::error(
                JsonRpcError::invalid_params("Missing parameters"),
                id,
            );
        }
    };

    // Check if command is allowed
    if !config.is_command_allowed(&params.command) {
        warn!("Command not in whitelist: {}", params.command);
        return JsonRpcResponse::error(
            JsonRpcError::permission_denied(format!(
                "Command not allowed: {}",
                params.command
            )),
            id,
        );
    }

    // Enforce maximum timeout
    let timeout_secs = params
        .timeout_seconds
        .min(config.security.max_command_timeout_seconds);

    // Build command
    let mut cmd = Command::new(&params.command);
    cmd.args(&params.args);

    if let Some(wd) = &params.working_directory {
        cmd.current_dir(wd);
    }

    if params.capture_output {
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
    } else {
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
    }

    // Execute command with timeout
    let start = Instant::now();

    let result = timeout(
        std::time::Duration::from_secs(timeout_secs),
        cmd.output(),
    )
    .await;

    let execution_time = start.elapsed();

    match result {
        Ok(Ok(output)) => {
            let stdout = if params.capture_output {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                String::new()
            };

            let stderr = if params.capture_output {
                String::from_utf8_lossy(&output.stderr).to_string()
            } else {
                String::new()
            };

            let result = ExecCommandResult {
                exit_code: output.status.code().unwrap_or(-1),
                stdout,
                stderr,
                execution_time_ms: execution_time.as_millis() as u64,
            };

            JsonRpcResponse::success(json!(result), id)
        }
        Ok(Err(e)) => {
            warn!("Command execution failed: {}", e);
            JsonRpcResponse::error(
                JsonRpcError::command_failed(format!("Execution failed: {}", e)),
                id,
            )
        }
        Err(_) => {
            warn!("Command timed out after {} seconds", timeout_secs);
            JsonRpcResponse::error(
                JsonRpcError::timeout(format!(
                    "Command timed out after {} seconds",
                    timeout_secs
                )),
                id,
            )
        }
    }
}
