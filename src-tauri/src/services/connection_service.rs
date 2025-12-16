use std::collections::HashMap;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};
use virt::connect::Connect;
use crate::utils::error::AppError;

/// Connection type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionType {
    Local,
    Ssh,
    Tls,
    Tcp,
}

impl Default for ConnectionType {
    fn default() -> Self {
        Self::Local
    }
}

/// Saved connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedConnection {
    /// Unique ID for this connection
    pub id: String,
    /// Display name
    pub name: String,
    /// Connection type
    pub connection_type: ConnectionType,
    /// Hypervisor type (qemu, xen, lxc, etc.)
    #[serde(default = "default_hypervisor")]
    pub hypervisor: String,
    /// Remote host (for SSH/TLS/TCP)
    pub host: Option<String>,
    /// SSH username (for SSH connections)
    pub username: Option<String>,
    /// SSH port (default 22)
    pub ssh_port: Option<u16>,
    /// TLS port (default 16514)
    pub tls_port: Option<u16>,
    /// Whether to auto-connect on startup
    #[serde(default)]
    pub auto_connect: bool,
    /// Connection path (system, session)
    #[serde(default = "default_path")]
    pub path: String,
}

fn default_hypervisor() -> String {
    "qemu".to_string()
}

fn default_path() -> String {
    "system".to_string()
}

impl SavedConnection {
    /// Create a new local connection
    pub fn local() -> Self {
        Self {
            id: "local".to_string(),
            name: "QEMU/KVM (Local)".to_string(),
            connection_type: ConnectionType::Local,
            hypervisor: "qemu".to_string(),
            host: None,
            username: None,
            ssh_port: None,
            tls_port: None,
            auto_connect: true,
            path: "system".to_string(),
        }
    }

    /// Build the libvirt URI for this connection
    pub fn build_uri(&self) -> String {
        match self.connection_type {
            ConnectionType::Local => {
                format!("{}:///{}", self.hypervisor, self.path)
            }
            ConnectionType::Ssh => {
                let user = self.username.as_deref().unwrap_or("root");
                let host = self.host.as_deref().unwrap_or("localhost");
                let port = self.ssh_port.unwrap_or(22);
                if port == 22 {
                    format!("{}+ssh://{}@{}/{}", self.hypervisor, user, host, self.path)
                } else {
                    format!("{}+ssh://{}@{}:{}/{}", self.hypervisor, user, host, port, self.path)
                }
            }
            ConnectionType::Tls => {
                let host = self.host.as_deref().unwrap_or("localhost");
                let port = self.tls_port.unwrap_or(16514);
                if port == 16514 {
                    format!("{}+tls://{}/{}", self.hypervisor, host, self.path)
                } else {
                    format!("{}+tls://{}:{}/{}", self.hypervisor, host, port, self.path)
                }
            }
            ConnectionType::Tcp => {
                let host = self.host.as_deref().unwrap_or("localhost");
                format!("{}+tcp://{}/{}", self.hypervisor, host, self.path)
            }
        }
    }
}

/// Active connection wrapper
#[allow(dead_code)]
struct ActiveConnection {
    config: SavedConnection,
    connection: Connect,
}

/// Connection service manages multiple libvirt connections
pub struct ConnectionService {
    /// Currently active connection ID
    active_connection_id: RwLock<Option<String>>,
    /// Active connections (only one for now, but extensible)
    connections: RwLock<HashMap<String, ActiveConnection>>,
    /// Saved connection configurations
    saved_connections: RwLock<Vec<SavedConnection>>,
}

impl ConnectionService {
    /// Create a new connection service
    pub fn new() -> Self {
        let mut saved = Vec::new();
        // Always include local connection
        saved.push(SavedConnection::local());

        Self {
            active_connection_id: RwLock::new(None),
            connections: RwLock::new(HashMap::new()),
            saved_connections: RwLock::new(saved),
        }
    }

    /// Connect to a saved connection by ID
    pub fn connect(&self, connection_id: &str) -> Result<(), AppError> {
        // Find the saved connection
        let saved = {
            let saved_conns = self.saved_connections.read()
                .map_err(|_| AppError::Other("Failed to lock saved connections".to_string()))?;
            saved_conns.iter().find(|c| c.id == connection_id).cloned()
        };

        let config = saved.ok_or_else(|| {
            AppError::Other(format!("Connection '{}' not found", connection_id))
        })?;

        let uri = config.build_uri();
        tracing::info!("Connecting to libvirt: {}", uri);

        // Attempt connection
        let connection = Connect::open(Some(&uri))
            .map_err(|e| AppError::LibvirtError(format!("Failed to connect to {}: {}", uri, e)))?;

        tracing::info!("Successfully connected to {}", uri);

        // Store active connection
        {
            let mut conns = self.connections.write()
                .map_err(|_| AppError::Other("Failed to lock connections".to_string()))?;
            conns.insert(connection_id.to_string(), ActiveConnection {
                config: config.clone(),
                connection,
            });
        }

        // Set as active
        {
            let mut active = self.active_connection_id.write()
                .map_err(|_| AppError::Other("Failed to lock active connection".to_string()))?;
            *active = Some(connection_id.to_string());
        }

        Ok(())
    }

    /// Disconnect from a connection
    pub fn disconnect(&self, connection_id: &str) -> Result<(), AppError> {
        let mut conns = self.connections.write()
            .map_err(|_| AppError::Other("Failed to lock connections".to_string()))?;

        if let Some(_conn) = conns.remove(connection_id) {
            tracing::info!("Disconnected from {}", connection_id);
        }

        // If this was the active connection, clear it
        let mut active = self.active_connection_id.write()
            .map_err(|_| AppError::Other("Failed to lock active connection".to_string()))?;
        if active.as_deref() == Some(connection_id) {
            *active = None;
        }

        Ok(())
    }

    /// Get the active libvirt connection
    #[allow(dead_code)]
    pub fn get_active_connection(&self) -> Result<Connect, AppError> {
        let active_id = {
            let active = self.active_connection_id.read()
                .map_err(|_| AppError::Other("Failed to lock active connection".to_string()))?;
            active.clone()
        };

        let connection_id = active_id.ok_or_else(|| {
            AppError::Other("No active connection".to_string())
        })?;

        let conns = self.connections.read()
            .map_err(|_| AppError::Other("Failed to lock connections".to_string()))?;

        let conn = conns.get(&connection_id).ok_or_else(|| {
            AppError::Other("Active connection not found".to_string())
        })?;

        // Clone the connection for use
        // Note: libvirt connections can be shared via cloning
        Ok(conn.connection.clone())
    }

    /// Get the currently active connection info
    pub fn get_active_connection_info(&self) -> Result<Option<SavedConnection>, AppError> {
        let active_id = {
            let active = self.active_connection_id.read()
                .map_err(|_| AppError::Other("Failed to lock active connection".to_string()))?;
            active.clone()
        };

        match active_id {
            Some(id) => {
                let conns = self.connections.read()
                    .map_err(|_| AppError::Other("Failed to lock connections".to_string()))?;
                Ok(conns.get(&id).map(|c| c.config.clone()))
            }
            None => Ok(None),
        }
    }

    /// Check if actively connected
    #[allow(dead_code)]
    pub fn is_connected(&self) -> bool {
        let active_id = self.active_connection_id.read().ok()
            .and_then(|a| a.clone());

        if let Some(id) = active_id {
            if let Ok(conns) = self.connections.read() {
                if let Some(conn) = conns.get(&id) {
                    return conn.connection.is_alive().unwrap_or(false);
                }
            }
        }
        false
    }

    /// Get all saved connections
    pub fn get_saved_connections(&self) -> Result<Vec<SavedConnection>, AppError> {
        let saved = self.saved_connections.read()
            .map_err(|_| AppError::Other("Failed to lock saved connections".to_string()))?;
        Ok(saved.clone())
    }

    /// Add a new saved connection
    pub fn add_connection(&self, config: SavedConnection) -> Result<(), AppError> {
        let mut saved = self.saved_connections.write()
            .map_err(|_| AppError::Other("Failed to lock saved connections".to_string()))?;

        // Check for duplicate ID
        if saved.iter().any(|c| c.id == config.id) {
            return Err(AppError::Other(format!("Connection '{}' already exists", config.id)));
        }

        saved.push(config);
        Ok(())
    }

    /// Update an existing connection
    pub fn update_connection(&self, config: SavedConnection) -> Result<(), AppError> {
        let mut saved = self.saved_connections.write()
            .map_err(|_| AppError::Other("Failed to lock saved connections".to_string()))?;

        if let Some(existing) = saved.iter_mut().find(|c| c.id == config.id) {
            *existing = config;
            Ok(())
        } else {
            Err(AppError::Other(format!("Connection '{}' not found", config.id)))
        }
    }

    /// Remove a saved connection
    pub fn remove_connection(&self, connection_id: &str) -> Result<(), AppError> {
        // Cannot remove local connection
        if connection_id == "local" {
            return Err(AppError::Other("Cannot remove local connection".to_string()));
        }

        // Disconnect first if connected
        let _ = self.disconnect(connection_id);

        let mut saved = self.saved_connections.write()
            .map_err(|_| AppError::Other("Failed to lock saved connections".to_string()))?;

        saved.retain(|c| c.id != connection_id);
        Ok(())
    }

    /// Get hostname of the active connection
    #[allow(dead_code)]
    pub fn get_hostname(&self) -> Result<String, AppError> {
        let conn = self.get_active_connection()?;
        conn.get_hostname()
            .map_err(|e| AppError::LibvirtError(format!("Failed to get hostname: {}", e)))
    }

    /// Get libvirt version
    #[allow(dead_code)]
    pub fn get_version(&self) -> Result<String, AppError> {
        let conn = self.get_active_connection()?;
        let version = conn.get_lib_version()
            .map_err(|e| AppError::LibvirtError(format!("Failed to get version: {}", e)))?;

        let major = version / 1000000;
        let minor = (version % 1000000) / 1000;
        let release = version % 1000;

        Ok(format!("{}.{}.{}", major, minor, release))
    }
}
