use tauri::State;
use crate::state::app_state::AppState;
use crate::services::connection_service::{SavedConnection, ConnectionType};

/// Get all saved connections
#[tauri::command]
pub async fn get_saved_connections(state: State<'_, AppState>) -> Result<Vec<SavedConnection>, String> {
    tracing::debug!("get_saved_connections command called");
    state.connections.get_saved_connections()
        .map_err(|e| e.to_string())
}

/// Get currently active connection info
#[tauri::command]
pub async fn get_active_connection(state: State<'_, AppState>) -> Result<Option<SavedConnection>, String> {
    tracing::debug!("get_active_connection command called");
    state.connections.get_active_connection_info()
        .map_err(|e| e.to_string())
}

/// Connect to a saved connection by ID
#[tauri::command]
pub async fn connect_to(state: State<'_, AppState>, connection_id: String) -> Result<(), String> {
    tracing::info!("connect_to command called for: {}", connection_id);
    state.connections.connect(&connection_id)
        .map_err(|e| e.to_string())
}

/// Disconnect from a connection
#[tauri::command]
pub async fn disconnect_from(state: State<'_, AppState>, connection_id: String) -> Result<(), String> {
    tracing::info!("disconnect_from command called for: {}", connection_id);
    state.connections.disconnect(&connection_id)
        .map_err(|e| e.to_string())
}

/// Add a new connection
#[tauri::command]
pub async fn add_connection(
    state: State<'_, AppState>,
    name: String,
    connection_type: String,
    host: Option<String>,
    username: Option<String>,
    ssh_port: Option<u16>,
    tls_port: Option<u16>,
    auto_connect: bool,
) -> Result<SavedConnection, String> {
    tracing::info!("add_connection command called: {}", name);

    let conn_type = match connection_type.as_str() {
        "local" => ConnectionType::Local,
        "ssh" => ConnectionType::Ssh,
        "tls" => ConnectionType::Tls,
        "tcp" => ConnectionType::Tcp,
        _ => return Err(format!("Invalid connection type: {}", connection_type)),
    };

    // Generate unique ID
    let id = format!("conn_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("unknown"));

    let config = SavedConnection {
        id: id.clone(),
        name,
        connection_type: conn_type,
        hypervisor: "qemu".to_string(),
        host,
        username,
        ssh_port,
        tls_port,
        auto_connect,
        path: "system".to_string(),
    };

    state.connections.add_connection(config.clone())
        .map_err(|e| e.to_string())?;

    Ok(config)
}

/// Update an existing connection
#[tauri::command]
pub async fn update_connection(
    state: State<'_, AppState>,
    id: String,
    name: String,
    connection_type: String,
    host: Option<String>,
    username: Option<String>,
    ssh_port: Option<u16>,
    tls_port: Option<u16>,
    auto_connect: bool,
) -> Result<(), String> {
    tracing::info!("update_connection command called: {}", id);

    let conn_type = match connection_type.as_str() {
        "local" => ConnectionType::Local,
        "ssh" => ConnectionType::Ssh,
        "tls" => ConnectionType::Tls,
        "tcp" => ConnectionType::Tcp,
        _ => return Err(format!("Invalid connection type: {}", connection_type)),
    };

    let config = SavedConnection {
        id,
        name,
        connection_type: conn_type,
        hypervisor: "qemu".to_string(),
        host,
        username,
        ssh_port,
        tls_port,
        auto_connect,
        path: "system".to_string(),
    };

    state.connections.update_connection(config)
        .map_err(|e| e.to_string())
}

/// Remove a saved connection
#[tauri::command]
pub async fn remove_connection(state: State<'_, AppState>, connection_id: String) -> Result<(), String> {
    tracing::info!("remove_connection command called: {}", connection_id);
    state.connections.remove_connection(&connection_id)
        .map_err(|e| e.to_string())
}

/// Test a connection before saving
#[tauri::command]
pub async fn test_connection(
    connection_type: String,
    host: Option<String>,
    username: Option<String>,
    ssh_port: Option<u16>,
    tls_port: Option<u16>,
) -> Result<String, String> {
    use virt::connect::Connect;

    tracing::info!("test_connection command called");

    let conn_type = match connection_type.as_str() {
        "local" => ConnectionType::Local,
        "ssh" => ConnectionType::Ssh,
        "tls" => ConnectionType::Tls,
        "tcp" => ConnectionType::Tcp,
        _ => return Err(format!("Invalid connection type: {}", connection_type)),
    };

    // Build test config
    let config = SavedConnection {
        id: "test".to_string(),
        name: "Test".to_string(),
        connection_type: conn_type,
        hypervisor: "qemu".to_string(),
        host,
        username,
        ssh_port,
        tls_port,
        auto_connect: false,
        path: "system".to_string(),
    };

    let uri = config.build_uri();
    tracing::info!("Testing connection to: {}", uri);

    // Try to connect
    let conn = Connect::open(Some(&uri))
        .map_err(|e| format!("Connection failed: {}", e))?;

    // Get hostname as confirmation
    let hostname = conn.get_hostname()
        .map_err(|e| format!("Failed to get hostname: {}", e))?;

    Ok(format!("Connected to {}", hostname))
}
