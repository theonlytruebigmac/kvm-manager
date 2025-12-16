use std::sync::Arc;
use crate::services::libvirt::LibvirtService;
use crate::services::connection_service::ConnectionService;
use crate::services::metrics_service::MetricsService;
use crate::services::retention_service::RetentionService;
use crate::services::guest_agent_service::GuestAgentService;
use crate::services::ws_proxy::WsProxyService;
use crate::services::serial_console_service::SerialConsoleService;
use crate::utils::error::AppError;

/// Application state shared across all Tauri commands
pub struct AppState {
    pub libvirt: Arc<LibvirtService>,
    pub connections: Arc<ConnectionService>,
    pub metrics: Arc<MetricsService>,
    pub retention_service: Arc<RetentionService>,
    pub guest_agent: Arc<GuestAgentService>,
    pub ws_proxy: Arc<WsProxyService>,
    pub serial_console: Arc<SerialConsoleService>,
}

impl AppState {
    /// Create a new AppState instance with libvirt connection and metrics service
    pub fn new() -> Result<Self, AppError> {
        tracing::info!("Initializing AppState");

        let libvirt = Arc::new(LibvirtService::new()?);
        let connections = Arc::new(ConnectionService::new());

        // Auto-connect to local
        connections.connect("local")?;

        let metrics = Arc::new(MetricsService::new(None)?);
        let retention_service = Arc::new(RetentionService::new(metrics.clone())?);
        let guest_agent = Arc::new(GuestAgentService::new());
        let ws_proxy = Arc::new(WsProxyService::new());
        let serial_console = Arc::new(SerialConsoleService::new());

        tracing::info!("AppState initialized successfully");

        Ok(Self {
            libvirt,
            connections,
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
}
