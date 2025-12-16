use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use portable_pty::{CommandBuilder, PtySize, native_pty_system, MasterPty};
use virt::domain::Domain;
use crate::services::libvirt::LibvirtService;
use crate::utils::error::{AppError, map_libvirt_error};

/// Serial console session info
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialConsoleInfo {
    pub vm_id: String,
    pub vm_name: String,
    pub pty_path: String,
    pub active: bool,
}

/// Active serial console connection using portable-pty
#[allow(dead_code)]
pub struct SerialConsoleConnection {
    pub vm_id: String,
    pub vm_name: String,
    pub pty_path: String,
    pub master: Box<dyn MasterPty + Send>,
    pub reader: Box<dyn Read + Send>,
    pub writer: Box<dyn Write + Send>,
}

/// Service for managing serial console connections
pub struct SerialConsoleService {
    active_connections: Mutex<HashMap<String, Arc<Mutex<SerialConsoleConnection>>>>,
}

impl SerialConsoleService {
    pub fn new() -> Self {
        Self {
            active_connections: Mutex::new(HashMap::new()),
        }
    }

    /// Get serial console info for a VM
    pub fn get_serial_console_info(
        libvirt: &LibvirtService,
        vm_id: &str,
    ) -> Result<SerialConsoleInfo, AppError> {
        tracing::debug!("Getting serial console info for VM: {}", vm_id);

        let conn = libvirt.get_connection();
        let domain = Domain::lookup_by_uuid_string(conn, vm_id)
            .map_err(|_| AppError::VmNotFound(vm_id.to_string()))?;

        let name = domain.get_name().map_err(map_libvirt_error)?;
        let is_active = domain.is_active().map_err(map_libvirt_error)?;

        // Get PTY path from domain XML
        let xml = domain.get_xml_desc(0).map_err(map_libvirt_error)?;
        let pty_path = Self::extract_serial_pty(&xml)?;

        Ok(SerialConsoleInfo {
            vm_id: vm_id.to_string(),
            vm_name: name,
            pty_path,
            active: is_active,
        })
    }

    /// Extract serial console PTY path from domain XML
    fn extract_serial_pty(xml: &str) -> Result<String, AppError> {
        // Look for serial console PTY
        // Format: <serial type='pty'>...<source path='/dev/pts/X'/>...</serial>
        // or: <console type='pty'>...<source path='/dev/pts/X'/>...</console>

        // Try serial first
        if let Some(pty) = Self::find_pty_in_section(xml, "serial") {
            return Ok(pty);
        }

        // Try console as fallback
        if let Some(pty) = Self::find_pty_in_section(xml, "console") {
            return Ok(pty);
        }

        Err(AppError::Other("No serial console found for VM".to_string()))
    }

    fn find_pty_in_section(xml: &str, section: &str) -> Option<String> {
        let section_start = format!("<{} type='pty'", section);
        let section_end = format!("</{}>", section);

        if let Some(start_idx) = xml.find(&section_start) {
            if let Some(end_idx) = xml[start_idx..].find(&section_end) {
                let section_content = &xml[start_idx..start_idx + end_idx];

                // Look for source path
                if let Some(source_start) = section_content.find("<source path='") {
                    let path_start = source_start + 14;
                    if let Some(path_end) = section_content[path_start..].find('\'') {
                        return Some(section_content[path_start..path_start + path_end].to_string());
                    }
                }
            }
        }
        None
    }

    /// Open a serial console connection using portable-pty with virsh console
    pub fn open_connection(&self, vm_id: &str, vm_name: &str, pty_path: &str) -> Result<(), AppError> {
        tracing::info!("Opening serial console for VM {} ({}) at {}", vm_id, vm_name, pty_path);

        // Check if already connected and clean up stale connections
        {
            let mut connections = self.active_connections.lock()
                .map_err(|_| AppError::Other("Failed to acquire lock".to_string()))?;

            // Remove any stale connections for this VM
            connections.remove(vm_id);
        }

        // Create a pseudo-terminal using portable-pty
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        }).map_err(|e| AppError::Other(format!("Failed to open PTY: {}", e)))?;

        // Build the virsh console command
        let mut cmd = CommandBuilder::new("virsh");
        cmd.args(["console", vm_name, "--force"]);

        // Spawn the command in the PTY
        let _child = pair.slave.spawn_command(cmd)
            .map_err(|e| AppError::Other(format!("Failed to spawn virsh console: {}", e)))?;

        // Get reader and writer from the master PTY
        let reader = pair.master.try_clone_reader()
            .map_err(|e| AppError::Other(format!("Failed to clone reader: {}", e)))?;

        let writer = pair.master.take_writer()
            .map_err(|e| AppError::Other(format!("Failed to get writer: {}", e)))?;

        let connection = Arc::new(Mutex::new(SerialConsoleConnection {
            vm_id: vm_id.to_string(),
            vm_name: vm_name.to_string(),
            pty_path: pty_path.to_string(),
            master: pair.master,
            reader,
            writer,
        }));

        {
            let mut connections = self.active_connections.lock()
                .map_err(|_| AppError::Other("Failed to acquire lock".to_string()))?;
            connections.insert(vm_id.to_string(), connection);
        }

        tracing::info!("Serial console opened for VM {} ({})", vm_id, vm_name);
        Ok(())
    }

    /// Close a serial console connection
    pub fn close_connection(&self, vm_id: &str) -> Result<(), AppError> {
        tracing::info!("Closing serial console for VM {}", vm_id);

        let mut connections = self.active_connections.lock()
            .map_err(|_| AppError::Other("Failed to acquire lock".to_string()))?;

        if connections.remove(vm_id).is_some() {
            // The PTY and process will be cleaned up when dropped
            tracing::info!("Serial console closed for VM {}", vm_id);
            Ok(())
        } else {
            Err(AppError::Other("No active serial console for this VM".to_string()))
        }
    }

    /// Read from serial console
    pub fn read_output(&self, vm_id: &str) -> Result<String, AppError> {
        let connections = self.active_connections.lock()
            .map_err(|_| AppError::Other("Failed to acquire lock".to_string()))?;

        let connection = connections.get(vm_id)
            .ok_or_else(|| AppError::Other("No active serial console for this VM".to_string()))?;

        let mut conn = connection.lock()
            .map_err(|_| AppError::Other("Failed to acquire connection lock".to_string()))?;

        let mut buffer = [0u8; 4096];
        match conn.reader.read(&mut buffer) {
            Ok(0) => Ok(String::new()),
            Ok(n) => {
                // Convert to string, handling invalid UTF-8 gracefully
                Ok(String::from_utf8_lossy(&buffer[..n]).to_string())
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No data available (non-blocking)
                Ok(String::new())
            }
            Err(e) => Err(AppError::Other(format!("Failed to read from console: {}", e))),
        }
    }

    /// Write to serial console
    pub fn write_input(&self, vm_id: &str, input: &str) -> Result<(), AppError> {
        let connections = self.active_connections.lock()
            .map_err(|_| AppError::Other("Failed to acquire lock".to_string()))?;

        let connection = connections.get(vm_id)
            .ok_or_else(|| AppError::Other("No active serial console for this VM".to_string()))?;

        let mut conn = connection.lock()
            .map_err(|_| AppError::Other("Failed to acquire connection lock".to_string()))?;

        conn.writer.write_all(input.as_bytes())
            .map_err(|e| AppError::Other(format!("Failed to write to console: {}", e)))?;

        conn.writer.flush()
            .map_err(|e| AppError::Other(format!("Failed to flush console: {}", e)))?;

        Ok(())
    }

    /// Check if a serial console is connected
    pub fn is_connected(&self, vm_id: &str) -> bool {
        if let Ok(connections) = self.active_connections.lock() {
            connections.contains_key(vm_id)
        } else {
            false
        }
    }
}
