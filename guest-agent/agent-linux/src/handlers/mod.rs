//! Request handlers

use kvmmanager_agent_common::*;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, warn};

use crate::config::Config;

mod system;
mod exec;
mod files;
mod power;

/// Handle incoming JSON-RPC request and return response
pub async fn handle_request(request: JsonRpcRequest, config: &Config) -> JsonRpcResponse {
    let id = request.id.clone();

    match request.method.as_str() {
        "ping" => handle_ping(id),
        "get_agent_info" => handle_get_agent_info(id),
        "get_system_info" => system::handle_get_system_info(id).await,
        "get_network_info" => system::handle_get_network_info(id).await,
        "get_disk_usage" => system::handle_get_disk_usage(id).await,
        "exec_command" => exec::handle_exec_command(request, id, config).await,
        "file_read" => files::handle_file_read(request, id, config).await,
        "file_write" => files::handle_file_write(request, id, config).await,
        "shutdown" => power::handle_shutdown(request, id).await,
        "reboot" => power::handle_reboot(request, id).await,
        _ => {
            warn!("Unknown method: {}", request.method);
            JsonRpcResponse::error(
                JsonRpcError::method_not_found(&request.method),
                id,
            )
        }
    }
}

fn handle_ping(id: RequestId) -> JsonRpcResponse {
    debug!("Handling ping");

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let result = PingResult {
        pong: true,
        timestamp,
    };

    JsonRpcResponse::success(json!(result), id)
}

fn handle_get_agent_info(id: RequestId) -> JsonRpcResponse {
    debug!("Handling get_agent_info");

    let info = AgentInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        protocol_version: "1.0.0".to_string(),
        platform: "linux".to_string(),
        capabilities: vec![
            "ping".to_string(),
            "get_agent_info".to_string(),
            "get_system_info".to_string(),
            "get_network_info".to_string(),
            "get_disk_usage".to_string(),
            "exec_command".to_string(),
            "file_read".to_string(),
            "file_write".to_string(),
            "shutdown".to_string(),
            "reboot".to_string(),
        ],
    };

    JsonRpcResponse::success(json!(info), id)
}
