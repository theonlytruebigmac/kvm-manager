use std::sync::Arc;
use crate::services::libvirt::LibvirtService;
use crate::services::metrics_service::MetricsService;
use crate::services::retention_service::RetentionService;
use crate::utils::error::AppError;

/// Application state shared across all Tauri commands
pub struct AppState {
    pub libvirt: Arc<LibvirtService>,
    pub metrics: Arc<MetricsService>,
    pub retention_service: Arc<RetentionService>,
}

impl AppState {
    /// Create a new AppState instance with libvirt connection and metrics service
    pub fn new() -> Result<Self, AppError> {
        tracing::info!("Initializing AppState");

        let libvirt = Arc::new(LibvirtService::new()?);
        let metrics = Arc::new(MetricsService::new(None)?);
        let retention_service = Arc::new(RetentionService::new(metrics.clone())?);

        tracing::info!("AppState initialized successfully");

        Ok(Self {
            libvirt,
            metrics,
            retention_service,
        })
    }

    /// Start background tasks
    pub fn start_background_tasks(&self) {
        // Start retention policy cleanup task
        let retention = self.retention_service.clone();
        tokio::spawn(async move {
            retention.start_cleanup_task().await;
        });
    }
}
