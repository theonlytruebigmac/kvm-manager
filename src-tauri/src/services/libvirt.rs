use virt::connect::Connect;
use crate::utils::error::{AppError, map_libvirt_error};

/// LibvirtService manages the connection to libvirtd
pub struct LibvirtService {
    connection: Connect,
}

impl LibvirtService {
    /// Create a new LibvirtService and connect to qemu:///system
    pub fn new() -> Result<Self, AppError> {
        tracing::info!("Connecting to libvirt at qemu:///system");

        let connection = Connect::open(Some("qemu:///system"))
            .map_err(map_libvirt_error)?;

        tracing::info!("Successfully connected to libvirt");

        Ok(Self { connection })
    }

    /// Get a reference to the libvirt connection
    pub fn get_connection(&self) -> &Connect {
        &self.connection
    }

    /// Check if the connection is alive
    pub fn is_alive(&self) -> bool {
        self.connection.is_alive().unwrap_or(false)
    }

    /// Get libvirt version
    pub fn get_version(&self) -> Result<String, AppError> {
        let version = self.connection.get_lib_version()
            .map_err(map_libvirt_error)?;

        // Convert version number to string (e.g., 8002000 -> "8.2.0")
        let major = version / 1000000;
        let minor = (version % 1000000) / 1000;
        let release = version % 1000;

        Ok(format!("{}.{}.{}", major, minor, release))
    }

    /// Get hostname of the libvirt host
    pub fn get_hostname(&self) -> Result<String, AppError> {
        self.connection.get_hostname()
            .map_err(map_libvirt_error)
    }
}
