//! Guest Agent Service
//!
//! This service manages communication with guest agents running inside VMs.
//! It connects to the virtio-serial Unix sockets created by libvirt and
//! uses JSON-RPC 2.0 to communicate with the guest agent.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info};

/// Guest agent service state
pub struct GuestAgentService {
    /// Map of VM name -> agent connection
    connections: Arc<RwLock<HashMap<String, AgentConnection>>>,
    /// Base path for virtio-serial sockets
    socket_base_path: PathBuf,
}

/// Connection to a guest agent
struct AgentConnection {
    vm_name: String,
    socket_path: PathBuf,
    stream: Arc<Mutex<Option<UnixStream>>>,
    next_request_id: Arc<Mutex<i64>>,
}

// ===== JSON-RPC Protocol Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<serde_json::Value>,
    id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

// ===== Guest Info Types (matching agent protocol) =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub version: String,
    pub protocol_version: String,
    pub platform: String,
    pub capabilities: Vec<String>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub mac_address: String,
    pub ipv4_addresses: Vec<String>,
    pub ipv6_addresses: Vec<String>,
    pub state: String,
    pub mtu: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub interfaces: Vec<NetworkInterface>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsageInfo {
    pub filesystems: Vec<FilesystemInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecCommandResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
}

impl GuestAgentService {
    /// Create a new guest agent service
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            socket_base_path: PathBuf::from("/var/lib/libvirt/qemu/channel/target"),
        }
    }

    /// Check if a guest agent is available for a VM
    pub async fn is_agent_available(&self, vm_name: &str) -> bool {
        let socket_path = self.get_socket_path(vm_name);

        // Check if socket file exists
        if !socket_path.exists() {
            debug!("Guest agent socket not found for VM '{}': {:?}", vm_name, socket_path);
            return false;
        }

        // Try to connect and ping
        match self.ping(vm_name).await {
            Ok(_) => true,
            Err(e) => {
                debug!("Guest agent ping failed for VM '{}': {}", vm_name, e);
                false
            }
        }
    }

    /// Get the socket path for a VM
    fn get_socket_path(&self, vm_name: &str) -> PathBuf {
        self.socket_base_path
            .join(format!("{}.org.kvmmanager.agent.0", vm_name))
    }

    /// Get or create a connection to a guest agent
    async fn get_connection(&self, vm_name: &str) -> Result<AgentConnection> {
        let connections = self.connections.read().await;

        if let Some(conn) = connections.get(vm_name) {
            // Check if connection is still valid
            if conn.is_connected().await {
                return Ok(conn.clone());
            }
        }

        drop(connections);

        // Create new connection
        let socket_path = self.get_socket_path(vm_name);

        debug!("Connecting to guest agent for VM '{}' at {:?}", vm_name, socket_path);

        let stream = UnixStream::connect(&socket_path)
            .await
            .context("Failed to connect to guest agent socket")?;

        let conn = AgentConnection {
            vm_name: vm_name.to_string(),
            socket_path,
            stream: Arc::new(Mutex::new(Some(stream))),
            next_request_id: Arc::new(Mutex::new(1)),
        };

        // Store connection
        let mut connections = self.connections.write().await;
        connections.insert(vm_name.to_string(), conn.clone());

        info!("Connected to guest agent for VM '{}'", vm_name);

        Ok(conn)
    }

    /// Send a JSON-RPC request and wait for response
    async fn send_request(
        &self,
        vm_name: &str,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let conn = self.get_connection(vm_name).await?;

        // Get request ID
        let mut id_lock = conn.next_request_id.lock().await;
        let request_id = *id_lock;
        *id_lock += 1;
        drop(id_lock);

        // Build request
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: request_id,
        };

        let mut request_json = serde_json::to_string(&request)
            .context("Failed to serialize request")?;
        request_json.push('\n');

        debug!("Sending request to {}: {}", vm_name, request_json.trim());

        // Send request
        let mut stream_lock = conn.stream.lock().await;
        let stream = stream_lock.as_mut()
            .context("No active connection")?;

        stream.write_all(request_json.as_bytes())
            .await
            .context("Failed to write request")?;

        stream.flush()
            .await
            .context("Failed to flush stream")?;

        // Read response
        let mut reader = BufReader::new(stream);
        let mut response_line = String::new();

        reader.read_line(&mut response_line)
            .await
            .context("Failed to read response")?;

        if response_line.is_empty() {
            anyhow::bail!("Connection closed by guest agent");
        }

        debug!("Received response from {}: {}", vm_name, response_line.trim());

        let response: JsonRpcResponse = serde_json::from_str(&response_line)
            .context("Failed to parse response")?;

        // Check for errors
        if let Some(error) = response.error {
            anyhow::bail!("Guest agent error: {} (code {})", error.message, error.code);
        }

        // Return result
        response.result
            .context("Response missing result field")
    }

    /// Ping the guest agent
    pub async fn ping(&self, vm_name: &str) -> Result<bool> {
        let result = self.send_request(vm_name, "ping", None).await?;

        #[derive(Deserialize)]
        struct PingResult {
            pong: bool,
        }

        let ping_result: PingResult = serde_json::from_value(result)?;
        Ok(ping_result.pong)
    }

    /// Get agent information
    pub async fn get_agent_info(&self, vm_name: &str) -> Result<AgentInfo> {
        let result = self.send_request(vm_name, "get_agent_info", None).await?;
        let info: AgentInfo = serde_json::from_value(result)?;
        Ok(info)
    }

    /// Get system information from guest
    pub async fn get_system_info(&self, vm_name: &str) -> Result<SystemInfo> {
        let result = self.send_request(vm_name, "get_system_info", None).await?;
        let info: SystemInfo = serde_json::from_value(result)?;
        Ok(info)
    }

    /// Get network information from guest
    pub async fn get_network_info(&self, vm_name: &str) -> Result<NetworkInfo> {
        let result = self.send_request(vm_name, "get_network_info", None).await?;
        let info: NetworkInfo = serde_json::from_value(result)?;
        Ok(info)
    }

    /// Get disk usage from guest
    pub async fn get_disk_usage(&self, vm_name: &str) -> Result<DiskUsageInfo> {
        let result = self.send_request(vm_name, "get_disk_usage", None).await?;
        let info: DiskUsageInfo = serde_json::from_value(result)?;
        Ok(info)
    }

    /// Execute a command in the guest
    pub async fn exec_command(
        &self,
        vm_name: &str,
        command: &str,
        args: Vec<String>,
        timeout_seconds: u64,
    ) -> Result<ExecCommandResult> {
        let params = json!({
            "command": command,
            "args": args,
            "timeout_seconds": timeout_seconds,
            "capture_output": true,
        });

        let result = self.send_request(vm_name, "exec_command", Some(params)).await?;
        let exec_result: ExecCommandResult = serde_json::from_value(result)?;
        Ok(exec_result)
    }

    /// Read a file from the guest
    pub async fn read_file(&self, vm_name: &str, path: &str) -> Result<String> {
        let params = json!({
            "path": path,
            "encoding": "utf8",
        });

        let result = self.send_request(vm_name, "file_read", Some(params)).await?;

        #[derive(Deserialize)]
        struct FileReadResult {
            content: String,
        }

        let file_result: FileReadResult = serde_json::from_value(result)?;
        Ok(file_result.content)
    }

    /// Write a file to the guest
    pub async fn write_file(
        &self,
        vm_name: &str,
        path: &str,
        content: &str,
        create_dirs: bool,
    ) -> Result<()> {
        let params = json!({
            "path": path,
            "content": content,
            "encoding": "utf8",
            "create_dirs": create_dirs,
        });

        self.send_request(vm_name, "file_write", Some(params)).await?;
        Ok(())
    }

    /// Shutdown the guest
    pub async fn shutdown(&self, vm_name: &str, force: bool) -> Result<()> {
        let params = json!({
            "timeout_seconds": 60,
            "force": force,
        });

        self.send_request(vm_name, "shutdown", Some(params)).await?;
        Ok(())
    }

    /// Reboot the guest
    pub async fn reboot(&self, vm_name: &str, force: bool) -> Result<()> {
        let params = json!({
            "timeout_seconds": 60,
            "force": force,
        });

        self.send_request(vm_name, "reboot", Some(params)).await?;
        Ok(())
    }

    /// Close connection to a specific VM
    #[allow(dead_code)]
    pub async fn close_connection(&self, vm_name: &str) {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.remove(vm_name) {
            let mut stream_lock = conn.stream.lock().await;
            *stream_lock = None;
            info!("Closed guest agent connection for VM '{}'", vm_name);
        }
    }

    /// Close all connections
    #[allow(dead_code)]
    pub async fn close_all(&self) {
        let mut connections = self.connections.write().await;
        for (vm_name, conn) in connections.drain() {
            let mut stream_lock = conn.stream.lock().await;
            *stream_lock = None;
            info!("Closed guest agent connection for VM '{}'", vm_name);
        }
    }
}

impl AgentConnection {
    /// Check if connection is still active
    async fn is_connected(&self) -> bool {
        let stream_lock = self.stream.lock().await;
        stream_lock.is_some()
    }

    /// Clone the connection (shares the underlying stream)
    fn clone(&self) -> Self {
        Self {
            vm_name: self.vm_name.clone(),
            socket_path: self.socket_path.clone(),
            stream: Arc::clone(&self.stream),
            next_request_id: Arc::clone(&self.next_request_id),
        }
    }
}

impl Default for GuestAgentService {
    fn default() -> Self {
        Self::new()
    }
}
