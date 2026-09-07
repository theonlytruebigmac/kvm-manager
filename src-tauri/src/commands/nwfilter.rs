use crate::models::nwfilter::{NwFilter, NwFilterConfig};
use crate::models::operation::{OperationKind, TargetIdentity};
use crate::services::nwfilter_service::NwFilterService;
use crate::state::app_state::AppState;
use crate::utils::error::SafeFailure;
use crate::utils::xml::validate_document_root;
use tauri::State;

fn filter_target(name: &str) -> TargetIdentity {
    TargetIdentity {
        resource_kind: "network_filter".to_string(),
        stable_id: name.to_string(),
        display_name: None,
    }
}

/// List all network filters
#[tauri::command]
pub async fn get_nwfilters(state: State<'_, AppState>) -> Result<Vec<NwFilter>, SafeFailure> {
    tracing::debug!("get_nwfilters command called");
    let operation = state
        .resolve_operation(OperationKind::Query, None)
        .map_err(SafeFailure::from)?;
    NwFilterService::list_filters(&operation.connection).map_err(SafeFailure::from)
}

/// Get a specific network filter by name
#[tauri::command]
pub async fn get_nwfilter(
    state: State<'_, AppState>,
    name: String,
) -> Result<NwFilter, SafeFailure> {
    tracing::debug!("get_nwfilter command called for: {}", name);
    let operation = state
        .resolve_operation(OperationKind::Query, Some(filter_target(&name)))
        .map_err(SafeFailure::from)?;
    NwFilterService::get_filter(&operation.connection, &name)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Get the XML of a network filter
#[tauri::command]
pub async fn get_nwfilter_xml(
    state: State<'_, AppState>,
    name: String,
) -> Result<String, SafeFailure> {
    tracing::debug!("get_nwfilter_xml command called");
    let operation = state
        .resolve_operation(OperationKind::Query, Some(filter_target(&name)))
        .map_err(SafeFailure::from)?;
    NwFilterService::get_filter_xml(&operation.connection, &name)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Create a new network filter
#[tauri::command]
pub async fn create_nwfilter(
    state: State<'_, AppState>,
    config: NwFilterConfig,
) -> Result<NwFilter, SafeFailure> {
    tracing::info!("create_nwfilter command called for: {}", config.name);
    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(filter_target(&config.name)))
        .map_err(SafeFailure::from)?;
    NwFilterService::create_filter(&operation.connection, config)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Create a network filter from raw XML
#[tauri::command]
pub async fn create_nwfilter_from_xml(
    state: State<'_, AppState>,
    xml: String,
) -> Result<NwFilter, SafeFailure> {
    tracing::info!("create_nwfilter_from_xml command called");
    validate_document_root(&xml, "filter").map_err(SafeFailure::from)?;
    let operation = state
        .resolve_operation(OperationKind::Mutation, None)
        .map_err(SafeFailure::from)?;
    NwFilterService::create_filter_from_xml(&operation.connection, &xml)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}

/// Delete a network filter
#[tauri::command]
pub async fn delete_nwfilter(
    state: State<'_, AppState>,
    name: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    tracing::info!("delete_nwfilter command called for: {}", name);
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "delete_nwfilter",
        "filter",
        &name,
        Some(name.clone()),
        "delete",
    )
    .map_err(SafeFailure::from)?;
    let operation = state
        .resolve_operation(OperationKind::Mutation, Some(filter_target(&name)))
        .map_err(SafeFailure::from)?;
    NwFilterService::delete_filter(&operation.connection, &name)
        .map_err(SafeFailure::from)
        .map_err(|failure| failure.with_context(&operation.context))
}
