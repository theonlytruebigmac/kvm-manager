use tauri::State;
use crate::state::app_state::AppState;
use crate::models::nwfilter::{NwFilter, NwFilterConfig};
use crate::services::nwfilter_service::NwFilterService;

/// List all network filters
#[tauri::command]
pub async fn get_nwfilters(state: State<'_, AppState>) -> Result<Vec<NwFilter>, String> {
    tracing::debug!("get_nwfilters command called");
    NwFilterService::list_filters(&state.libvirt)
        .map_err(|e| e.to_string())
}

/// Get a specific network filter by name
#[tauri::command]
pub async fn get_nwfilter(state: State<'_, AppState>, name: String) -> Result<NwFilter, String> {
    tracing::debug!("get_nwfilter command called for: {}", name);
    NwFilterService::get_filter(&state.libvirt, &name)
        .map_err(|e| e.to_string())
}

/// Get the XML of a network filter
#[tauri::command]
pub async fn get_nwfilter_xml(state: State<'_, AppState>, name: String) -> Result<String, String> {
    tracing::debug!("get_nwfilter_xml command called for: {}", name);
    NwFilterService::get_filter_xml(&state.libvirt, &name)
        .map_err(|e| e.to_string())
}

/// Create a new network filter
#[tauri::command]
pub async fn create_nwfilter(
    state: State<'_, AppState>,
    config: NwFilterConfig,
) -> Result<NwFilter, String> {
    tracing::info!("create_nwfilter command called for: {}", config.name);
    NwFilterService::create_filter(&state.libvirt, config)
        .map_err(|e| e.to_string())
}

/// Create a network filter from raw XML
#[tauri::command]
pub async fn create_nwfilter_from_xml(
    state: State<'_, AppState>,
    xml: String,
) -> Result<NwFilter, String> {
    tracing::info!("create_nwfilter_from_xml command called");
    NwFilterService::create_filter_from_xml(&state.libvirt, &xml)
        .map_err(|e| e.to_string())
}

/// Delete a network filter
#[tauri::command]
pub async fn delete_nwfilter(state: State<'_, AppState>, name: String) -> Result<(), String> {
    tracing::info!("delete_nwfilter command called for: {}", name);
    NwFilterService::delete_filter(&state.libvirt, &name)
        .map_err(|e| e.to_string())
}
