use crate::models::network::{Network, NetworkConfig};
use crate::models::operation::{OperationKind, TargetIdentity};
use crate::services::network_service::NetworkService;
use crate::state::app_state::AppState;
use crate::utils::error::SafeFailure;
use tauri::State;

fn network_target(name: &str) -> TargetIdentity {
    TargetIdentity {
        resource_kind: "network".to_string(),
        stable_id: name.to_string(),
        display_name: None,
    }
}

/// Get all virtual networks (active and inactive)
#[tauri::command]
pub async fn get_networks(state: State<'_, AppState>) -> Result<Vec<Network>, SafeFailure> {
    tracing::debug!("get_networks command called");

    let operation = state
        .resolve_operation(OperationKind::Query, None)
        .map_err(SafeFailure::from)?;
    NetworkService::list_networks(&operation.connection).map_err(SafeFailure::from)
}

/// Get a single network by name
#[tauri::command]
pub async fn get_network(
    state: State<'_, AppState>,
    network_name: String,
) -> Result<Network, SafeFailure> {
    tracing::debug!("get_network command called for network: {}", network_name);

    let operation = state
        .resolve_operation(OperationKind::Query, Some(network_target(&network_name)))
        .map_err(SafeFailure::from)?;
    NetworkService::get_network(&operation.connection, &network_name)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Create a new virtual network
#[tauri::command]
pub async fn create_network(
    state: State<'_, AppState>,
    config: NetworkConfig,
) -> Result<String, SafeFailure> {
    tracing::info!("create_network command called for network: {}", config.name);

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(network_target(&config.name)))
        .map_err(SafeFailure::from)?;
    NetworkService::create_network(&operation.connection, config)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Delete a virtual network
#[tauri::command]
pub async fn delete_network(
    state: State<'_, AppState>,
    network_name: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "delete_network command called for network: {}",
        network_name
    );

    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "delete_network",
        "network",
        &network_name,
        Some(network_name.clone()),
        "delete",
    )
    .map_err(SafeFailure::from)?;
    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(network_target(&network_name)))
        .map_err(SafeFailure::from)?;
    NetworkService::delete_network(&operation.connection, &network_name)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Start a network
#[tauri::command]
pub async fn start_network(
    state: State<'_, AppState>,
    network_name: String,
) -> Result<(), SafeFailure> {
    tracing::info!("start_network command called for network: {}", network_name);

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(network_target(&network_name)))
        .map_err(SafeFailure::from)?;
    NetworkService::start_network(&operation.connection, &network_name)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Stop a network
#[tauri::command]
pub async fn stop_network(
    state: State<'_, AppState>,
    network_name: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!("stop_network command called for network: {}", network_name);

    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "stop_network",
        "network",
        &network_name,
        Some(network_name.clone()),
        "stop",
    )
    .map_err(SafeFailure::from)?;
    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(network_target(&network_name)))
        .map_err(SafeFailure::from)?;
    NetworkService::stop_network(&operation.connection, &network_name)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Add a port forwarding rule
#[tauri::command]
pub async fn add_port_forward(
    host_port: u16,
    guest_ip: String,
    guest_port: u16,
    protocol: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "add_port_forward command called: {}:{} -> {}:{}",
        protocol,
        host_port,
        guest_ip,
        guest_port
    );

    NetworkService::add_port_forward(host_port, &guest_ip, guest_port, &protocol)
        .map_err(SafeFailure::from)
}

/// Remove a port forwarding rule
#[tauri::command]
pub async fn remove_port_forward(
    state: State<'_, AppState>,
    host_port: u16,
    guest_ip: String,
    guest_port: u16,
    protocol: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "remove_port_forward command called: {}:{} -> {}:{}",
        protocol,
        host_port,
        guest_ip,
        guest_port
    );

    let stable_id = format!("{protocol}/{host_port}/{guest_ip}/{guest_port}");
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "remove_port_forward",
        "host_rule",
        &stable_id,
        None,
        "remove",
    )
    .map_err(SafeFailure::from)?;
    NetworkService::remove_port_forward(host_port, &guest_ip, guest_port, &protocol)
        .map_err(SafeFailure::from)
}

/// Set network autostart
#[tauri::command]
pub async fn set_network_autostart(
    state: State<'_, AppState>,
    network_name: String,
    autostart: bool,
) -> Result<(), SafeFailure> {
    tracing::info!(
        "set_network_autostart command called: {} -> {}",
        network_name,
        autostart
    );

    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(network_target(&network_name)))
        .map_err(SafeFailure::from)?;
    NetworkService::set_network_autostart(&operation.connection, &network_name, autostart)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Get DHCP leases for a network
#[tauri::command]
pub async fn get_dhcp_leases(
    state: State<'_, AppState>,
    network_name: String,
) -> Result<Vec<crate::services::network_service::DhcpLease>, SafeFailure> {
    tracing::info!("get_dhcp_leases command called: {}", network_name);

    let operation = state
        .resolve_operation(OperationKind::Query, Some(network_target(&network_name)))
        .map_err(SafeFailure::from)?;
    NetworkService::get_dhcp_leases(&operation.connection, &network_name)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Get detailed network information
#[tauri::command]
pub async fn get_network_details(
    state: State<'_, AppState>,
    network_name: String,
) -> Result<crate::services::network_service::NetworkDetails, SafeFailure> {
    tracing::info!("get_network_details command called: {}", network_name);

    let operation = state
        .resolve_operation(OperationKind::Query, Some(network_target(&network_name)))
        .map_err(SafeFailure::from)?;
    NetworkService::get_network_details(&operation.connection, &network_name)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}
