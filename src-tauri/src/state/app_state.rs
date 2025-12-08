use std::sync::Arc;
use crate::services::libvirt::LibvirtService;
use crate::utils::error::AppError;

/// Application state shared across all Tauri commands
pub struct AppState {
    pub libvirt: Arc<LibvirtService>,
}

impl AppState {
    /// Create a new AppState instance with libvirt connection
    pub fn new() -> Result<Self, AppError> {
        tracing::info!("Initializing AppState");

        let libvirt = Arc::new(LibvirtService::new()?);

        tracing::info!("AppState initialized successfully");

        Ok(Self { libvirt })
    }
}
