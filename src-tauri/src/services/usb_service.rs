use std::collections::HashSet;
use std::process::Command;
use std::fs;

use virt::domain::Domain;
use virt::sys;
use crate::models::usb::UsbDevice;
use crate::services::libvirt::LibvirtService;

/// Scan the host system for USB devices
pub fn list_usb_devices(libvirt: &LibvirtService) -> Result<Vec<UsbDevice>, String> {
    // Use lsusb to list USB devices
    let output = Command::new("lsusb")
        .output()
        .map_err(|e| format!("Failed to execute lsusb: {}. Is usbutils installed?", e))?;

    if !output.status.success() {
        return Err(format!(
            "lsusb failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let lsusb_output = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    // Get devices in use by VMs
    let used_devices = get_usb_devices_in_use(libvirt)?;

    // Parse lsusb output
    // Format: Bus 001 Device 002: ID 046d:c52b Logitech, Inc. Unifying Receiver
    for line in lsusb_output.lines() {
        if let Some(device) = parse_lsusb_line(line, &used_devices) {
            // Skip USB hubs
            if device.device_class.as_ref().map_or(false, |c| c.contains("Hub")) {
                continue;
            }
            devices.push(device);
        }
    }

    // Sort by vendor name then product name
    devices.sort_by(|a, b| {
        let a_name = a.display_name();
        let b_name = b.display_name();
        a_name.cmp(&b_name)
    });

    Ok(devices)
}

/// Parse a single line from lsusb output
fn parse_lsusb_line(line: &str, used_devices: &HashSet<String>) -> Option<UsbDevice> {
    // Example: Bus 001 Device 002: ID 046d:c52b Logitech, Inc. Unifying Receiver

    // Find "Bus " prefix
    if !line.starts_with("Bus ") {
        return None;
    }

    // Extract bus number (3 digits after "Bus ")
    let bus = line.get(4..7)?.to_string();

    // Find "Device "
    let device_start = line.find("Device ")?;
    let device = line.get(device_start + 7..device_start + 10)?.trim_end_matches(':').to_string();

    // Find "ID " and extract vendor:product
    let id_start = line.find("ID ")?;
    let id_part = line.get(id_start + 3..)?;

    // Parse vendor:product (8 chars: xxxx:yyyy)
    if id_part.len() < 9 {
        return None;
    }
    let vendor_id = id_part.get(0..4)?.to_string();
    let product_id = id_part.get(5..9)?.to_string();

    // Validate hex IDs
    if !vendor_id.chars().all(|c| c.is_ascii_hexdigit())
        || !product_id.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }

    // Get description (everything after vendor:product)
    let description = id_part.get(10..).unwrap_or("").trim().to_string();

    // Try to get more detailed info from sysfs
    let sysfs_info = get_sysfs_device_info(&bus, &device);

    // Parse vendor and product from description
    let (vendor_name, product_name) = parse_description(&description);

    let device_key = format!("{}:{}", vendor_id, product_id);
    let (in_use, used_by_vm) = if let Some(vm_name) = used_devices.iter().find(|s| s.starts_with(&device_key)) {
        let parts: Vec<&str> = vm_name.split('=').collect();
        if parts.len() > 1 {
            (true, Some(parts[1].to_string()))
        } else {
            (true, None)
        }
    } else {
        (false, None)
    };

    Some(UsbDevice {
        bus,
        device,
        vendor_id,
        product_id,
        vendor_name,
        product_name,
        description,
        speed: sysfs_info.0,
        device_class: sysfs_info.1,
        in_use,
        used_by_vm,
    })
}

/// Get device info from sysfs
fn get_sysfs_device_info(bus: &str, device: &str) -> (Option<String>, Option<String>) {
    let sysfs_base = format!("/sys/bus/usb/devices/{}-{}", bus.trim_start_matches('0'), device);

    // Try to read speed
    let speed = fs::read_to_string(format!("{}/speed", sysfs_base))
        .ok()
        .map(|s| {
            let speed_mbps = s.trim();
            match speed_mbps {
                "1.5" => "1.5 Mbps (USB 1.0 Low Speed)".to_string(),
                "12" => "12 Mbps (USB 1.1 Full Speed)".to_string(),
                "480" => "480 Mbps (USB 2.0 High Speed)".to_string(),
                "5000" => "5 Gbps (USB 3.0 SuperSpeed)".to_string(),
                "10000" => "10 Gbps (USB 3.1 SuperSpeed+)".to_string(),
                "20000" => "20 Gbps (USB 3.2 SuperSpeed++)".to_string(),
                _ => format!("{} Mbps", speed_mbps),
            }
        });

    // Try to read device class
    let device_class = fs::read_to_string(format!("{}/bDeviceClass", sysfs_base))
        .ok()
        .and_then(|class_code| {
            let code = class_code.trim();
            Some(match code {
                "00" => "Defined by Interface".to_string(),
                "01" => "Audio".to_string(),
                "02" => "Communications".to_string(),
                "03" => "Human Interface Device".to_string(),
                "05" => "Physical".to_string(),
                "06" => "Image".to_string(),
                "07" => "Printer".to_string(),
                "08" => "Mass Storage".to_string(),
                "09" => "Hub".to_string(),
                "0a" => "CDC-Data".to_string(),
                "0b" => "Smart Card".to_string(),
                "0d" => "Content Security".to_string(),
                "0e" => "Video".to_string(),
                "0f" => "Personal Healthcare".to_string(),
                "10" => "Audio/Video".to_string(),
                "11" => "Billboard".to_string(),
                "dc" => "Diagnostic".to_string(),
                "e0" => "Wireless Controller".to_string(),
                "ef" => "Miscellaneous".to_string(),
                "fe" => "Application Specific".to_string(),
                "ff" => "Vendor Specific".to_string(),
                _ => format!("Class {}", code),
            })
        });

    (speed, device_class)
}

/// Parse vendor and product names from lsusb description
fn parse_description(description: &str) -> (String, String) {
    // Try to split on common patterns
    // Example: "Logitech, Inc. Unifying Receiver"
    // Example: "Apple, Inc. FaceTime HD Camera"
    // Example: "Generic USB Hub"

    if description.is_empty() {
        return ("Unknown".to_string(), "Unknown".to_string());
    }

    // Look for patterns like "Vendor, Inc. Product" or "Vendor Product"
    if let Some(pos) = description.find(", Inc.") {
        let vendor = description[..pos].to_string();
        let product = description[pos + 6..].trim().to_string();
        return (vendor, product);
    }

    if let Some(pos) = description.find(" Inc.") {
        let vendor = description[..pos].to_string();
        let product = description[pos + 5..].trim().to_string();
        return (vendor, product);
    }

    if let Some(pos) = description.find(" Corp.") {
        let vendor = description[..pos].to_string();
        let product = description[pos + 6..].trim().to_string();
        return (vendor, product);
    }

    if let Some(pos) = description.find(" Technology") {
        let vendor = description[..pos + 11].to_string();
        let product = description[pos + 11..].trim().to_string();
        return (vendor, if product.is_empty() { "USB Device".to_string() } else { product });
    }

    // Default: first word is vendor, rest is product
    let parts: Vec<&str> = description.splitn(2, ' ').collect();
    if parts.len() == 2 {
        (parts[0].to_string(), parts[1].to_string())
    } else {
        (description.to_string(), String::new())
    }
}

/// Get list of USB devices currently attached to running VMs
fn get_usb_devices_in_use(libvirt: &LibvirtService) -> Result<HashSet<String>, String> {
    let mut used = HashSet::new();

    let conn = libvirt.get_connection();

    // Get active domains
    let flags = sys::VIR_CONNECT_LIST_DOMAINS_ACTIVE;
    let domains = conn
        .list_all_domains(flags)
        .map_err(|e| format!("Failed to list domains: {}", e))?;

    for domain in domains {
        if let Ok(xml) = domain.get_xml_desc(0) {
            if let Ok(name) = domain.get_name() {
                // Parse XML for USB hostdev entries
                parse_usb_devices_from_xml(&xml, &name, &mut used);
            }
        }
    }

    Ok(used)
}

/// Parse USB devices from domain XML
fn parse_usb_devices_from_xml(xml: &str, vm_name: &str, used: &mut HashSet<String>) {
    // Simple string-based parsing for USB devices
    // Look for patterns like <vendor id='0xXXXX'/> and <product id='0xYYYY'/>

    // Find all hostdev type='usb' blocks
    let mut pos = 0;
    while let Some(start) = xml[pos..].find("<hostdev") {
        let absolute_start = pos + start;

        // Check if this is a USB hostdev
        if let Some(end) = xml[absolute_start..].find("</hostdev>") {
            let block = &xml[absolute_start..absolute_start + end + 10];

            if block.contains("type='usb'") || block.contains("type=\"usb\"") {
                // Extract vendor ID
                let vendor_id = extract_id_from_xml(block, "vendor");
                let product_id = extract_id_from_xml(block, "product");

                if let (Some(vid), Some(pid)) = (vendor_id, product_id) {
                    used.insert(format!("{}:{}={}", vid, pid, vm_name));
                }
            }

            pos = absolute_start + end + 10;
        } else {
            break;
        }
    }
}

/// Extract an ID from XML like <vendor id='0xXXXX'/>
fn extract_id_from_xml(xml: &str, tag: &str) -> Option<String> {
    // Look for <vendor id='0xXXXX'/> or <vendor id="0xXXXX"/>
    let pattern1 = format!("<{} id='0x", tag);
    let pattern2 = format!("<{} id=\"0x", tag);

    let start = xml.find(&pattern1).or_else(|| xml.find(&pattern2))?;
    let id_start = start + pattern1.len();

    // Extract 4 hex characters
    let id = xml.get(id_start..id_start + 4)?;
    if id.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(id.to_lowercase())
    } else {
        None
    }
}

/// Attach a USB device to a running VM
pub fn attach_usb_device(
    libvirt: &LibvirtService,
    vm_id: &str,
    vendor_id: &str,
    product_id: &str,
) -> Result<(), String> {
    let conn = libvirt.get_connection();

    let domain = Domain::lookup_by_uuid_string(conn, vm_id)
        .map_err(|e| format!("VM '{}' not found: {}", vm_id, e))?;

    // Check if the domain is running
    let info = domain
        .get_info()
        .map_err(|e| format!("Failed to get VM info: {}", e))?;

    if info.state != sys::VIR_DOMAIN_RUNNING {
        return Err("VM must be running to attach USB devices".to_string());
    }

    // Validate vendor/product IDs
    if !is_valid_hex_id(vendor_id) || !is_valid_hex_id(product_id) {
        return Err("Invalid vendor or product ID format".to_string());
    }

    // Create USB hostdev XML
    let device_xml = format!(
        r#"<hostdev mode='subsystem' type='usb' managed='yes'>
  <source>
    <vendor id='0x{}'/>
    <product id='0x{}'/>
  </source>
</hostdev>"#,
        vendor_id, product_id
    );

    // Attach the device
    domain
        .attach_device(&device_xml)
        .map_err(|e| format!("Failed to attach USB device: {}", e))?;

    Ok(())
}

/// Detach a USB device from a running VM
pub fn detach_usb_device(
    libvirt: &LibvirtService,
    vm_id: &str,
    vendor_id: &str,
    product_id: &str,
) -> Result<(), String> {
    let conn = libvirt.get_connection();

    let domain = Domain::lookup_by_uuid_string(conn, vm_id)
        .map_err(|e| format!("VM '{}' not found: {}", vm_id, e))?;

    // Validate vendor/product IDs
    if !is_valid_hex_id(vendor_id) || !is_valid_hex_id(product_id) {
        return Err("Invalid vendor or product ID format".to_string());
    }

    // Create USB hostdev XML (must match the attached device)
    let device_xml = format!(
        r#"<hostdev mode='subsystem' type='usb' managed='yes'>
  <source>
    <vendor id='0x{}'/>
    <product id='0x{}'/>
  </source>
</hostdev>"#,
        vendor_id, product_id
    );

    // Detach the device
    domain
        .detach_device(&device_xml)
        .map_err(|e| format!("Failed to detach USB device: {}", e))?;

    Ok(())
}

/// Validate a 4-character hex ID
fn is_valid_hex_id(id: &str) -> bool {
    id.len() == 4 && id.chars().all(|c| c.is_ascii_hexdigit())
}
