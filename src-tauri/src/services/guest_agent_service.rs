//! Guest Agent Service
//!
//! This service communicates with QEMU Guest Agent running inside VMs
//! using libvirt's qemu-agent-command via virsh.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::debug;

/// Guest agent service - uses virsh to communicate with QEMU Guest Agent
pub struct GuestAgentService {}

// ===== Guest Info Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub version: String,
    pub protocol_version: String,
    pub platform: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_type: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub hostname: String,
    pub architecture: String,
    pub uptime_seconds: u64,
    pub cpu_count: u32,
    pub total_memory_kb: u64,
    pub free_memory_kb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub mac_address: Option<String>,
    pub ipv4_addresses: Vec<String>,
    pub ipv6_addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub interfaces: Vec<NetworkInterface>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub mount_point: String,
    pub device: String,
    pub filesystem: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsageInfo {
    pub disks: Vec<DiskInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecCommandResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
}

// ===== New Types for Extended Guest Agent Data =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuStats {
    pub cpu: u32,
    pub user: u64,
    pub system: u64,
    pub idle: u64,
    pub iowait: u64,
    pub steal: u64,
    pub nice: u64,
    pub irq: u64,
    pub softirq: u64,
    pub guest: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuestCpuStats {
    pub cpus: Vec<CpuStats>,
    pub total_user: u64,
    pub total_system: u64,
    pub total_idle: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskStats {
    pub name: String,
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub read_ios: u64,
    pub write_ios: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuestDiskStats {
    pub disks: Vec<DiskStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuestUser {
    pub username: String,
    pub login_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuestTimezone {
    pub zone: String,
    pub offset: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuestFullInfo {
    pub agent_version: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub hostname: String,
    pub architecture: String,
    pub timezone: Option<GuestTimezone>,
    pub users: Vec<GuestUser>,
    pub cpu_count: u32,
    pub filesystems: Vec<DiskInfo>,
    pub network_interfaces: Vec<NetworkInterface>,
}

impl GuestAgentService {
    /// Create a new guest agent service
    pub fn new() -> Self {
        Self {}
    }

    /// Check if a guest agent is available for a VM
    pub async fn is_agent_available(&self, vm_name: &str) -> bool {
        match self.ping(vm_name).await {
            Ok(_) => true,
            Err(e) => {
                debug!("Guest agent not available for '{}': {}", vm_name, e);
                false
            }
        }
    }

    /// Send a command to the guest agent via virsh and get the response
    fn send_agent_command(vm_name: &str, command: &str) -> Result<serde_json::Value> {
        let output = Command::new("virsh")
            .args(["qemu-agent-command", vm_name, command])
            .output()
            .context("Failed to execute virsh command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("virsh qemu-agent-command failed: {}", stderr);
        }

        let response = String::from_utf8_lossy(&output.stdout);
        let parsed: serde_json::Value = serde_json::from_str(&response)
            .context("Failed to parse guest agent response")?;

        // Check for error in response
        if let Some(error) = parsed.get("error") {
            anyhow::bail!("Guest agent error: {}", error);
        }

        Ok(parsed.get("return").cloned().unwrap_or(serde_json::Value::Null))
    }

    /// Ping the guest agent
    pub async fn ping(&self, vm_name: &str) -> Result<bool> {
        let command = r#"{"execute":"guest-ping"}"#;
        Self::send_agent_command(vm_name, command)?;
        Ok(true)
    }

    /// Get agent info
    pub async fn get_agent_info(&self, vm_name: &str) -> Result<AgentInfo> {
        let command = r#"{"execute":"guest-info"}"#;
        let result = Self::send_agent_command(vm_name, command)?;

        let version = result.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        // Extract supported commands as capabilities
        let capabilities: Vec<String> = result.get("supported_commands")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|cmd| {
                        let enabled = cmd.get("enabled").and_then(|e| e.as_bool()).unwrap_or(false);
                        if enabled {
                            cmd.get("name").and_then(|n| n.as_str()).map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(AgentInfo {
            version,
            protocol_version: "1.0".to_string(),
            platform: "qemu-ga".to_string(),
            capabilities,
        })
    }

    /// Get system information
    pub async fn get_system_info(&self, vm_name: &str) -> Result<SystemInfo> {
        // Get OS info
        let os_command = r#"{"execute":"guest-get-osinfo"}"#;
        let os_info = Self::send_agent_command(vm_name, os_command)?;

        // Get hostname
        let hostname_command = r#"{"execute":"guest-get-host-name"}"#;
        let hostname_result = Self::send_agent_command(vm_name, hostname_command)
            .unwrap_or(serde_json::json!({"host-name": "unknown"}));

        // Get VCPUs
        let vcpu_command = r#"{"execute":"guest-get-vcpus"}"#;
        let vcpus = Self::send_agent_command(vm_name, vcpu_command)
            .ok()
            .and_then(|v| v.as_array().map(|a| a.len() as u32))
            .unwrap_or(1);

        let os_name = os_info.get("pretty-name")
            .or_else(|| os_info.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let os_version = os_info.get("version-id")
            .or_else(|| os_info.get("version"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let kernel_version = os_info.get("kernel-release")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let architecture = os_info.get("machine")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let hostname = hostname_result.get("host-name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        // Determine OS type from ID
        let os_id = os_info.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let os_type = if os_id.contains("windows") || os_name.to_lowercase().contains("windows") {
            "windows"
        } else {
            "linux"
        }.to_string();

        Ok(SystemInfo {
            os_type,
            os_name,
            os_version,
            kernel_version,
            hostname,
            architecture,
            uptime_seconds: 0, // Not directly available from qemu-ga
            cpu_count: vcpus,
            total_memory_kb: 0, // Would need guest-exec to get
            free_memory_kb: 0,
        })
    }

    /// Get network information
    pub async fn get_network_info(&self, vm_name: &str) -> Result<NetworkInfo> {
        let command = r#"{"execute":"guest-network-get-interfaces"}"#;
        let result = Self::send_agent_command(vm_name, command)?;

        let interfaces: Vec<NetworkInterface> = result.as_array()
            .map(|arr| {
                arr.iter().map(|iface| {
                    let name = iface.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let mac_address = iface.get("hardware-address")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let mut ipv4_addresses = Vec::new();
                    let mut ipv6_addresses = Vec::new();

                    if let Some(addrs) = iface.get("ip-addresses").and_then(|v| v.as_array()) {
                        for addr in addrs {
                            if let (Some(ip), Some(ip_type)) = (
                                addr.get("ip-address").and_then(|v| v.as_str()),
                                addr.get("ip-address-type").and_then(|v| v.as_str())
                            ) {
                                match ip_type {
                                    "ipv4" => ipv4_addresses.push(ip.to_string()),
                                    "ipv6" => ipv6_addresses.push(ip.to_string()),
                                    _ => {}
                                }
                            }
                        }
                    }

                    NetworkInterface {
                        name,
                        mac_address,
                        ipv4_addresses,
                        ipv6_addresses,
                    }
                }).collect()
            })
            .unwrap_or_default();

        Ok(NetworkInfo { interfaces })
    }

    /// Get disk usage information
    pub async fn get_disk_usage(&self, vm_name: &str) -> Result<DiskUsageInfo> {
        let command = r#"{"execute":"guest-get-fsinfo"}"#;
        let result = Self::send_agent_command(vm_name, command)?;

        let disks: Vec<DiskInfo> = result.as_array()
            .map(|arr| {
                arr.iter().filter_map(|fs| {
                    let mount_point = fs.get("mountpoint")
                        .and_then(|v| v.as_str())?
                        .to_string();

                    let device = fs.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let filesystem = fs.get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let total_bytes = fs.get("total-bytes")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    let used_bytes = fs.get("used-bytes")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    Some(DiskInfo {
                        mount_point,
                        device,
                        filesystem,
                        total_bytes,
                        used_bytes,
                        free_bytes: total_bytes.saturating_sub(used_bytes),
                    })
                }).collect()
            })
            .unwrap_or_default();

        Ok(DiskUsageInfo { disks })
    }

    /// Execute a command in the guest
    pub async fn exec_command(&self, vm_name: &str, command: &str, args: Vec<String>) -> Result<ExecCommandResult> {
        // Build the exec command
        let exec_cmd = serde_json::json!({
            "execute": "guest-exec",
            "arguments": {
                "path": command,
                "arg": args,
                "capture-output": true
            }
        });

        let result = Self::send_agent_command(vm_name, &exec_cmd.to_string())?;

        let pid = result.get("pid")
            .and_then(|v| v.as_i64())
            .context("No PID returned from guest-exec")?;

        // Wait for command to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Get the result
        let status_cmd = serde_json::json!({
            "execute": "guest-exec-status",
            "arguments": {
                "pid": pid
            }
        });

        let status = Self::send_agent_command(vm_name, &status_cmd.to_string())?;

        let exit_code = status.get("exitcode")
            .and_then(|v| v.as_i64())
            .unwrap_or(-1) as i32;

        // Decode base64 output using simple decoder
        fn decode_base64(input: &str) -> Option<String> {
            let decoded = input.as_bytes().chunks(4).map(|chunk| {
                let mut val = 0u32;
                for (i, &byte) in chunk.iter().enumerate() {
                    let ch = match byte {
                        b'A'..=b'Z' => byte - b'A',
                        b'a'..=b'z' => byte - b'a' + 26,
                        b'0'..=b'9' => byte - b'0' + 52,
                        b'+' => 62,
                        b'/' => 63,
                        b'=' => 0,
                        _ => return None,
                    };
                    val |= (ch as u32) << (18 - 6 * i);
                }
                Some(val)
            }).collect::<Option<Vec<u32>>>()?;

            let mut bytes = Vec::new();
            for val in decoded {
                bytes.push((val >> 16) as u8);
                if bytes.len() < input.trim_end_matches('=').len() * 3 / 4 + 1 {
                    bytes.push((val >> 8) as u8);
                }
                if bytes.len() < input.trim_end_matches('=').len() * 3 / 4 + 1 {
                    bytes.push(val as u8);
                }
            }
            bytes.truncate(input.trim_end_matches('=').len() * 3 / 4);
            String::from_utf8(bytes).ok()
        }

        let stdout = status.get("out-data")
            .and_then(|v| v.as_str())
            .and_then(decode_base64)
            .unwrap_or_default();

        let stderr = status.get("err-data")
            .and_then(|v| v.as_str())
            .and_then(decode_base64)
            .unwrap_or_default();

        Ok(ExecCommandResult {
            exit_code,
            stdout,
            stderr,
            execution_time_ms: 0,
        })
    }

    /// Shutdown the guest gracefully
    pub async fn shutdown(&self, vm_name: &str, _force: bool) -> Result<()> {
        let command = r#"{"execute":"guest-shutdown"}"#;
        Self::send_agent_command(vm_name, command)?;
        Ok(())
    }

    /// Reboot the guest
    pub async fn reboot(&self, vm_name: &str, _force: bool) -> Result<()> {
        let command = r#"{"execute":"guest-shutdown","arguments":{"mode":"reboot"}}"#;
        Self::send_agent_command(vm_name, command)?;
        Ok(())
    }

    /// Get CPU statistics from guest
    pub async fn get_cpu_stats(&self, vm_name: &str) -> Result<GuestCpuStats> {
        let command = r#"{"execute":"guest-get-cpustats"}"#;
        let result = Self::send_agent_command(vm_name, command)?;

        let cpus: Vec<CpuStats> = result.as_array()
            .map(|arr| {
                arr.iter().map(|cpu| {
                    CpuStats {
                        cpu: cpu.get("cpu").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                        user: cpu.get("user").and_then(|v| v.as_u64()).unwrap_or(0),
                        system: cpu.get("system").and_then(|v| v.as_u64()).unwrap_or(0),
                        idle: cpu.get("idle").and_then(|v| v.as_u64()).unwrap_or(0),
                        iowait: cpu.get("iowait").and_then(|v| v.as_u64()).unwrap_or(0),
                        steal: cpu.get("steal").and_then(|v| v.as_u64()).unwrap_or(0),
                        nice: cpu.get("nice").and_then(|v| v.as_u64()).unwrap_or(0),
                        irq: cpu.get("irq").and_then(|v| v.as_u64()).unwrap_or(0),
                        softirq: cpu.get("softirq").and_then(|v| v.as_u64()).unwrap_or(0),
                        guest: cpu.get("guest").and_then(|v| v.as_u64()).unwrap_or(0),
                    }
                }).collect()
            })
            .unwrap_or_default();

        // Calculate totals
        let total_user: u64 = cpus.iter().map(|c| c.user).sum();
        let total_system: u64 = cpus.iter().map(|c| c.system).sum();
        let total_idle: u64 = cpus.iter().map(|c| c.idle).sum();
        let total_all: u64 = cpus.iter().map(|c| c.user + c.system + c.idle + c.iowait + c.nice + c.irq + c.softirq + c.steal).sum();

        let usage_percent = if total_all > 0 {
            ((total_all - total_idle) as f64 / total_all as f64) * 100.0
        } else {
            0.0
        };

        Ok(GuestCpuStats {
            cpus,
            total_user,
            total_system,
            total_idle,
            usage_percent,
        })
    }

    /// Get disk I/O statistics from guest
    pub async fn get_disk_stats(&self, vm_name: &str) -> Result<GuestDiskStats> {
        let command = r#"{"execute":"guest-get-diskstats"}"#;
        let result = Self::send_agent_command(vm_name, command)?;

        let disks: Vec<DiskStats> = result.as_array()
            .map(|arr| {
                arr.iter()
                    .filter(|disk| {
                        // Only include real disks (vda, sda, etc) not loop devices
                        let name = disk.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        !name.starts_with("loop") && !name.starts_with("dm-")
                    })
                    .map(|disk| {
                        let name = disk.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let stats = disk.get("stats");
                        DiskStats {
                            name,
                            read_bytes: stats.and_then(|s| s.get("read-sectors")).and_then(|v| v.as_u64()).unwrap_or(0) * 512,
                            write_bytes: stats.and_then(|s| s.get("write-sectors")).and_then(|v| v.as_u64()).unwrap_or(0) * 512,
                            read_ios: stats.and_then(|s| s.get("read-ios")).and_then(|v| v.as_u64()).unwrap_or(0),
                            write_ios: stats.and_then(|s| s.get("write-ios")).and_then(|v| v.as_u64()).unwrap_or(0),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(GuestDiskStats { disks })
    }

    /// Get logged-in users
    pub async fn get_users(&self, vm_name: &str) -> Result<Vec<GuestUser>> {
        let command = r#"{"execute":"guest-get-users"}"#;
        let result = Self::send_agent_command(vm_name, command)?;

        let users: Vec<GuestUser> = result.as_array()
            .map(|arr| {
                arr.iter().map(|user| {
                    GuestUser {
                        username: user.get("user").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                        login_time: user.get("login-time").and_then(|v| v.as_f64()).unwrap_or(0.0) as u64,
                    }
                }).collect()
            })
            .unwrap_or_default();

        Ok(users)
    }

    /// Get timezone info
    pub async fn get_timezone(&self, vm_name: &str) -> Result<GuestTimezone> {
        let command = r#"{"execute":"guest-get-timezone"}"#;
        let result = Self::send_agent_command(vm_name, command)?;

        Ok(GuestTimezone {
            zone: result.get("zone").and_then(|v| v.as_str()).unwrap_or("UTC").to_string(),
            offset: result.get("offset").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        })
    }

    /// Get full guest information (combines multiple queries)
    pub async fn get_full_info(&self, vm_name: &str) -> Result<GuestFullInfo> {
        // Get agent info
        let agent_info = self.get_agent_info(vm_name).await?;

        // Get system info
        let system_info = self.get_system_info(vm_name).await?;

        // Get timezone
        let timezone = self.get_timezone(vm_name).await.ok();

        // Get users
        let users = self.get_users(vm_name).await.unwrap_or_default();

        // Get disk usage
        let disk_usage = self.get_disk_usage(vm_name).await?;

        // Get network info
        let network_info = self.get_network_info(vm_name).await?;

        Ok(GuestFullInfo {
            agent_version: agent_info.version,
            os_name: system_info.os_name,
            os_version: system_info.os_version,
            kernel_version: system_info.kernel_version,
            hostname: system_info.hostname,
            architecture: system_info.architecture,
            timezone,
            users,
            cpu_count: system_info.cpu_count,
            filesystems: disk_usage.disks,
            network_interfaces: network_info.interfaces,
        })
    }
}

impl Default for GuestAgentService {
    fn default() -> Self {
        Self::new()
    }
}
