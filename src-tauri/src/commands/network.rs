use tauri::State;
use crate::models::network::{Network, NetworkConfig};
use crate::services::network_service::NetworkService;
use crate::state::app_state::AppState;

/// Get all virtual networks (active and inactive)
#[tauri::command]
pub async fn get_networks(state: State<'_, AppState>) -> Result<Vec<Network>, String> {
    tracing::debug!("get_networks command called");

    NetworkService::list_networks(&state.libvirt)
        .map_err(|e| e.to_string())
}

/// Get a single network by name
#[tauri::command]
pub async fn get_network(state: State<'_, AppState>, network_name: String) -> Result<Network, String> {
    tracing::debug!("get_network command called for network: {}", network_name);

    NetworkService::get_network(&state.libvirt, &network_name)
        .map_err(|e| e.to_string())
}

/// Create a new virtual network
#[tauri::command]
pub async fn create_network(
    state: State<'_, AppState>,
    config: NetworkConfig,
) -> Result<String, String> {
    tracing::info!("create_network command called for network: {}", config.name);

    NetworkService::create_network(&state.libvirt, config)
        .map_err(|e| e.to_string())
}

/// Delete a virtual network
#[tauri::command]
pub async fn delete_network(
    state: State<'_, AppState>,
    network_name: String,
) -> Result<(), String> {
    tracing::info!("delete_network command called for network: {}", network_name);

    NetworkService::delete_network(&state.libvirt, &network_name)
        .map_err(|e| e.to_string())
}

/// Start a network
#[tauri::command]
pub async fn start_network(
    state: State<'_, AppState>,
    network_name: String,
) -> Result<(), String> {
    tracing::info!("start_network command called for network: {}", network_name);

    NetworkService::start_network(&state.libvirt, &network_name)
        .map_err(|e| e.to_string())
}

/// Stop a network
#[tauri::command]
pub async fn stop_network(
    state: State<'_, AppState>,
    network_name: String,
) -> Result<(), String> {
    tracing::info!("stop_network command called for network: {}", network_name);

    NetworkService::stop_network(&state.libvirt, &network_name)
        .map_err(|e| e.to_string())
}
