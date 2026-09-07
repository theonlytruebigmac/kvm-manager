use crate::models::host::{DistributionFamily, DistributionProfile};
use std::collections::BTreeMap;
use std::fs;

/// Maps OS metadata to the explicitly supported Linux profile matrix.
pub struct DistributionProfileService;

impl DistributionProfileService {
    pub fn scoped_unknown(label: &str) -> DistributionProfile {
        let mut profile = Self::best_effort(label);
        profile.service = "Inspect this connection with its administrator; local setup commands are not applicable.".to_string();
        profile.permission_guidance =
            "No local permission change is recommended for this connection scope.".to_string();
        profile.firmware_guidance =
            "Firmware support is reported by the selected libvirt connection.".to_string();
        profile.limitations = vec![
            "The remote, session, or test guest host distribution is not inferred from the desktop host.".to_string(),
        ];
        profile
    }
    pub fn detect_local() -> DistributionProfile {
        fs::read_to_string("/etc/os-release")
            .map(|contents| Self::classify_os_release(&contents))
            .unwrap_or_else(|_| Self::best_effort("Unknown Linux"))
    }

    pub fn parse_os_release(contents: &str) -> BTreeMap<String, String> {
        contents
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    return None;
                }
                let (key, value) = line.split_once('=')?;
                let value = value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                Some((key.trim().to_ascii_uppercase(), value))
            })
            .collect()
    }

    pub fn classify_os_release(contents: &str) -> DistributionProfile {
        let metadata = Self::parse_os_release(contents);
        let id = metadata
            .get("ID")
            .map(String::as_str)
            .unwrap_or_default()
            .to_ascii_lowercase();
        let id_like = metadata
            .get("ID_LIKE")
            .map(String::as_str)
            .unwrap_or_default()
            .to_ascii_lowercase();
        let display_name = metadata
            .get("PRETTY_NAME")
            .or_else(|| metadata.get("NAME"))
            .cloned()
            .unwrap_or_else(|| "Unknown Linux".to_string());

        if matches!(id.as_str(), "arch" | "cachyos" | "endeavouros") || id_like.contains("arch") {
            return Self::arch(display_name);
        }
        if matches!(id.as_str(), "debian" | "ubuntu" | "linuxmint" | "pop")
            || id_like.contains("debian")
        {
            return Self::debian(display_name);
        }
        if matches!(
            id.as_str(),
            "fedora" | "rhel" | "centos" | "rocky" | "almalinux"
        ) || id_like.contains("fedora")
            || id_like.contains("rhel")
        {
            return Self::fedora(display_name);
        }
        if id.starts_with("opensuse") || id == "sles" || id_like.contains("suse") {
            return Self::opensuse(display_name);
        }

        Self::best_effort(&display_name)
    }

    fn arch(display_name: String) -> DistributionProfile {
        Self::supported(
            DistributionFamily::ArchCachyos,
            display_name,
            "pacman",
            &["libvirt", "qemu-full", "edk2-ovmf", "swtpm"],
            "Enable libvirtd.socket.",
            "Start a new login session after adding the user to the libvirt group.",
        )
    }

    fn debian(display_name: String) -> DistributionProfile {
        Self::supported(
            DistributionFamily::DebianUbuntu,
            display_name,
            "apt",
            &[
                "libvirt-daemon-system",
                "qemu-system-x86",
                "ovmf",
                "swtpm-tools",
            ],
            "Enable and start libvirtd or the distribution's libvirt socket.",
            "Start a new login session after adding the user to the libvirt group.",
        )
    }

    fn fedora(display_name: String) -> DistributionProfile {
        Self::supported(
            DistributionFamily::FedoraRhel,
            display_name,
            "dnf",
            &["libvirt-daemon-kvm", "qemu-kvm", "edk2-ovmf", "swtpm"],
            "Enable and start libvirtd or the distribution's libvirt socket.",
            "Use the distribution's libvirt access policy; do not run the desktop application as root.",
        )
    }

    fn opensuse(display_name: String) -> DistributionProfile {
        Self::supported(
            DistributionFamily::Opensuse,
            display_name,
            "zypper",
            &["libvirt", "qemu-kvm", "qemu-ovmf-x86_64", "swtpm"],
            "Enable and start libvirtd or the distribution's libvirt socket.",
            "Use the distribution's libvirt access policy; do not run the desktop application as root.",
        )
    }

    fn supported(
        family: DistributionFamily,
        display_name: String,
        package_manager: &str,
        packages: &[&str],
        service: &str,
        permission_guidance: &str,
    ) -> DistributionProfile {
        DistributionProfile {
            family,
            display_name,
            package_manager: package_manager.to_string(),
            supported: true,
            packages: packages
                .iter()
                .map(|package| (*package).to_string())
                .collect(),
            service: service.to_string(),
            permission_guidance: permission_guidance.to_string(),
            firmware_guidance:
                "Firmware is discovered from libvirt capabilities and verified existing files."
                    .to_string(),
            limitations: Vec::new(),
        }
    }

    fn best_effort(display_name: &str) -> DistributionProfile {
        DistributionProfile {
            family: DistributionFamily::BestEffort,
            display_name: display_name.to_string(),
            package_manager: "unknown".to_string(),
            supported: false,
            packages: Vec::new(),
            service: "Check your distribution's libvirt documentation.".to_string(),
            permission_guidance: "Do not run the desktop application as root.".to_string(),
            firmware_guidance: "Use libvirt capability detection; no package or path is assumed."
                .to_string(),
            limitations: vec![
                "This distribution is not in the verified support matrix.".to_string()
            ],
        }
    }
}
