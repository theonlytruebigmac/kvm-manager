use crate::utils::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Alert threshold type
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ThresholdType {
    Cpu,
    Memory,
    Disk,
    Network,
}

/// Alert severity level
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Resource alert configuration
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResourceAlert {
    pub id: String,
    pub name: String,
    pub vm_id: String,
    pub threshold_type: ThresholdType,
    pub threshold_value: f64, // Percentage (0-100) or absolute value
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub consecutive_checks: u32, // Number of consecutive checks before alerting
    pub current_trigger_count: u32,
    pub last_triggered: Option<i64>,
    pub created_at: i64,
}

/// Request to create an alert
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateAlertRequest {
    pub name: String,
    pub vm_id: String,
    pub threshold_type: ThresholdType,
    pub threshold_value: f64,
    pub severity: AlertSeverity,
    pub consecutive_checks: u32,
}

/// Alert event when threshold is exceeded
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AlertEvent {
    pub alert_id: String,
    pub alert_name: String,
    pub vm_id: String,
    pub threshold_type: ThresholdType,
    pub threshold_value: f64,
    pub current_value: f64,
    pub severity: AlertSeverity,
    pub timestamp: i64,
}

/// Alert service for managing resource alerts
pub struct AlertService {
    alerts_dir: PathBuf,
}

impl AlertService {
    /// Create a new AlertService instance
    pub fn new() -> Result<Self, AppError> {
        let alerts_dir = Self::get_alerts_dir()?;

        // Create alerts directory if it doesn't exist
        if !alerts_dir.exists() {
            fs::create_dir_all(&alerts_dir)
                .map_err(|e| AppError::AlertError(format!("Failed to create alerts directory: {}", e)))?;
        }

        Ok(Self { alerts_dir })
    }

    /// Get the alerts directory path
    fn get_alerts_dir() -> Result<PathBuf, AppError> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| AppError::AlertError("Could not determine config directory".to_string()))?;

        Ok(config_dir.join("kvm-manager").join("alerts"))
    }

    /// Create a new resource alert
    pub fn create_alert(&self, request: CreateAlertRequest) -> Result<ResourceAlert, AppError> {
        // Validate threshold value
        if request.threshold_value < 0.0 || request.threshold_value > 100.0 {
            return Err(AppError::AlertError(
                "Threshold value must be between 0 and 100".to_string(),
            ));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        let alert = ResourceAlert {
            id: id.clone(),
            name: request.name,
            vm_id: request.vm_id,
            threshold_type: request.threshold_type,
            threshold_value: request.threshold_value,
            severity: request.severity,
            enabled: true,
            consecutive_checks: request.consecutive_checks.max(1),
            current_trigger_count: 0,
            last_triggered: None,
            created_at: now,
        };

        // Save alert to file
        self.save_alert(&alert)?;

        Ok(alert)
    }

    /// List all alerts
    pub fn list_alerts(&self) -> Result<Vec<ResourceAlert>, AppError> {
        let mut alerts = Vec::new();

        if !self.alerts_dir.exists() {
            return Ok(alerts);
        }

        let entries = fs::read_dir(&self.alerts_dir)
            .map_err(|e| AppError::AlertError(format!("Failed to read alerts directory: {}", e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| AppError::AlertError(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_alert(&path) {
                    Ok(alert) => alerts.push(alert),
                    Err(e) => {
                        eprintln!("Warning: Failed to load alert {:?}: {}", path, e);
                    }
                }
            }
        }

        // Sort by created_at descending
        alerts.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(alerts)
    }

    /// Get an alert by ID
    pub fn get_alert(&self, id: &str) -> Result<ResourceAlert, AppError> {
        let path = self.alerts_dir.join(format!("{}.json", id));

        if !path.exists() {
            return Err(AppError::AlertError(format!("Alert not found: {}", id)));
        }

        self.load_alert(&path)
    }

    /// Update alert enabled status
    pub fn update_alert_status(&self, id: &str, enabled: bool) -> Result<ResourceAlert, AppError> {
        let mut alert = self.get_alert(id)?;
        alert.enabled = enabled;

        // Reset trigger count when disabling
        if !enabled {
            alert.current_trigger_count = 0;
        }

        self.save_alert(&alert)?;
        Ok(alert)
    }

    /// Delete an alert
    pub fn delete_alert(&self, id: &str) -> Result<(), AppError> {
        let path = self.alerts_dir.join(format!("{}.json", id));

        if !path.exists() {
            return Err(AppError::AlertError(format!("Alert not found: {}", id)));
        }

        fs::remove_file(&path)
            .map_err(|e| AppError::AlertError(format!("Failed to delete alert: {}", e)))?;

        Ok(())
    }

    /// Get alerts for a specific VM
    pub fn get_vm_alerts(&self, vm_id: &str) -> Result<Vec<ResourceAlert>, AppError> {
        let alerts = self.list_alerts()?;
        Ok(alerts.into_iter().filter(|a| a.vm_id == vm_id).collect())
    }

    /// Check if a value exceeds threshold and update alert state
    pub fn check_threshold(
        &self,
        alert_id: &str,
        current_value: f64,
    ) -> Result<Option<AlertEvent>, AppError> {
        let mut alert = self.get_alert(alert_id)?;

        if !alert.enabled {
            return Ok(None);
        }

        // Check if threshold is exceeded
        let exceeded = current_value >= alert.threshold_value;

        if exceeded {
            alert.current_trigger_count += 1;

            // Trigger alert if consecutive checks threshold is met
            if alert.current_trigger_count >= alert.consecutive_checks {
                let now = chrono::Utc::now().timestamp();
                alert.last_triggered = Some(now);
                alert.current_trigger_count = 0; // Reset after triggering

                self.save_alert(&alert)?;

                return Ok(Some(AlertEvent {
                    alert_id: alert.id,
                    alert_name: alert.name,
                    vm_id: alert.vm_id,
                    threshold_type: alert.threshold_type,
                    threshold_value: alert.threshold_value,
                    current_value,
                    severity: alert.severity,
                    timestamp: now,
                }));
            }

            self.save_alert(&alert)?;
        } else {
            // Reset trigger count if threshold not exceeded
            if alert.current_trigger_count > 0 {
                alert.current_trigger_count = 0;
                self.save_alert(&alert)?;
            }
        }

        Ok(None)
    }

    /// Save an alert to disk
    fn save_alert(&self, alert: &ResourceAlert) -> Result<(), AppError> {
        let path = self.alerts_dir.join(format!("{}.json", alert.id));

        let json = serde_json::to_string_pretty(alert)
            .map_err(|e| AppError::AlertError(format!("Failed to serialize alert: {}", e)))?;

        fs::write(&path, json)
            .map_err(|e| AppError::AlertError(format!("Failed to write alert file: {}", e)))?;

        Ok(())
    }

    /// Load an alert from disk
    fn load_alert(&self, path: &PathBuf) -> Result<ResourceAlert, AppError> {
        let json = fs::read_to_string(path)
            .map_err(|e| AppError::AlertError(format!("Failed to read alert file: {}", e)))?;

        let alert: ResourceAlert = serde_json::from_str(&json)
            .map_err(|e| AppError::AlertError(format!("Failed to parse alert file: {}", e)))?;

        Ok(alert)
    }
}
