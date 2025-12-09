//! JSON-RPC 2.0 Protocol Types

use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    pub id: RequestId,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: RequestId,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Request ID (can be string, number, or null)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
    Null,
}

impl JsonRpcRequest {
    pub fn new(method: impl Into<String>, params: Option<serde_json::Value>, id: RequestId) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
            id,
        }
    }
}

impl JsonRpcResponse {
    pub fn success(result: serde_json::Value, id: RequestId) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(error: JsonRpcError, id: RequestId) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

impl JsonRpcError {
    // Standard JSON-RPC 2.0 errors
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    // Custom application errors
    pub const COMMAND_FAILED: i32 = -32000;
    pub const PERMISSION_DENIED: i32 = -32001;
    pub const NOT_FOUND: i32 = -32002;
    pub const TIMEOUT: i32 = -32003;

    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    pub fn with_data(code: i32, message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }

    pub fn parse_error() -> Self {
        Self::new(Self::PARSE_ERROR, "Parse error")
    }

    pub fn invalid_request(msg: impl Into<String>) -> Self {
        Self::new(Self::INVALID_REQUEST, msg)
    }

    pub fn method_not_found(method: &str) -> Self {
        Self::new(Self::METHOD_NOT_FOUND, format!("Method not found: {}", method))
    }

    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self::new(Self::INVALID_PARAMS, msg)
    }

    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self::new(Self::INTERNAL_ERROR, msg)
    }

    pub fn command_failed(msg: impl Into<String>) -> Self {
        Self::new(Self::COMMAND_FAILED, msg)
    }

    pub fn permission_denied(msg: impl Into<String>) -> Self {
        Self::new(Self::PERMISSION_DENIED, msg)
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::new(Self::NOT_FOUND, msg)
    }

    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::new(Self::TIMEOUT, msg)
    }
}

// ===== Method-specific types =====

/// Parameters for exec_command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecCommandParams {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    #[serde(default = "default_capture_output")]
    pub capture_output: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
}

fn default_timeout() -> u64 {
    30
}

fn default_capture_output() -> bool {
    true
}

/// Result from exec_command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecCommandResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
}

/// Result from get_agent_info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub version: String,
    pub protocol_version: String,
    pub platform: String,
    pub capabilities: Vec<String>,
}

/// Result from get_system_info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_type: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub hostname: String,
    pub architecture: String,
    pub cpu_count: usize,
    pub total_memory_kb: u64,
    pub uptime_seconds: u64,
}

/// Network interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub mac_address: String,
    pub ipv4_addresses: Vec<String>,
    pub ipv6_addresses: Vec<String>,
    pub state: String,
    pub mtu: u32,
}

/// Result from get_network_info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub interfaces: Vec<NetworkInterface>,
}

/// Parameters for file_read
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReadParams {
    pub path: String,
    #[serde(default = "default_encoding")]
    pub encoding: String, // "utf8" or "base64"
}

fn default_encoding() -> String {
    "utf8".to_string()
}

/// Result from file_read
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReadResult {
    pub content: String,
    pub size_bytes: u64,
    pub encoding: String,
}

/// Parameters for file_write
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWriteParams {
    pub path: String,
    pub content: String,
    #[serde(default = "default_encoding")]
    pub encoding: String, // "utf8" or "base64"
    #[serde(default)]
    pub create_dirs: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<String>, // e.g., "0644"
}

/// Result from file_write
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWriteResult {
    pub bytes_written: u64,
    pub path: String,
}

/// Parameters for shutdown/reboot
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PowerParams {
    #[serde(default = "default_power_timeout")]
    pub timeout_seconds: u64,
    #[serde(default)]
    pub force: bool,
}

fn default_power_timeout() -> u64 {
    60
}

impl PowerParams {
    pub fn new() -> Self {
        Self {
            timeout_seconds: 60,
            force: false,
        }
    }
}

/// Result from shutdown/reboot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerResult {
    pub initiated: bool,
    pub message: String,
}

/// Filesystem information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemInfo {
    pub mount_point: String,
    pub device: String,
    pub fs_type: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub used_percent: f64,
}

/// Result from get_disk_usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsageInfo {
    pub filesystems: Vec<FilesystemInfo>,
}

/// Result from ping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResult {
    pub pong: bool,
    pub timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req = JsonRpcRequest::new(
            "ping",
            None,
            RequestId::Number(1),
        );

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"ping\""));
        assert!(json.contains("\"id\":1"));
    }

    #[test]
    fn test_success_response() {
        let res = JsonRpcResponse::success(
            serde_json::json!({"pong": true}),
            RequestId::Number(1),
        );

        let json = serde_json::to_string(&res).unwrap();
        assert!(json.contains("\"result\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_error_response() {
        let res = JsonRpcResponse::error(
            JsonRpcError::method_not_found("test"),
            RequestId::Number(1),
        );

        let json = serde_json::to_string(&res).unwrap();
        assert!(json.contains("\"error\""));
        assert!(!json.contains("\"result\""));
    }
}
