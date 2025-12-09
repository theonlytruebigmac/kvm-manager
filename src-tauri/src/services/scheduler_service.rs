use crate::utils::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Schedule frequency type
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScheduleFrequency {
    Once,
    Daily,
    Weekly,
    Monthly,
}

/// Scheduled operation type
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OperationType {
    Start,
    Stop,
    Reboot,
    Snapshot,
}

/// Scheduled operation
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledOperation {
    pub id: String,
    pub name: String,
    pub vm_id: String,
    pub operation: OperationType,
    pub frequency: ScheduleFrequency,
    pub scheduled_time: String, // Format: "HH:MM" for time of day
    pub day_of_week: Option<u8>, // 0-6 for weekly (0=Sunday)
    pub day_of_month: Option<u8>, // 1-31 for monthly
    pub enabled: bool,
    pub last_run: Option<i64>,
    pub next_run: i64,
    pub created_at: i64,
}

/// Request to create a scheduled operation
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateScheduleRequest {
    pub name: String,
    pub vm_id: String,
    pub operation: OperationType,
    pub frequency: ScheduleFrequency,
    pub scheduled_time: String,
    pub day_of_week: Option<u8>,
    pub day_of_month: Option<u8>,
}

/// Scheduler service
pub struct SchedulerService {
    schedules_dir: PathBuf,
}

impl SchedulerService {
    /// Create a new SchedulerService instance
    pub fn new() -> Result<Self, AppError> {
        let schedules_dir = Self::get_schedules_dir()?;

        // Create schedules directory if it doesn't exist
        if !schedules_dir.exists() {
            fs::create_dir_all(&schedules_dir)
                .map_err(|e| AppError::ScheduleError(format!("Failed to create schedules directory: {}", e)))?;
        }

        Ok(Self { schedules_dir })
    }

    /// Get the schedules directory path
    fn get_schedules_dir() -> Result<PathBuf, AppError> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| AppError::ScheduleError("Could not determine config directory".to_string()))?;

        Ok(config_dir.join("kvm-manager").join("schedules"))
    }

    /// Create a new scheduled operation
    pub fn create_schedule(&self, request: CreateScheduleRequest) -> Result<ScheduledOperation, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        let next_run = self.calculate_next_run(
            &request.frequency,
            &request.scheduled_time,
            request.day_of_week,
            request.day_of_month,
        )?;

        let schedule = ScheduledOperation {
            id: id.clone(),
            name: request.name,
            vm_id: request.vm_id,
            operation: request.operation,
            frequency: request.frequency,
            scheduled_time: request.scheduled_time,
            day_of_week: request.day_of_week,
            day_of_month: request.day_of_month,
            enabled: true,
            last_run: None,
            next_run,
            created_at: now,
        };

        // Save schedule to file
        self.save_schedule(&schedule)?;

        Ok(schedule)
    }

    /// List all schedules
    pub fn list_schedules(&self) -> Result<Vec<ScheduledOperation>, AppError> {
        let mut schedules = Vec::new();

        if !self.schedules_dir.exists() {
            return Ok(schedules);
        }

        let entries = fs::read_dir(&self.schedules_dir)
            .map_err(|e| AppError::ScheduleError(format!("Failed to read schedules directory: {}", e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| AppError::ScheduleError(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_schedule(&path) {
                    Ok(schedule) => schedules.push(schedule),
                    Err(e) => {
                        eprintln!("Warning: Failed to load schedule {:?}: {}", path, e);
                    }
                }
            }
        }

        // Sort by next_run ascending
        schedules.sort_by(|a, b| a.next_run.cmp(&b.next_run));

        Ok(schedules)
    }

    /// Get a schedule by ID
    pub fn get_schedule(&self, id: &str) -> Result<ScheduledOperation, AppError> {
        let path = self.schedules_dir.join(format!("{}.json", id));

        if !path.exists() {
            return Err(AppError::ScheduleError(format!("Schedule not found: {}", id)));
        }

        self.load_schedule(&path)
    }

    /// Update schedule enabled status
    pub fn update_schedule_status(&self, id: &str, enabled: bool) -> Result<ScheduledOperation, AppError> {
        let mut schedule = self.get_schedule(id)?;
        schedule.enabled = enabled;

        // Recalculate next run if enabling
        if enabled {
            schedule.next_run = self.calculate_next_run(
                &schedule.frequency,
                &schedule.scheduled_time,
                schedule.day_of_week,
                schedule.day_of_month,
            )?;
        }

        self.save_schedule(&schedule)?;
        Ok(schedule)
    }

    /// Delete a schedule
    pub fn delete_schedule(&self, id: &str) -> Result<(), AppError> {
        let path = self.schedules_dir.join(format!("{}.json", id));

        if !path.exists() {
            return Err(AppError::ScheduleError(format!("Schedule not found: {}", id)));
        }

        fs::remove_file(&path)
            .map_err(|e| AppError::ScheduleError(format!("Failed to delete schedule: {}", e)))?;

        Ok(())
    }

    /// Get schedules for a specific VM
    pub fn get_vm_schedules(&self, vm_id: &str) -> Result<Vec<ScheduledOperation>, AppError> {
        let schedules = self.list_schedules()?;
        Ok(schedules.into_iter().filter(|s| s.vm_id == vm_id).collect())
    }

    /// Calculate next run timestamp
    fn calculate_next_run(
        &self,
        frequency: &ScheduleFrequency,
        scheduled_time: &str,
        day_of_week: Option<u8>,
        day_of_month: Option<u8>,
    ) -> Result<i64, AppError> {
        use chrono::{Datelike, Duration, NaiveTime};

        // Parse time
        let parts: Vec<&str> = scheduled_time.split(':').collect();
        if parts.len() != 2 {
            return Err(AppError::ScheduleError("Invalid time format. Use HH:MM".to_string()));
        }

        let hour: u32 = parts[0].parse()
            .map_err(|_| AppError::ScheduleError("Invalid hour".to_string()))?;
        let minute: u32 = parts[1].parse()
            .map_err(|_| AppError::ScheduleError("Invalid minute".to_string()))?;

        let scheduled = NaiveTime::from_hms_opt(hour, minute, 0)
            .ok_or_else(|| AppError::ScheduleError("Invalid time".to_string()))?;

        let now = chrono::Local::now();
        let today = now.date_naive();
        let current_time = now.time();

        let next_run = match frequency {
            ScheduleFrequency::Once => {
                // If time has passed today, schedule for tomorrow
                let mut target_date = today;
                if current_time > scheduled {
                    target_date = today + Duration::days(1);
                }
                target_date.and_time(scheduled)
            }
            ScheduleFrequency::Daily => {
                // Schedule for today if time hasn't passed, otherwise tomorrow
                let mut target_date = today;
                if current_time > scheduled {
                    target_date = today + Duration::days(1);
                }
                target_date.and_time(scheduled)
            }
            ScheduleFrequency::Weekly => {
                let target_weekday = day_of_week
                    .ok_or_else(|| AppError::ScheduleError("day_of_week required for weekly schedules".to_string()))?;

                let current_weekday = today.weekday().num_days_from_sunday() as u8;
                let days_until_target = if target_weekday >= current_weekday {
                    (target_weekday - current_weekday) as i64
                } else {
                    (7 - current_weekday + target_weekday) as i64
                };

                let mut target_date = today + Duration::days(days_until_target);

                // If it's the same day but time has passed, add a week
                if days_until_target == 0 && current_time > scheduled {
                    target_date = target_date + Duration::days(7);
                }

                target_date.and_time(scheduled)
            }
            ScheduleFrequency::Monthly => {
                let target_day = day_of_month
                    .ok_or_else(|| AppError::ScheduleError("day_of_month required for monthly schedules".to_string()))?;

                let current_day = today.day() as u8;
                let mut target_date = today;

                if target_day < current_day || (target_day == current_day && current_time > scheduled) {
                    // Next month
                    let next_month = if today.month() == 12 {
                        today.with_year(today.year() + 1).and_then(|d| d.with_month(1))
                    } else {
                        today.with_month(today.month() + 1)
                    };
                    target_date = next_month.ok_or_else(|| AppError::ScheduleError("Failed to calculate next month".to_string()))?;
                }

                // Set the day, handling month-end edge cases
                target_date = target_date.with_day(target_day.min(28) as u32)
                    .ok_or_else(|| AppError::ScheduleError("Invalid day of month".to_string()))?;

                target_date.and_time(scheduled)
            }
        };

        Ok(next_run.and_utc().timestamp())
    }

    /// Save a schedule to disk
    fn save_schedule(&self, schedule: &ScheduledOperation) -> Result<(), AppError> {
        let path = self.schedules_dir.join(format!("{}.json", schedule.id));

        let json = serde_json::to_string_pretty(schedule)
            .map_err(|e| AppError::ScheduleError(format!("Failed to serialize schedule: {}", e)))?;

        fs::write(&path, json)
            .map_err(|e| AppError::ScheduleError(format!("Failed to write schedule file: {}", e)))?;

        Ok(())
    }

    /// Load a schedule from disk
    fn load_schedule(&self, path: &PathBuf) -> Result<ScheduledOperation, AppError> {
        let json = fs::read_to_string(path)
            .map_err(|e| AppError::ScheduleError(format!("Failed to read schedule file: {}", e)))?;

        let schedule: ScheduledOperation = serde_json::from_str(&json)
            .map_err(|e| AppError::ScheduleError(format!("Failed to parse schedule file: {}", e)))?;

        Ok(schedule)
    }
}
