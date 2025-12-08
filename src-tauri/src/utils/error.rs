use thiserror::Error;

/// Application error types
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Libvirt error: {0}")]
    LibvirtError(String),

    #[error("VM not found: {0}")]
    VmNotFound(String),

    #[error("VM is already {0}")]
    InvalidVmState(String),

    #[error("Network not found: {0}")]
    NetworkNotFound(String),

    #[error("Network state error: {0}")]
    InvalidNetworkState(String),

    #[error("libvirtd is not running")]
    LibvirtdNotRunning,

    #[error("Permission denied. Add user to libvirt group")]
    PermissionDenied,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

/// Map virt::error::Error to user-friendly AppError
pub fn map_libvirt_error(err: virt::error::Error) -> AppError {
    // Simply wrap the libvirt error message for now
    // We can add more sophisticated error mapping later
    AppError::LibvirtError(err.message().to_string())
}/// Convert AppError to String for Tauri command results
impl From<AppError> for String {
    fn from(err: AppError) -> String {
        err.to_string()
    }
}
