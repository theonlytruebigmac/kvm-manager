use crate::services::scheduler_service::{OperationType, ScheduleFrequency, SchedulerService};
use crate::utils::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Backup configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BackupConfig {
    pub id: String,
    pub name: String,
    pub vm_id: String,
    pub schedule_id: String, // Link to scheduler service
    pub retention_count: u32, // Number of backups to keep
    pub enabled: bool,
    pub last_backup: Option<i64>,
    pub backup_count: u32,
    pub created_at: i64,
}

/// Request to create a backup configuration
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateBackupRequest {
    pub name: String,
    pub vm_id: String,
    pub frequency: ScheduleFrequency,
    pub scheduled_time: String,
    pub day_of_week: Option<u8>,
    pub day_of_month: Option<u8>,
    pub retention_count: u32,
}

/// Backup service
pub struct BackupService {
    backups_dir: PathBuf,
}

impl BackupService {
    /// Create a new BackupService instance
    pub fn new() -> Result<Self, AppError> {
        let backups_dir = Self::get_backups_dir()?;

        // Create backups directory if it doesn't exist
        if !backups_dir.exists() {
            fs::create_dir_all(&backups_dir)
                .map_err(|e| AppError::Other(format!("Failed to create backups directory: {}", e)))?;
        }

        Ok(Self { backups_dir })
    }

    /// Get the backups directory path
    fn get_backups_dir() -> Result<PathBuf, AppError> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| AppError::Other("Could not determine config directory".to_string()))?;

        Ok(config_dir.join("kvm-manager").join("backups"))
    }

    /// Create a new backup configuration
    pub fn create_backup_config(&self, request: CreateBackupRequest) -> Result<BackupConfig, AppError> {
        // Validate retention count
        if request.retention_count == 0 {
            return Err(AppError::Other("Retention count must be at least 1".to_string()));
        }

        // Create a schedule for the backup using the scheduler service
        let scheduler = SchedulerService::new()?;
        let schedule_request = crate::services::scheduler_service::CreateScheduleRequest {
            name: format!("Backup: {}", request.name),
            vm_id: request.vm_id.clone(),
            operation: OperationType::Snapshot,
            frequency: request.frequency,
            scheduled_time: request.scheduled_time,
            day_of_week: request.day_of_week,
            day_of_month: request.day_of_month,
        };
        let schedule = scheduler.create_schedule(schedule_request)?;

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        let backup_config = BackupConfig {
            id: id.clone(),
            name: request.name,
            vm_id: request.vm_id,
            schedule_id: schedule.id,
            retention_count: request.retention_count,
            enabled: true,
            last_backup: None,
            backup_count: 0,
            created_at: now,
        };

        // Save backup config to file
        self.save_backup_config(&backup_config)?;

        Ok(backup_config)
    }

    /// List all backup configurations
    pub fn list_backup_configs(&self) -> Result<Vec<BackupConfig>, AppError> {
        let mut configs = Vec::new();

        if !self.backups_dir.exists() {
            return Ok(configs);
        }

        let entries = fs::read_dir(&self.backups_dir)
            .map_err(|e| AppError::Other(format!("Failed to read backups directory: {}", e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| AppError::Other(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_backup_config(&path) {
                    Ok(config) => configs.push(config),
                    Err(e) => {
                        eprintln!("Warning: Failed to load backup config {:?}: {}", path, e);
                    }
                }
            }
        }

        // Sort by created_at descending
        configs.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(configs)
    }

    /// Get a backup configuration by ID
    pub fn get_backup_config(&self, id: &str) -> Result<BackupConfig, AppError> {
        let path = self.backups_dir.join(format!("{}.json", id));

        if !path.exists() {
            return Err(AppError::Other(format!("Backup config not found: {}", id)));
        }

        self.load_backup_config(&path)
    }

    /// Update backup enabled status
    pub fn update_backup_status(&self, id: &str, enabled: bool) -> Result<BackupConfig, AppError> {
        let mut config = self.get_backup_config(id)?;
        config.enabled = enabled;

        // Also update the associated schedule
        let scheduler = SchedulerService::new()?;
        scheduler.update_schedule_status(&config.schedule_id, enabled)?;

        self.save_backup_config(&config)?;
        Ok(config)
    }

    /// Delete a backup configuration
    pub fn delete_backup_config(&self, id: &str) -> Result<(), AppError> {
        let config = self.get_backup_config(id)?;

        // Delete the associated schedule
        let scheduler = SchedulerService::new()?;
        scheduler.delete_schedule(&config.schedule_id)?;

        // Delete the backup config
        let path = self.backups_dir.join(format!("{}.json", id));
        fs::remove_file(&path)
            .map_err(|e| AppError::Other(format!("Failed to delete backup config: {}", e)))?;

        Ok(())
    }

    /// Get backup configurations for a specific VM
    pub fn get_vm_backup_configs(&self, vm_id: &str) -> Result<Vec<BackupConfig>, AppError> {
        let configs = self.list_backup_configs()?;
        Ok(configs.into_iter().filter(|c| c.vm_id == vm_id).collect())
    }

    /// Record a backup execution
    pub fn record_backup(&self, id: &str) -> Result<BackupConfig, AppError> {
        let mut config = self.get_backup_config(id)?;

        let now = chrono::Utc::now().timestamp();
        config.last_backup = Some(now);
        config.backup_count += 1;

        self.save_backup_config(&config)?;
        Ok(config)
    }

    /// Save a backup config to disk
    fn save_backup_config(&self, config: &BackupConfig) -> Result<(), AppError> {
        let path = self.backups_dir.join(format!("{}.json", config.id));

        let json = serde_json::to_string_pretty(config)
            .map_err(|e| AppError::Other(format!("Failed to serialize backup config: {}", e)))?;

        fs::write(&path, json)
            .map_err(|e| AppError::Other(format!("Failed to write backup config file: {}", e)))?;

        Ok(())
    }

    /// Load a backup config from disk
    fn load_backup_config(&self, path: &PathBuf) -> Result<BackupConfig, AppError> {
        let json = fs::read_to_string(path)
            .map_err(|e| AppError::Other(format!("Failed to read backup config file: {}", e)))?;

        let config: BackupConfig = serde_json::from_str(&json)
            .map_err(|e| AppError::Other(format!("Failed to parse backup config file: {}", e)))?;

        Ok(config)
    }
}
