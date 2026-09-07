use crate::models::operation::{OperationKind, TargetIdentity};
use crate::services::confirmation_service::ConfirmationService;
use crate::services::connection_service::{ConnectionService, ResolvedOperation};
use crate::services::guest_agent_service::GuestAgentService;
use crate::services::metrics_service::MetricsService;
use crate::services::retention_service::RetentionService;
use crate::services::serial_console_service::SerialConsoleService;
use crate::services::ws_proxy::WsProxyService;
use crate::utils::error::AppError;
use std::sync::Arc;

/// Application state shared across all Tauri commands
pub struct AppState {
    pub connections: Arc<ConnectionService>,
    pub confirmations: Arc<ConfirmationService>,
    pub metrics: Arc<MetricsService>,
    pub retention_service: Arc<RetentionService>,
    pub guest_agent: Arc<GuestAgentService>,
    pub ws_proxy: Arc<WsProxyService>,
    pub serial_console: Arc<SerialConsoleService>,
}

impl AppState {
    /// Create application state. Selected libvirt handles are owned by ConnectionService.
    pub fn new() -> Result<Self, AppError> {
        tracing::info!("Initializing AppState");

        let connections = Arc::new(ConnectionService::new());
        let confirmations = Arc::new(ConfirmationService::new());

        // A failed local connection is a degraded state, not a startup failure.
        if let Err(error) = connections.connect("local") {
            tracing::warn!("Local connection is unavailable at startup: {}", error);
        }

        let metrics = Arc::new(MetricsService::new(None)?);
        let retention_service = Arc::new(RetentionService::new(metrics.clone())?);
        let guest_agent = Arc::new(GuestAgentService::new());
        let ws_proxy = Arc::new(WsProxyService::new());
        let serial_console = Arc::new(SerialConsoleService::new());

        tracing::info!("AppState initialized successfully");

        Ok(Self {
            connections,
            confirmations,
            metrics,
            retention_service,
            guest_agent,
            ws_proxy,
            serial_console,
        })
    }

    /// Start background tasks
    pub fn start_background_tasks(&self) {
        // Start retention policy cleanup task
        let retention = self.retention_service.clone();
        retention.start_cleanup_task();
    }

    /// Captures the active selected connection once for a command. Services must use the returned
    /// handle for every query, mutation, and post-mutation refresh in that operation.
    pub fn resolve_operation(
        &self,
        operation_kind: OperationKind,
        target: Option<TargetIdentity>,
    ) -> Result<ResolvedOperation, AppError> {
        self.connections.resolve_operation(operation_kind, target)
    }

    /// Captures and verifies the selected connection before a local-host integration runs.
    pub fn resolve_host_local_operation(
        &self,
        target: Option<TargetIdentity>,
    ) -> Result<ResolvedOperation, AppError> {
        let operation = self.resolve_operation(OperationKind::HostLocal, target)?;
        operation.require_capability("hostDevice")?;
        Ok(operation)
    }

    /// Captures and verifies the selected connection before a local console adapter runs.
    pub fn resolve_console_operation(
        &self,
        target: Option<TargetIdentity>,
    ) -> Result<ResolvedOperation, AppError> {
        let operation = self.resolve_operation(OperationKind::Console, target)?;
        operation.require_capability("console")?;
        Ok(operation)
    }

    /// Captures and verifies the selected connection before a local guest-agent adapter runs.
    pub fn resolve_guest_agent_operation(
        &self,
        target: Option<TargetIdentity>,
    ) -> Result<ResolvedOperation, AppError> {
        let operation = self.resolve_operation(OperationKind::Query, target)?;
        operation.require_capability("guestAgent")?;
        Ok(operation)
    }
}
