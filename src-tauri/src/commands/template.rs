use crate::services::template_service::{CreateTemplateRequest, TemplateService, VmTemplate};
use crate::state::app_state::AppState;
use tauri::State;

/// Create a new VM template
#[tauri::command]
pub async fn create_template(
    _state: State<'_, AppState>,
    request: CreateTemplateRequest,
) -> Result<VmTemplate, String> {
    let template_service = TemplateService::new().map_err(|e| e.to_string())?;
    template_service.create_template(request).map_err(|e| e.to_string())
}

/// List all VM templates
#[tauri::command]
pub async fn list_templates(
    _state: State<'_, AppState>,
) -> Result<Vec<VmTemplate>, String> {
    let template_service = TemplateService::new().map_err(|e| e.to_string())?;
    template_service.list_templates().map_err(|e| e.to_string())
}

/// Get a template by ID
#[tauri::command]
pub async fn get_template(
    _state: State<'_, AppState>,
    id: String,
) -> Result<VmTemplate, String> {
    let template_service = TemplateService::new().map_err(|e| e.to_string())?;
    template_service.get_template(&id).map_err(|e| e.to_string())
}

/// Update a template
#[tauri::command]
pub async fn update_template(
    _state: State<'_, AppState>,
    id: String,
    request: CreateTemplateRequest,
) -> Result<VmTemplate, String> {
    let template_service = TemplateService::new().map_err(|e| e.to_string())?;
    template_service.update_template(&id, request).map_err(|e| e.to_string())
}

/// Delete a template
#[tauri::command]
pub async fn delete_template(
    _state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let template_service = TemplateService::new().map_err(|e| e.to_string())?;
    template_service.delete_template(&id).map_err(|e| e.to_string())
}
