use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use chrono::{Datelike, Timelike};
use crate::services::metrics_service::MetricsService;
use crate::utils::error::AppError;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RetentionPolicy {
    pub enabled: bool,
    pub retention_days: i64,
    pub cleanup_hour: u32, // Hour of day to run cleanup (0-23)
    pub last_cleanup: Option<i64>,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: 30,
            cleanup_hour: 2, // 2 AM
            last_cleanup: None,
        }
    }
}

pub struct RetentionService {
    config_path: PathBuf,
    metrics_service: Arc<MetricsService>,
}

impl RetentionService {
    /// Create a new retention service
    pub fn new(metrics_service: Arc<MetricsService>) -> Result<Self, AppError> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| AppError::InvalidConfig("Could not determine config directory".to_string()))?
            .join("kvm-manager");

        fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("retention_policy.json");

        Ok(Self {
            config_path,
            metrics_service,
        })
    }

    /// Load retention policy from disk
    pub fn load_policy(&self) -> Result<RetentionPolicy, AppError> {
        if !self.config_path.exists() {
            return Ok(RetentionPolicy::default());
        }

        let contents = fs::read_to_string(&self.config_path)?;

        serde_json::from_str(&contents)
            .map_err(|e| AppError::InvalidConfig(format!("Failed to parse retention policy: {}", e)))
    }

    /// Save retention policy to disk
    pub fn save_policy(&self, policy: &RetentionPolicy) -> Result<(), AppError> {
        let json = serde_json::to_string_pretty(policy)
            .map_err(|e| AppError::InvalidConfig(format!("Failed to serialize policy: {}", e)))?;

        fs::write(&self.config_path, json)?;

        Ok(())
    }

    /// Update retention policy
    pub fn update_policy(&self, policy: RetentionPolicy) -> Result<RetentionPolicy, AppError> {
        self.save_policy(&policy)?;
        Ok(policy)
    }

    /// Execute cleanup based on retention policy
    pub fn execute_cleanup(&self) -> Result<usize, AppError> {
        let policy = self.load_policy()?;

        if !policy.enabled {
            tracing::info!("Retention policy is disabled, skipping cleanup");
            return Ok(0);
        }

        let cutoff_time = chrono::Utc::now().timestamp() - (policy.retention_days * 24 * 3600);

        tracing::info!(
            "Executing retention policy cleanup: removing metrics older than {} days (cutoff: {})",
            policy.retention_days,
            cutoff_time
        );

        let deleted_count = self.metrics_service.cleanup_old_metrics(cutoff_time)?;

        // Update last cleanup time
        let mut updated_policy = policy;
        updated_policy.last_cleanup = Some(chrono::Utc::now().timestamp());
        self.save_policy(&updated_policy)?;

        tracing::info!("Retention policy cleanup completed: deleted {} records", deleted_count);

        Ok(deleted_count)
    }

    /// Check if cleanup should run based on schedule
    pub fn should_run_cleanup(&self) -> Result<bool, AppError> {
        let policy = self.load_policy()?;

        if !policy.enabled {
            return Ok(false);
        }

        // Check if we've never run cleanup
        if policy.last_cleanup.is_none() {
            return Ok(true);
        }

        let now = chrono::Utc::now();
        let current_hour = now.hour();

        // Only run during the configured hour
        if current_hour != policy.cleanup_hour {
            return Ok(false);
        }

        // Check if we already ran today
        if let Some(last_cleanup) = policy.last_cleanup {
            let last_cleanup_date = chrono::DateTime::from_timestamp(last_cleanup, 0)
                .ok_or_else(|| AppError::InvalidConfig("Invalid last_cleanup timestamp".to_string()))?;

            let last_cleanup_day = last_cleanup_date.ordinal();
            let current_day = now.ordinal();

            // If we already ran today, skip
            if last_cleanup_day == current_day {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Start background cleanup task
    pub async fn start_cleanup_task(self: Arc<Self>) {
        tracing::info!("Starting retention policy cleanup task");

        tokio::spawn(async move {
            loop {
                // Check every hour
                sleep(Duration::from_secs(3600)).await;

                match self.should_run_cleanup() {
                    Ok(true) => {
                        tracing::info!("Scheduled cleanup time reached, executing cleanup");
                        match self.execute_cleanup() {
                            Ok(count) => {
                                tracing::info!("Scheduled cleanup completed: deleted {} records", count);
                            }
                            Err(e) => {
                                tracing::error!("Scheduled cleanup failed: {}", e);
                            }
                        }
                    }
                    Ok(false) => {
                        // Not time to run yet
                    }
                    Err(e) => {
                        tracing::error!("Failed to check cleanup schedule: {}", e);
                    }
                }
            }
        });
    }
}
