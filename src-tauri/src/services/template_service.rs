use crate::models::vm::VmConfig;
use crate::utils::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// VM Template structure
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VmTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub config: VmConfig,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Template creation request
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateTemplateRequest {
    pub name: String,
    pub description: String,
    pub config: VmConfig,
}

/// Template service for managing VM templates
pub struct TemplateService {
    templates_dir: PathBuf,
}

impl TemplateService {
    /// Create a new TemplateService instance
    pub fn new() -> Result<Self, AppError> {
        let templates_dir = Self::get_templates_dir()?;

        // Create templates directory if it doesn't exist
        if !templates_dir.exists() {
            fs::create_dir_all(&templates_dir)
                .map_err(|e| AppError::TemplateError(format!("Failed to create templates directory: {}", e)))?;
        }

        Ok(Self { templates_dir })
    }

    /// Get the templates directory path
    fn get_templates_dir() -> Result<PathBuf, AppError> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| AppError::TemplateError("Could not determine config directory".to_string()))?;

        Ok(config_dir.join("kvm-manager").join("templates"))
    }

    /// Create a new template
    pub fn create_template(&self, request: CreateTemplateRequest) -> Result<VmTemplate, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        let template = VmTemplate {
            id: id.clone(),
            name: request.name,
            description: request.description,
            config: request.config,
            created_at: now,
            updated_at: now,
        };

        // Save template to file
        self.save_template(&template)?;

        Ok(template)
    }

    /// List all templates
    pub fn list_templates(&self) -> Result<Vec<VmTemplate>, AppError> {
        let mut templates = Vec::new();

        if !self.templates_dir.exists() {
            return Ok(templates);
        }

        let entries = fs::read_dir(&self.templates_dir)
            .map_err(|e| AppError::TemplateError(format!("Failed to read templates directory: {}", e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| AppError::TemplateError(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_template(&path) {
                    Ok(template) => templates.push(template),
                    Err(e) => {
                        eprintln!("Warning: Failed to load template {:?}: {}", path, e);
                    }
                }
            }
        }

        // Sort by created_at descending (newest first)
        templates.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(templates)
    }

    /// Get a template by ID
    pub fn get_template(&self, id: &str) -> Result<VmTemplate, AppError> {
        let path = self.templates_dir.join(format!("{}.json", id));

        if !path.exists() {
            return Err(AppError::TemplateError(format!("Template not found: {}", id)));
        }

        self.load_template(&path)
    }

    /// Update a template
    pub fn update_template(&self, id: &str, request: CreateTemplateRequest) -> Result<VmTemplate, AppError> {
        let mut template = self.get_template(id)?;

        template.name = request.name;
        template.description = request.description;
        template.config = request.config;
        template.updated_at = chrono::Utc::now().timestamp();

        self.save_template(&template)?;

        Ok(template)
    }

    /// Delete a template
    pub fn delete_template(&self, id: &str) -> Result<(), AppError> {
        let path = self.templates_dir.join(format!("{}.json", id));

        if !path.exists() {
            return Err(AppError::TemplateError(format!("Template not found: {}", id)));
        }

        fs::remove_file(&path)
            .map_err(|e| AppError::TemplateError(format!("Failed to delete template: {}", e)))?;

        Ok(())
    }

    /// Save a template to disk
    fn save_template(&self, template: &VmTemplate) -> Result<(), AppError> {
        let path = self.templates_dir.join(format!("{}.json", template.id));

        let json = serde_json::to_string_pretty(template)
            .map_err(|e| AppError::TemplateError(format!("Failed to serialize template: {}", e)))?;

        fs::write(&path, json)
            .map_err(|e| AppError::TemplateError(format!("Failed to write template file: {}", e)))?;

        Ok(())
    }

    /// Load a template from disk
    fn load_template(&self, path: &PathBuf) -> Result<VmTemplate, AppError> {
        let json = fs::read_to_string(path)
            .map_err(|e| AppError::TemplateError(format!("Failed to read template file: {}", e)))?;

        let template: VmTemplate = serde_json::from_str(&json)
            .map_err(|e| AppError::TemplateError(format!("Failed to parse template file: {}", e)))?;

        Ok(template)
    }
}
