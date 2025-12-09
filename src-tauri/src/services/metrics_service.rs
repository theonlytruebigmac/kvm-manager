use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crate::utils::error::AppError;

/// VM performance metrics for historical tracking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VmMetrics {
    pub vm_id: String,
    pub timestamp: i64,
    pub cpu_usage: f64,
    pub memory_usage_mb: u64,
    pub memory_total_mb: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}

/// Historical metrics query result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoricalMetrics {
    pub vm_id: String,
    pub data_points: Vec<MetricDataPoint>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricDataPoint {
    pub timestamp: i64,
    pub cpu_usage: f64,
    pub memory_usage_mb: u64,
    pub memory_total_mb: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}

/// MetricsService handles persistence of VM performance metrics
pub struct MetricsService {
    conn: Arc<Mutex<Connection>>,
}

impl MetricsService {
    /// Initialize the metrics service with SQLite database
    pub fn new(db_path: Option<PathBuf>) -> Result<Self, AppError> {
        let path = db_path.unwrap_or_else(|| {
            let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("kvm-manager");
            std::fs::create_dir_all(&path).ok();
            path.push("metrics.db");
            path
        });

        tracing::info!("Initializing metrics database at: {:?}", path);

        let conn = Connection::open(&path)
            .map_err(|e| AppError::Other(format!("Failed to open metrics database: {}", e)))?;

        // Create metrics table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS vm_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                vm_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                cpu_usage REAL NOT NULL,
                memory_usage_mb INTEGER NOT NULL,
                memory_total_mb INTEGER NOT NULL,
                disk_read_bytes INTEGER NOT NULL,
                disk_write_bytes INTEGER NOT NULL,
                network_rx_bytes INTEGER NOT NULL,
                network_tx_bytes INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| AppError::Other(format!("Failed to create metrics table: {}", e)))?;

        // Create index for efficient queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_vm_timestamp ON vm_metrics(vm_id, timestamp)",
            [],
        )
        .map_err(|e| AppError::Other(format!("Failed to create index: {}", e)))?;

        tracing::info!("Metrics database initialized successfully");

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Store VM metrics
    pub fn store_metrics(&self, metrics: &VmMetrics) -> Result<(), AppError> {
        let conn = self.conn.lock()
            .map_err(|e| AppError::Other(format!("Failed to lock database: {}", e)))?;

        conn.execute(
            "INSERT INTO vm_metrics (
                vm_id, timestamp, cpu_usage, memory_usage_mb, memory_total_mb,
                disk_read_bytes, disk_write_bytes, network_rx_bytes, network_tx_bytes
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                metrics.vm_id,
                metrics.timestamp,
                metrics.cpu_usage,
                metrics.memory_usage_mb,
                metrics.memory_total_mb,
                metrics.disk_read_bytes,
                metrics.disk_write_bytes,
                metrics.network_rx_bytes,
                metrics.network_tx_bytes,
            ],
        )
        .map_err(|e| AppError::Other(format!("Failed to insert metrics: {}", e)))?;

        Ok(())
    }

    /// Get historical metrics for a VM within a time range
    pub fn get_historical_metrics(
        &self,
        vm_id: &str,
        start_time: i64,
        end_time: i64,
        max_points: Option<usize>,
    ) -> Result<HistoricalMetrics, AppError> {
        let conn = self.conn.lock()
            .map_err(|e| AppError::Other(format!("Failed to lock database: {}", e)))?;

        // Calculate sampling interval if max_points is specified
        let query = if let Some(max_points) = max_points {
            // Get total count to determine sampling interval
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM vm_metrics WHERE vm_id = ?1 AND timestamp >= ?2 AND timestamp <= ?3",
                params![vm_id, start_time, end_time],
                |row| row.get(0),
            )
            .unwrap_or(0);

            let interval = if count > max_points as i64 {
                count / max_points as i64
            } else {
                1
            };

            format!(
                "SELECT timestamp, cpu_usage, memory_usage_mb, memory_total_mb,
                        disk_read_bytes, disk_write_bytes, network_rx_bytes, network_tx_bytes
                 FROM vm_metrics
                 WHERE vm_id = ?1 AND timestamp >= ?2 AND timestamp <= ?3
                 AND id % {} = 0
                 ORDER BY timestamp ASC",
                interval
            )
        } else {
            "SELECT timestamp, cpu_usage, memory_usage_mb, memory_total_mb,
                    disk_read_bytes, disk_write_bytes, network_rx_bytes, network_tx_bytes
             FROM vm_metrics
             WHERE vm_id = ?1 AND timestamp >= ?2 AND timestamp <= ?3
             ORDER BY timestamp ASC".to_string()
        };

        let mut stmt = conn.prepare(&query)
            .map_err(|e| AppError::Other(format!("Failed to prepare query: {}", e)))?;

        let data_points = stmt.query_map(
            params![vm_id, start_time, end_time],
            |row| {
                Ok(MetricDataPoint {
                    timestamp: row.get(0)?,
                    cpu_usage: row.get(1)?,
                    memory_usage_mb: row.get(2)?,
                    memory_total_mb: row.get(3)?,
                    disk_read_bytes: row.get(4)?,
                    disk_write_bytes: row.get(5)?,
                    network_rx_bytes: row.get(6)?,
                    network_tx_bytes: row.get(7)?,
                })
            },
        )
        .map_err(|e| AppError::Other(format!("Failed to query metrics: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Other(format!("Failed to collect metrics: {}", e)))?;

        Ok(HistoricalMetrics {
            vm_id: vm_id.to_string(),
            data_points,
        })
    }

    /// Delete old metrics older than the specified timestamp
    pub fn cleanup_old_metrics(&self, older_than: i64) -> Result<usize, AppError> {
        let conn = self.conn.lock()
            .map_err(|e| AppError::Other(format!("Failed to lock database: {}", e)))?;

        let deleted = conn.execute(
            "DELETE FROM vm_metrics WHERE timestamp < ?1",
            params![older_than],
        )
        .map_err(|e| AppError::Other(format!("Failed to cleanup metrics: {}", e)))?;

        tracing::info!("Deleted {} old metric records", deleted);
        Ok(deleted)
    }

    /// Get the total number of metrics stored
    pub fn get_metrics_count(&self) -> Result<i64, AppError> {
        let conn = self.conn.lock()
            .map_err(|e| AppError::Other(format!("Failed to lock database: {}", e)))?;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM vm_metrics",
            [],
            |row| row.get(0),
        )
        .map_err(|e| AppError::Other(format!("Failed to count metrics: {}", e)))?;

        Ok(count)
    }
}
