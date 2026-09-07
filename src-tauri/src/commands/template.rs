use crate::services::template_service::{CreateTemplateRequest, TemplateService, VmTemplate};
use crate::state::app_state::AppState;
use crate::utils::error::SafeFailure;
use tauri::State;

/// Create a new VM template
#[tauri::command]
pub async fn create_template(
    _state: State<'_, AppState>,
    request: CreateTemplateRequest,
) -> Result<VmTemplate, SafeFailure> {
    let template_service = TemplateService::new().map_err(SafeFailure::from)?;
    template_service
        .create_template(request)
        .map_err(SafeFailure::from)
}

/// List all VM templates
#[tauri::command]
pub async fn list_templates(_state: State<'_, AppState>) -> Result<Vec<VmTemplate>, SafeFailure> {
    let template_service = TemplateService::new().map_err(SafeFailure::from)?;
    template_service.list_templates().map_err(SafeFailure::from)
}

/// Get a template by ID
#[tauri::command]
pub async fn get_template(
    _state: State<'_, AppState>,
    id: String,
) -> Result<VmTemplate, SafeFailure> {
    let template_service = TemplateService::new().map_err(SafeFailure::from)?;
    template_service
        .get_template(&id)
        .map_err(SafeFailure::from)
}

/// Update a template
#[tauri::command]
pub async fn update_template(
    _state: State<'_, AppState>,
    id: String,
    request: CreateTemplateRequest,
) -> Result<VmTemplate, SafeFailure> {
    let template_service = TemplateService::new().map_err(SafeFailure::from)?;
    template_service
        .update_template(&id, request)
        .map_err(SafeFailure::from)
}

/// Delete a template
#[tauri::command]
pub async fn delete_template(
    state: State<'_, AppState>,
    id: String,
    confirmation_token: String,
) -> Result<(), SafeFailure> {
    crate::commands::confirmation::require_destructive_confirmation(
        &state,
        &confirmation_token,
        "delete_template",
        "template",
        &id,
        None,
        "delete",
    )
    .map_err(SafeFailure::from)?;
    let template_service = TemplateService::new().map_err(SafeFailure::from)?;
    template_service
        .delete_template(&id)
        .map_err(SafeFailure::from)
}
