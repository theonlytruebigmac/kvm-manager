//! System information handlers

use kvmmanager_agent_common::*;
use serde_json::json;
use std::fs;
use std::process::Command;
use sysinfo::{System, Disks, Networks};
use tracing::debug;

pub async fn handle_get_system_info(id: RequestId) -> JsonRpcResponse {
    debug!("Handling get_system_info");

    let mut sys = System::new_all();
    sys.refresh_all();

    // Get OS information from /etc/os-release
    let (os_name, os_version) = parse_os_release();

    // Get hostname
    let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());

    // Get kernel version
    let kernel_version = System::kernel_version().unwrap_or_else(|| "unknown".to_string());

    // Get architecture
    let architecture = std::env::consts::ARCH.to_string();

    let info = SystemInfo {
        os_type: "linux".to_string(),
        os_name,
        os_version,
        kernel_version,
        hostname,
        architecture,
        cpu_count: sys.cpus().len(),
        total_memory_kb: sys.total_memory(),
        uptime_seconds: System::uptime(),
    };

    JsonRpcResponse::success(json!(info), id)
}

pub async fn handle_get_network_info(id: RequestId) -> JsonRpcResponse {
    debug!("Handling get_network_info");

    let networks = Networks::new_with_refreshed_list();
    let mut interfaces = Vec::new();

    for (interface_name, data) in &networks {
        // Get IP addresses using `ip addr show` command
        let (ipv4_addrs, ipv6_addrs) = get_interface_ips(interface_name);

        let iface = NetworkInterface {
            name: interface_name.clone(),
            mac_address: format_mac_address(data.mac_address()),
            ipv4_addresses: ipv4_addrs,
            ipv6_addresses: ipv6_addrs,
            state: "up".to_string(), // sysinfo doesn't provide this easily
            mtu: 1500, // Default MTU, sysinfo 0.30 doesn't have mtu() method
        };

        interfaces.push(iface);
    }

    let info = NetworkInfo { interfaces };

    JsonRpcResponse::success(json!(info), id)
}

pub async fn handle_get_disk_usage(id: RequestId) -> JsonRpcResponse {
    debug!("Handling get_disk_usage");

    let disks = Disks::new_with_refreshed_list();
    let mut filesystems = Vec::new();

    for disk in &disks {
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total - available;
        let used_percent = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let fs_info = FilesystemInfo {
            mount_point: disk.mount_point().to_string_lossy().to_string(),
            device: disk.name().to_string_lossy().to_string(),
            fs_type: disk.file_system().to_string_lossy().to_string(),
            total_bytes: total,
            used_bytes: used,
            available_bytes: available,
            used_percent,
        };

        filesystems.push(fs_info);
    }

    let info = DiskUsageInfo { filesystems };

    JsonRpcResponse::success(json!(info), id)
}

/// Parse /etc/os-release to get OS name and version
fn parse_os_release() -> (String, String) {
    let content = fs::read_to_string("/etc/os-release")
        .or_else(|_| fs::read_to_string("/usr/lib/os-release"))
        .unwrap_or_default();

    let mut name = String::from("Linux");
    let mut version = String::from("Unknown");

    for line in content.lines() {
        if let Some(val) = line.strip_prefix("NAME=") {
            name = val.trim_matches('"').to_string();
        } else if let Some(val) = line.strip_prefix("VERSION=") {
            version = val.trim_matches('"').to_string();
        } else if let Some(val) = line.strip_prefix("VERSION_ID=") {
            if version == "Unknown" {
                version = val.trim_matches('"').to_string();
            }
        }
    }

    (name, version)
}

/// Format MAC address as colon-separated hex
fn format_mac_address(mac: sysinfo::MacAddr) -> String {
    let bytes = mac.0;
    format!(
        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]
    )
}

/// Get IPv4 and IPv6 addresses for an interface using `ip addr show`
fn get_interface_ips(interface: &str) -> (Vec<String>, Vec<String>) {
    let output = Command::new("ip")
        .args(&["addr", "show", interface])
        .output();

    let mut ipv4_addrs = Vec::new();
    let mut ipv6_addrs = Vec::new();

    if let Ok(output) = output {
        let text = String::from_utf8_lossy(&output.stdout);

        for line in text.lines() {
            let line = line.trim();

            if line.starts_with("inet ") {
                if let Some(addr) = line.split_whitespace().nth(1) {
                    ipv4_addrs.push(addr.to_string());
                }
            } else if line.starts_with("inet6 ") {
                if let Some(addr) = line.split_whitespace().nth(1) {
                    ipv6_addrs.push(addr.to_string());
                }
            }
        }
    }

    (ipv4_addrs, ipv6_addrs)
}
