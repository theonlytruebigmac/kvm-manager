use serde::{Deserialize, Serialize};
use crate::state::app_state::AppState;
use crate::utils::error::AppError;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum OptimizationCategory {
    Cpu,
    Memory,
    Disk,
    Network,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum OptimizationSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OptimizationSuggestion {
    pub vm_id: String,
    pub vm_name: String,
    pub category: OptimizationCategory,
    pub severity: OptimizationSeverity,
    pub title: String,
    pub description: String,
    pub recommendation: String,
    pub current_value: f64,
    pub threshold: f64,
}

pub struct OptimizationService;

impl OptimizationService {
    /// Analyze VM metrics and generate optimization suggestions
    pub fn analyze_vm_performance(
        app_state: &AppState,
        vm_id: &str,
        vm_name: &str,
        time_range_hours: i64,
    ) -> Result<Vec<OptimizationSuggestion>, AppError> {
        let end_time = chrono::Utc::now().timestamp();
        let start_time = end_time - (time_range_hours * 3600);

        let metrics = app_state.metrics.get_historical_metrics(
            vm_id,
            start_time,
            end_time,
            None,
        )?;

        let mut suggestions = Vec::new();

        if metrics.data_points.is_empty() {
            return Ok(suggestions);
        }

        // Analyze CPU usage
        let avg_cpu = metrics.data_points.iter()
            .map(|dp| dp.cpu_usage)
            .sum::<f64>() / metrics.data_points.len() as f64;

        let max_cpu = metrics.data_points.iter()
            .map(|dp| dp.cpu_usage)
            .fold(0.0, f64::max);        if avg_cpu < 10.0 {
            suggestions.push(OptimizationSuggestion {
                vm_id: vm_id.to_string(),
                vm_name: vm_name.to_string(),
                category: OptimizationCategory::Cpu,
                severity: OptimizationSeverity::Info,
                title: "Low CPU utilization".to_string(),
                description: format!("Average CPU usage is {:.1}% over the last {} hours", avg_cpu, time_range_hours),
                recommendation: "Consider reducing CPU count or consolidating workloads".to_string(),
                current_value: avg_cpu,
                threshold: 10.0,
            });
        } else if avg_cpu > 80.0 {
            suggestions.push(OptimizationSuggestion {
                vm_id: vm_id.to_string(),
                vm_name: vm_name.to_string(),
                category: OptimizationCategory::Cpu,
                severity: if avg_cpu > 95.0 { OptimizationSeverity::Critical } else { OptimizationSeverity::Warning },
                title: "High CPU utilization".to_string(),
                description: format!("Average CPU usage is {:.1}% over the last {} hours", avg_cpu, time_range_hours),
                recommendation: "Consider increasing CPU count or optimizing workload".to_string(),
                current_value: avg_cpu,
                threshold: 80.0,
            });
        }

        if max_cpu > 95.0 {
            suggestions.push(OptimizationSuggestion {
                vm_id: vm_id.to_string(),
                vm_name: vm_name.to_string(),
                category: OptimizationCategory::Cpu,
                severity: OptimizationSeverity::Warning,
                title: "CPU spikes detected".to_string(),
                description: format!("CPU reached {:.1}% utilization", max_cpu),
                recommendation: "Monitor for CPU-intensive processes or consider increasing CPU resources".to_string(),
                current_value: max_cpu,
                threshold: 95.0,
            });
        }

        // Analyze memory usage
        let avg_memory_percent = metrics.data_points.iter()
            .map(|dp| {
                let total = dp.memory_total_mb as f64;
                if total > 0.0 {
                    (dp.memory_usage_mb as f64 / total) * 100.0
                } else {
                    0.0
                }
            })
            .sum::<f64>() / metrics.data_points.len() as f64;

        if avg_memory_percent < 20.0 {
            suggestions.push(OptimizationSuggestion {
                vm_id: vm_id.to_string(),
                vm_name: vm_name.to_string(),
                category: OptimizationCategory::Memory,
                severity: OptimizationSeverity::Info,
                title: "Low memory utilization".to_string(),
                description: format!("Average memory usage is {:.1}% over the last {} hours", avg_memory_percent, time_range_hours),
                recommendation: "Consider reducing allocated memory to free up host resources".to_string(),
                current_value: avg_memory_percent,
                threshold: 20.0,
            });
        } else if avg_memory_percent > 85.0 {
            suggestions.push(OptimizationSuggestion {
                vm_id: vm_id.to_string(),
                vm_name: vm_name.to_string(),
                category: OptimizationCategory::Memory,
                severity: if avg_memory_percent > 95.0 { OptimizationSeverity::Critical } else { OptimizationSeverity::Warning },
                title: "High memory utilization".to_string(),
                description: format!("Average memory usage is {:.1}% over the last {} hours", avg_memory_percent, time_range_hours),
                recommendation: "Consider increasing allocated memory or optimizing memory usage".to_string(),
                current_value: avg_memory_percent,
                threshold: 85.0,
            });
        }

        // Analyze disk I/O
        let total_disk_read: f64 = metrics.data_points.iter()
            .map(|dp| dp.disk_read_bytes as f64)
            .sum();

        let total_disk_write: f64 = metrics.data_points.iter()
            .map(|dp| dp.disk_write_bytes as f64)
            .sum();        let disk_read_mb_per_sec = (total_disk_read / (1024.0 * 1024.0)) / (time_range_hours as f64 * 3600.0);
        let disk_write_mb_per_sec = (total_disk_write / (1024.0 * 1024.0)) / (time_range_hours as f64 * 3600.0);

        if disk_write_mb_per_sec > 50.0 {
            suggestions.push(OptimizationSuggestion {
                vm_id: vm_id.to_string(),
                vm_name: vm_name.to_string(),
                category: OptimizationCategory::Disk,
                severity: OptimizationSeverity::Warning,
                title: "High disk write activity".to_string(),
                description: format!("Average disk write speed is {:.2} MB/s", disk_write_mb_per_sec),
                recommendation: "Consider using faster storage or optimizing write operations".to_string(),
                current_value: disk_write_mb_per_sec,
                threshold: 50.0,
            });
        }

        if disk_read_mb_per_sec > 100.0 {
            suggestions.push(OptimizationSuggestion {
                vm_id: vm_id.to_string(),
                vm_name: vm_name.to_string(),
                category: OptimizationCategory::Disk,
                severity: OptimizationSeverity::Warning,
                title: "High disk read activity".to_string(),
                description: format!("Average disk read speed is {:.2} MB/s", disk_read_mb_per_sec),
                recommendation: "Consider adding caching or using faster storage".to_string(),
                current_value: disk_read_mb_per_sec,
                threshold: 100.0,
            });
        }

        // Analyze network I/O
        let total_network_rx: f64 = metrics.data_points.iter()
            .map(|dp| dp.network_rx_bytes as f64)
            .sum();

        let total_network_tx: f64 = metrics.data_points.iter()
            .map(|dp| dp.network_tx_bytes as f64)
            .sum();        let network_rx_mbps = ((total_network_rx * 8.0) / (1024.0 * 1024.0)) / (time_range_hours as f64 * 3600.0);
        let network_tx_mbps = ((total_network_tx * 8.0) / (1024.0 * 1024.0)) / (time_range_hours as f64 * 3600.0);

        if network_rx_mbps > 800.0 {
            suggestions.push(OptimizationSuggestion {
                vm_id: vm_id.to_string(),
                vm_name: vm_name.to_string(),
                category: OptimizationCategory::Network,
                severity: OptimizationSeverity::Warning,
                title: "High network receive traffic".to_string(),
                description: format!("Average network receive speed is {:.2} Mbps", network_rx_mbps),
                recommendation: "Monitor network bandwidth and consider upgrading network configuration".to_string(),
                current_value: network_rx_mbps,
                threshold: 800.0,
            });
        }

        if network_tx_mbps > 800.0 {
            suggestions.push(OptimizationSuggestion {
                vm_id: vm_id.to_string(),
                vm_name: vm_name.to_string(),
                category: OptimizationCategory::Network,
                severity: OptimizationSeverity::Warning,
                title: "High network transmit traffic".to_string(),
                description: format!("Average network transmit speed is {:.2} Mbps", network_tx_mbps),
                recommendation: "Monitor network bandwidth and consider upgrading network configuration".to_string(),
                current_value: network_tx_mbps,
                threshold: 800.0,
            });
        }

        Ok(suggestions)
    }

    /// Analyze all VMs and generate optimization suggestions
    pub fn analyze_all_vms(
        app_state: &AppState,
        time_range_hours: i64,
    ) -> Result<Vec<OptimizationSuggestion>, AppError> {
        use crate::services::vm_service::VmService;

        let vms = VmService::list_vms(&app_state.libvirt)?;
        let mut all_suggestions = Vec::new();

        for vm in vms {
            match Self::analyze_vm_performance(app_state, &vm.id, &vm.name, time_range_hours) {
                Ok(suggestions) => all_suggestions.extend(suggestions),
                Err(e) => {
                    tracing::warn!("Failed to analyze VM {}: {}", vm.name, e);
                }
            }
        }

        Ok(all_suggestions)
    }
}
