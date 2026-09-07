use crate::models::operation::{
    CapabilityState, ConnectionCapability, ConnectionScope, OperationContext, OperationKind,
    TargetIdentity,
};
use crate::utils::error::AppError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;
use virt::connect::{Connect, ConnectAuth, ConnectCredential};
use virt::sys;

#[derive(Clone)]
struct EphemeralSshCredentials {
    username: String,
    password: String,
}

thread_local! {
    /// `virt` exposes a function-pointer-only callback. Keep credentials on the connecting
    /// thread for the duration of one libvirt call; they are never stored in SavedConnection.
    static EPHEMERAL_SSH_CREDENTIALS: RefCell<Option<EphemeralSshCredentials>> = const { RefCell::new(None) };
}

fn provide_ssh_credentials(credentials: &mut Vec<ConnectCredential>) {
    EPHEMERAL_SSH_CREDENTIALS.with(|stored| {
        let stored = stored.borrow();
        let Some(auth) = stored.as_ref() else {
            return;
        };
        for credential in credentials {
            credential.result = match credential.typed {
                value if value == sys::VIR_CRED_AUTHNAME as i32 => Some(auth.username.clone()),
                value
                    if value == sys::VIR_CRED_PASSPHRASE as i32
                        || value == sys::VIR_CRED_NOECHOPROMPT as i32
                        || value == sys::VIR_CRED_ECHOPROMPT as i32 =>
                {
                    Some(auth.password.clone())
                }
                _ => None,
            };
        }
    });
}

/// Connection type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ConnectionType {
    #[default]
    Local,
    Ssh,
    Tls,
    Tcp,
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
            ConnectionType::Ssh => self.ssh_uri("ssh", "no_tty=1"),
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

    /// Use libvirt's in-process libssh2 transport when the user explicitly supplies a password.
    /// The ordinary `+ssh` transport launches the system ssh binary, which has no GUI credential
    /// callback and would otherwise print a terminal prompt into the application logs.
    fn password_auth_uri(&self) -> String {
        self.ssh_uri("libssh2", "sshauth=password,keyboard-interactive")
    }

    fn ssh_uri(&self, transport: &str, query: &str) -> String {
        let user = self.username.as_deref().unwrap_or("root");
        let host = self.host.as_deref().unwrap_or("localhost");
        let port = self.ssh_port.unwrap_or(22);
        if port == 22 {
            format!(
                "{}+{}://{}@{}/{}?{}",
                self.hypervisor, transport, user, host, self.path, query
            )
        } else {
            format!(
                "{}+{}://{}@{}:{}/{}?{}",
                self.hypervisor, transport, user, host, port, self.path, query
            )
        }
    }
}

/// Active connection wrapper
#[allow(dead_code)]
struct ActiveConnection {
    config: SavedConnection,
    connection: Connect,
}

/// Captures the selected connection and its cloned live handle under the same lock snapshot.
/// Callers pass this through an operation instead of consulting mutable selection state again.
#[derive(Clone)]
pub struct ResolvedOperation {
    pub context: OperationContext,
    pub connection: Connect,
}

impl ResolvedOperation {
    /// Rejects a host-local integration before it can reach the fixed local adapter.
    pub fn require_capability(&self, kind: &str) -> Result<(), AppError> {
        let available = self.context.capabilities.iter().any(|capability| {
            capability.kind == kind && capability.state == CapabilityState::Available
        });
        if available {
            Ok(())
        } else {
            Err(AppError::Unsupported(format!(
                "The selected connection does not support {kind} operations"
            )))
        }
    }
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

impl Default for ConnectionService {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionService {
    /// Create a new connection service
    pub fn new() -> Self {
        // Always include the local connection.
        let saved = vec![SavedConnection::local()];

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
            let saved_conns = self
                .saved_connections
                .read()
                .map_err(|_| AppError::Other("Failed to lock saved connections".to_string()))?;
            saved_conns.iter().find(|c| c.id == connection_id).cloned()
        };

        let config = saved
            .ok_or_else(|| AppError::Other(format!("Connection '{}' not found", connection_id)))?;

        let uri = config.build_uri();
        tracing::info!("Connecting to libvirt");

        // Attempt connection
        let connection = Connect::open(Some(&uri)).map_err(|_| {
            AppError::LibvirtError("The selected connection could not be opened".to_string())
        })?;

        self.activate_connection(connection_id, config, connection)
    }

    /// Connect an SSH entry using a password supplied for this attempt only. Password
    /// authentication needs libvirt's in-process SSH transport so it can use `ConnectAuth`.
    pub fn connect_with_password(
        &self,
        connection_id: &str,
        password: String,
    ) -> Result<(), AppError> {
        if password.is_empty() {
            return Err(AppError::InvalidConfig(
                "An SSH password is required".to_string(),
            ));
        }
        let config = self.saved_connection(connection_id)?;
        if config.connection_type != ConnectionType::Ssh {
            return Err(AppError::InvalidConfig(
                "Password authentication is only available for SSH connections".to_string(),
            ));
        }

        let username = config
            .username
            .clone()
            .unwrap_or_else(|| "root".to_string());
        let uri = config.password_auth_uri();
        tracing::info!("Connecting to libvirt with interactive SSH authentication");

        let mut auth = ConnectAuth::new(
            vec![
                sys::VIR_CRED_AUTHNAME,
                sys::VIR_CRED_PASSPHRASE,
                sys::VIR_CRED_NOECHOPROMPT,
                sys::VIR_CRED_ECHOPROMPT,
            ],
            provide_ssh_credentials,
        );
        let connection = EPHEMERAL_SSH_CREDENTIALS
            .with(|stored| {
                let previous = stored.replace(Some(EphemeralSshCredentials { username, password }));
                let result = Connect::open_auth(Some(&uri), &mut auth, 0);
                stored.replace(previous);
                result
            })
            .map_err(|_| {
                AppError::LibvirtError(
                    "The SSH connection could not be opened with the supplied credentials"
                        .to_string(),
                )
            })?;

        self.activate_connection(connection_id, config, connection)
    }

    fn saved_connection(&self, connection_id: &str) -> Result<SavedConnection, AppError> {
        let saved_conns = self
            .saved_connections
            .read()
            .map_err(|_| AppError::Other("Failed to lock saved connections".to_string()))?;
        saved_conns
            .iter()
            .find(|connection| connection.id == connection_id)
            .cloned()
            .ok_or_else(|| AppError::Other(format!("Connection '{}' not found", connection_id)))
    }

    fn activate_connection(
        &self,
        connection_id: &str,
        config: SavedConnection,
        connection: Connect,
    ) -> Result<(), AppError> {
        tracing::info!("Successfully connected to libvirt");

        // Store active connection
        {
            let mut conns = self
                .connections
                .write()
                .map_err(|_| AppError::Other("Failed to lock connections".to_string()))?;
            conns.insert(
                connection_id.to_string(),
                ActiveConnection {
                    config: config.clone(),
                    connection,
                },
            );
        }

        // Set as active
        {
            let mut active = self
                .active_connection_id
                .write()
                .map_err(|_| AppError::Other("Failed to lock active connection".to_string()))?;
            *active = Some(connection_id.to_string());
        }

        Ok(())
    }

    /// Disconnect from a connection
    pub fn disconnect(&self, connection_id: &str) -> Result<(), AppError> {
        let mut conns = self
            .connections
            .write()
            .map_err(|_| AppError::Other("Failed to lock connections".to_string()))?;

        if let Some(_conn) = conns.remove(connection_id) {
            tracing::info!("Disconnected from {}", connection_id);
        }

        // If this was the active connection, clear it
        let mut active = self
            .active_connection_id
            .write()
            .map_err(|_| AppError::Other("Failed to lock active connection".to_string()))?;
        if active.as_deref() == Some(connection_id) {
            *active = None;
        }

        Ok(())
    }

    /// Get the active libvirt connection
    #[allow(dead_code)]
    pub fn get_active_connection(&self) -> Result<Connect, AppError> {
        self.resolve_operation(OperationKind::Query, None)
            .map(|resolved| resolved.connection)
    }

    /// Resolves one operation context and a live libvirt handle while the active selection and
    /// connection map are both read-locked. Reconnection or selection changes after this point
    /// cannot redirect the already-captured operation to another host.
    pub fn resolve_operation(
        &self,
        operation_kind: OperationKind,
        target: Option<TargetIdentity>,
    ) -> Result<ResolvedOperation, AppError> {
        // Writers acquire the connection map before the active ID; retain that order here.
        let connections = self.connections.read().map_err(|_| {
            AppError::Unavailable("The selected connection is unavailable".to_string())
        })?;
        let active_id = self
            .active_connection_id
            .read()
            .map_err(|_| {
                AppError::Unavailable("The selected connection is unavailable".to_string())
            })?
            .clone()
            .ok_or_else(|| AppError::Unavailable("No selected connection is active".to_string()))?;
        let active = connections.get(&active_id).ok_or_else(|| {
            AppError::Unavailable("The selected connection is unavailable".to_string())
        })?;
        if !active.connection.is_alive().unwrap_or(false) {
            return Err(AppError::Unavailable(
                "The selected connection is unavailable".to_string(),
            ));
        }

        let scope = connection_scope(&active.config);
        let context = OperationContext {
            operation_id: Uuid::new_v4().to_string(),
            operation_kind,
            connection_id: active.config.id.clone(),
            connection_label: active.config.name.clone(),
            connection_scope: scope.clone(),
            capabilities: connection_capabilities(&scope),
            target,
            captured_at: Utc::now().to_rfc3339(),
        };
        Ok(ResolvedOperation {
            context,
            connection: active.connection.clone(),
        })
    }

    /// Get the currently active connection info
    pub fn get_active_connection_info(&self) -> Result<Option<SavedConnection>, AppError> {
        let active_id = {
            let active = self
                .active_connection_id
                .read()
                .map_err(|_| AppError::Other("Failed to lock active connection".to_string()))?;
            active.clone()
        };

        match active_id {
            Some(id) => {
                let conns = self
                    .connections
                    .read()
                    .map_err(|_| AppError::Other("Failed to lock connections".to_string()))?;
                Ok(conns.get(&id).map(|c| c.config.clone()))
            }
            None => Ok(None),
        }
    }

    /// Check if actively connected
    #[allow(dead_code)]
    pub fn is_connected(&self) -> bool {
        let active_id = self
            .active_connection_id
            .read()
            .ok()
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
        let saved = self
            .saved_connections
            .read()
            .map_err(|_| AppError::Other("Failed to lock saved connections".to_string()))?;
        Ok(saved.clone())
    }

    /// Add a new saved connection
    pub fn add_connection(&self, config: SavedConnection) -> Result<(), AppError> {
        let mut saved = self
            .saved_connections
            .write()
            .map_err(|_| AppError::Other("Failed to lock saved connections".to_string()))?;

        // Check for duplicate ID
        if saved.iter().any(|c| c.id == config.id) {
            return Err(AppError::Other(format!(
                "Connection '{}' already exists",
                config.id
            )));
        }

        saved.push(config);
        Ok(())
    }

    /// Update an existing connection
    pub fn update_connection(&self, config: SavedConnection) -> Result<(), AppError> {
        let mut saved = self
            .saved_connections
            .write()
            .map_err(|_| AppError::Other("Failed to lock saved connections".to_string()))?;

        if let Some(existing) = saved.iter_mut().find(|c| c.id == config.id) {
            *existing = config;
            Ok(())
        } else {
            Err(AppError::Other(format!(
                "Connection '{}' not found",
                config.id
            )))
        }
    }

    /// Remove a saved connection
    pub fn remove_connection(&self, connection_id: &str) -> Result<(), AppError> {
        // Cannot remove local connection
        if connection_id == "local" {
            return Err(AppError::Other(
                "Cannot remove local connection".to_string(),
            ));
        }

        // Disconnect first if connected
        let _ = self.disconnect(connection_id);

        let mut saved = self
            .saved_connections
            .write()
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
        let version = conn
            .get_lib_version()
            .map_err(|e| AppError::LibvirtError(format!("Failed to get version: {}", e)))?;

        let major = version / 1000000;
        let minor = (version % 1000000) / 1000;
        let release = version % 1000;

        Ok(format!("{}.{}.{}", major, minor, release))
    }
}

fn connection_scope(config: &SavedConnection) -> ConnectionScope {
    match config.connection_type {
        ConnectionType::Local if config.path == "session" => ConnectionScope::LocalSession,
        ConnectionType::Local => ConnectionScope::LocalSystem,
        ConnectionType::Ssh | ConnectionType::Tls | ConnectionType::Tcp => ConnectionScope::Remote,
    }
}

fn connection_capabilities(scope: &ConnectionScope) -> Vec<ConnectionCapability> {
    let checked_at = Utc::now().to_rfc3339();
    let host_device = matches!(scope, ConnectionScope::LocalSystem);
    vec![
        ConnectionCapability {
            kind: "resourceManagement".to_string(),
            state: CapabilityState::Available,
            reason_code: None,
            recovery_action: None,
            checked_at: checked_at.clone(),
        },
        ConnectionCapability {
            kind: "hostDevice".to_string(),
            state: if host_device {
                CapabilityState::Available
            } else {
                CapabilityState::Unavailable
            },
            reason_code: (!host_device).then(|| "requiresLocalHost".to_string()),
            recovery_action: None,
            checked_at,
        },
        ConnectionCapability {
            kind: "migration".to_string(),
            state: CapabilityState::Available,
            reason_code: None,
            recovery_action: None,
            checked_at: Utc::now().to_rfc3339(),
        },
        ConnectionCapability {
            kind: "console".to_string(),
            state: if host_device {
                CapabilityState::Available
            } else {
                CapabilityState::Unavailable
            },
            reason_code: (!host_device).then(|| "requiresLocalHost".to_string()),
            recovery_action: None,
            checked_at: Utc::now().to_rfc3339(),
        },
        ConnectionCapability {
            kind: "guestAgent".to_string(),
            state: if host_device {
                CapabilityState::Available
            } else {
                CapabilityState::Unavailable
            },
            reason_code: (!host_device).then(|| "requiresLocalHost".to_string()),
            recovery_action: None,
            checked_at: Utc::now().to_rfc3339(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::error::{SafeFailure, SafeFailureCode};
    use std::path::PathBuf;
    use virt::domain::Domain;

    fn fixture_connection(name: &str) -> Connect {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/hardening/connections")
            .join(name);
        Connect::open(Some(&format!("test://{}", path.display()))).unwrap()
    }

    fn ssh_connection() -> SavedConnection {
        SavedConnection {
            id: "remote".to_string(),
            name: "Remote host".to_string(),
            connection_type: ConnectionType::Ssh,
            hypervisor: "qemu".to_string(),
            host: Some("192.0.2.10".to_string()),
            username: Some("admin".to_string()),
            ssh_port: Some(2222),
            tls_port: None,
            auto_connect: false,
            path: "system".to_string(),
        }
    }

    #[test]
    fn agent_ssh_uri_never_requests_a_terminal_password() {
        assert_eq!(
            ssh_connection().build_uri(),
            "qemu+ssh://admin@192.0.2.10:2222/system?no_tty=1"
        );
    }

    #[test]
    fn password_auth_uri_uses_libvirt_callback_transport_without_embedding_a_secret() {
        let uri = ssh_connection().password_auth_uri();
        assert_eq!(
            uri,
            "qemu+libssh2://admin@192.0.2.10:2222/system?sshauth=password,keyboard-interactive"
        );
        assert!(!uri.contains("password="));
    }

    #[test]
    fn resolves_context_and_handle_from_one_selected_connection_snapshot() {
        let connection_a = fixture_connection("fixture-a.xml");
        let connection_b = fixture_connection("fixture-b.xml");
        let service = ConnectionService {
            active_connection_id: RwLock::new(Some("fixture-a".to_string())),
            connections: RwLock::new(HashMap::from([
                (
                    "fixture-a".to_string(),
                    ActiveConnection {
                        config: SavedConnection {
                            id: "fixture-a".to_string(),
                            name: "Fixture A".to_string(),
                            connection_type: ConnectionType::Local,
                            hypervisor: "test".to_string(),
                            host: None,
                            username: None,
                            ssh_port: None,
                            tls_port: None,
                            auto_connect: false,
                            path: "system".to_string(),
                        },
                        connection: connection_a,
                    },
                ),
                (
                    "fixture-b".to_string(),
                    ActiveConnection {
                        config: SavedConnection {
                            id: "fixture-b".to_string(),
                            name: "Fixture B".to_string(),
                            connection_type: ConnectionType::Local,
                            hypervisor: "test".to_string(),
                            host: None,
                            username: None,
                            ssh_port: None,
                            tls_port: None,
                            auto_connect: false,
                            path: "system".to_string(),
                        },
                        connection: connection_b,
                    },
                ),
            ])),
            saved_connections: RwLock::new(Vec::new()),
        };

        let resolved = service
            .resolve_operation(OperationKind::Query, None)
            .unwrap();
        let selected = Domain::lookup_by_name(&resolved.connection, "same-name").unwrap();

        assert_eq!(resolved.context.connection_id, "fixture-a");
        assert_eq!(resolved.context.connection_label, "Fixture A");
        assert_eq!(
            selected.get_uuid_string().unwrap(),
            "00000000-0000-0000-0000-0000000000a1"
        );
    }

    #[test]
    fn remote_connections_explicitly_gate_local_console_and_host_devices() {
        let capabilities = connection_capabilities(&ConnectionScope::Remote);

        for kind in ["console", "guestAgent", "hostDevice"] {
            let capability = capabilities
                .iter()
                .find(|capability| capability.kind == kind)
                .unwrap();
            assert_eq!(capability.state, CapabilityState::Unavailable);
            assert_eq!(capability.reason_code.as_deref(), Some("requiresLocalHost"));
        }
        assert_eq!(
            capabilities
                .iter()
                .find(|capability| capability.kind == "migration")
                .unwrap()
                .state,
            CapabilityState::Available
        );
    }

    #[test]
    fn unavailable_capability_is_rejected_before_a_host_local_operation() {
        let operation = ResolvedOperation {
            context: OperationContext {
                operation_id: "operation".to_string(),
                operation_kind: OperationKind::HostLocal,
                connection_id: "remote".to_string(),
                connection_label: "Remote".to_string(),
                connection_scope: ConnectionScope::Remote,
                capabilities: connection_capabilities(&ConnectionScope::Remote),
                target: None,
                captured_at: "now".to_string(),
            },
            connection: fixture_connection("fixture-a.xml"),
        };

        assert!(operation.require_capability("hostDevice").is_err());
        assert!(operation.require_capability("resourceManagement").is_ok());
    }

    #[test]
    fn poisoned_connection_state_returns_an_unavailable_error_without_panicking() {
        let service = std::sync::Arc::new(ConnectionService::new());
        let poisoned = service.clone();
        let _ = std::thread::spawn(move || {
            let _guard = poisoned.connections.write().unwrap();
            panic!("intentional lock poison");
        })
        .join();

        let error = match service.resolve_operation(OperationKind::Query, None) {
            Ok(_) => panic!("a poisoned connection lock must not resolve"),
            Err(error) => error,
        };
        let failure = SafeFailure::from(error);
        assert_eq!(failure.code, SafeFailureCode::Unavailable);
    }
}
