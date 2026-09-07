use crate::models::host::{
    CapabilityResult, CapabilityState, DistributionProfile, GuestCapabilityReview,
    GuestRequirements, HostProbe, HostReadinessReport, ReadinessRepairAction, ReadinessRepairMode,
    StorageReadinessState, VmCreationReadiness,
};
use crate::models::operation::{ConnectionScope, OperationContext};
use crate::models::vm::VmConfig;
use crate::services::distribution_profile_service::DistributionProfileService;
use crate::services::libvirt::ConnectionProvider;
use crate::services::storage_service::StorageService;
use crate::utils::error::AppError;
use chrono::Utc;
use std::path::Path;
use virt::network::Network;

/// Performs safe host capability probes and maps each independently to actionable diagnostics.
pub struct HostReadinessService;

impl HostReadinessService {
    pub fn connection_report(
        libvirt: &impl ConnectionProvider,
        context: &OperationContext,
        required_disk_bytes: Option<u64>,
        selected_pool_id: Option<&str>,
    ) -> Result<VmCreationReadiness, AppError> {
        let profile = if context.connection_scope == ConnectionScope::LocalSystem {
            DistributionProfileService::detect_local()
        } else {
            DistributionProfileService::scoped_unknown(&format!(
                "{} connection",
                match context.connection_scope {
                    ConnectionScope::LocalSession => "Local session",
                    ConnectionScope::Remote => "Remote",
                    ConnectionScope::Test => "Test",
                    ConnectionScope::LocalSystem => "Local system",
                }
            ))
        };
        let domain_caps = libvirt
            .get_connection()
            .get_domain_capabilities(None, None, None, None, 0)
            .unwrap_or_default();
        let mut capabilities = vec![CapabilityResult {
            kind: "libvirt_access".to_string(),
            state: CapabilityState::Available,
            summary: "The selected libvirt connection is available.".to_string(),
            remediation: None,
            details: Some("Capability checks are owned by the selected connection.".to_string()),
            repair_action: None,
        }];
        let accelerated = context.connection_scope == ConnectionScope::Test
            || domain_caps
                .to_ascii_lowercase()
                .contains("<domain>kvm</domain>");
        capabilities.push(Self::capability(
            "virtualization",
            accelerated,
            "Hardware-accelerated virtualization is available on this connection.",
            "Enable virtualization for the selected guest host and verify libvirt KVM access.",
        ));
        capabilities.extend(Self::domain_capabilities(&domain_caps));
        let storage = StorageService::storage_readiness(
            libvirt,
            &context.connection_id,
            required_disk_bytes,
            selected_pool_id,
        )?;
        capabilities.push(CapabilityResult {
            kind: "storage".to_string(),
            state: if storage.pools.iter().any(|pool| pool.eligible) {
                CapabilityState::Available
            } else {
                CapabilityState::Unavailable
            },
            summary: if storage.pools.iter().any(|pool| pool.eligible) {
                "Usable storage is available on this connection.".to_string()
            } else {
                "No active storage pool can satisfy the requested disk.".to_string()
            },
            remediation: (!storage.pools.iter().any(|pool| pool.eligible)).then(|| {
                "Inspect, create, or activate a storage pool for the selected connection."
                    .to_string()
            }),
            details: None,
            repair_action: None,
        });
        Self::attach_repair_actions(&mut capabilities, context, &profile);
        let overall_state = if capabilities
            .iter()
            .any(|capability| capability.state == CapabilityState::Unavailable)
        {
            "degraded"
        } else {
            "ready"
        };
        Ok(VmCreationReadiness {
            checked_at: Utc::now().to_rfc3339(),
            connection_id: context.connection_id.clone(),
            connection_label: context.connection_label.clone(),
            connection_scope: context.connection_scope.clone(),
            distribution: profile,
            overall_state: overall_state.to_string(),
            capabilities,
            storage,
        })
    }

    pub fn preflight(
        libvirt: &impl ConnectionProvider,
        context: &OperationContext,
        config: &VmConfig,
    ) -> Result<GuestCapabilityReview, AppError> {
        let needs_disk = config.installation_type != "import";
        let required_bytes = needs_disk
            .then(|| StorageService::disk_bytes(config.disk_size_gb))
            .transpose()?;
        let readiness = Self::connection_report(
            libvirt,
            context,
            required_bytes,
            config.storage_pool_id.as_deref(),
        )?;
        let requirements = GuestRequirements {
            firmware: config.firmware.clone(),
            tpm_enabled: config.tpm_enabled,
            network: (!config.network.trim().is_empty()).then(|| config.network.clone()),
        };
        let machine = (config.chipset == "q35").then_some("q35");
        let profile_caps = libvirt
            .get_connection()
            .get_domain_capabilities(None, None, machine, None, 0)
            .unwrap_or_default();
        let mut capabilities: Vec<_> = readiness
            .capabilities
            .into_iter()
            .filter(|item| !matches!(item.kind.as_str(), "uefi" | "secure_boot" | "tpm"))
            .collect();
        capabilities.extend(Self::domain_capabilities(&profile_caps));
        let firmware_kind = match config.firmware.as_str() {
            "uefi-secure" => Some("secure_boot"),
            "uefi" => Some("uefi"),
            _ => None,
        };
        if let Some(kind) = firmware_kind {
            Self::require_capability(&mut capabilities, kind);
        }
        if config.tpm_enabled {
            Self::require_capability(&mut capabilities, "tpm");
        }
        if let Some(network_name) = requirements.network.as_deref() {
            let network_available = Network::lookup_by_name(libvirt.get_connection(), network_name)
                .and_then(|network| network.is_active())
                .unwrap_or(false);
            capabilities.push(Self::capability(
                "network",
                network_available,
                "The selected virtual network is active.",
                "Select or activate a virtual network on this connection.",
            ));
        }
        let storage_ok = !needs_disk || readiness.storage.state == StorageReadinessState::Ready;
        let can_create = storage_ok
            && capabilities
                .iter()
                .filter(|capability| {
                    capability.kind == "network"
                        || capability.kind == "libvirt_access"
                        || firmware_kind == Some(capability.kind.as_str())
                        || (config.tpm_enabled && capability.kind == "tpm")
                })
                .all(|capability| capability.state == CapabilityState::Available);
        Ok(GuestCapabilityReview {
            checked_at: Utc::now().to_rfc3339(),
            connection_id: context.connection_id.clone(),
            requirements,
            capabilities,
            storage: readiness.storage,
            can_create,
        })
    }

    pub fn domain_capabilities(xml: &str) -> Vec<CapabilityResult> {
        let lower = xml.to_ascii_lowercase();
        let uefi = lower.contains("<value>efi</value>")
            || lower.contains("firmware='efi'")
            || lower.contains("firmware=\"efi\"");
        let secure_boot =
            uefi && (Self::enum_has_yes(&lower, "secureboot") || lower.contains("secure-boot"));
        let enrolled_keys =
            Self::enum_has_yes(&lower, "enrolledkeys") || lower.contains("enrolled-keys");
        let secure = secure_boot && enrolled_keys;
        let tpm = (lower.contains("<tpm") && !lower.contains("<tpm supported='no'"))
            || lower.contains("<value>tpm-tis</value>");
        vec![
            Self::capability(
                "uefi",
                uefi,
                "UEFI firmware is available through libvirt.",
                "Install firmware supported by this connection and refresh capabilities.",
            ),
            Self::capability(
                "secure_boot",
                secure,
                "Secure Boot firmware with enrolled keys is available through libvirt.",
                if secure_boot && !enrolled_keys {
                    "Secure Boot firmware is installed, but its variable-store template does not contain enrolled signing keys."
                } else {
                    "Install Secure Boot firmware with an enrolled-key template supported by this connection."
                },
            ),
            Self::capability(
                "tpm",
                tpm,
                "A TPM emulator is available through libvirt.",
                "Install and configure a libvirt-supported TPM emulator on this connection.",
            ),
        ]
    }

    fn enum_has_yes(xml: &str, enum_name: &str) -> bool {
        let single = format!("name='{enum_name}'");
        let double = format!("name=\"{enum_name}\"");
        xml.find(&single)
            .or_else(|| xml.find(&double))
            .and_then(|start| {
                xml[start..]
                    .find("</enum>")
                    .map(|end| &xml[start..start + end])
            })
            .map(|section| section.contains("<value>yes</value>"))
            .unwrap_or(false)
    }

    fn require_capability(capabilities: &mut [CapabilityResult], kind: &str) {
        if let Some(capability) = capabilities.iter_mut().find(|item| item.kind == kind) {
            if capability.state != CapabilityState::Available {
                capability.details =
                    Some("Required by the selected guest configuration.".to_string());
            }
        }
    }
    pub fn probe_local() -> HostProbe {
        HostProbe {
            libvirt_access: Self::command_available("virsh"),
            qemu_emulator: Self::command_available("qemu-system-x86_64"),
            kvm: Path::new("/dev/kvm").exists(),
            uefi: Self::firmware_exists(),
            secure_boot: Self::firmware_exists(),
            forwarding_privilege: Self::native_forwarding_helper_exists(),
        }
    }

    pub fn report(profile: DistributionProfile, probe: HostProbe) -> HostReadinessReport {
        let capabilities = vec![
            Self::capability(
                "libvirt_access",
                probe.libvirt_access,
                "Libvirt connection tools are available.",
                "Install and enable libvirt, then verify access to qemu:///system.",
            ),
            Self::capability(
                "qemu_emulator",
                probe.qemu_emulator,
                "A QEMU system emulator is available.",
                "Install the QEMU system emulator package for this distribution.",
            ),
            Self::capability(
                "kvm",
                probe.kvm,
                "KVM device access is available.",
                "Enable virtualization in firmware and ensure /dev/kvm is accessible.",
            ),
            Self::capability(
                "uefi",
                probe.uefi,
                "UEFI firmware was found.",
                "Install the distribution's UEFI firmware package.",
            ),
            Self::capability(
                "secure_boot",
                probe.secure_boot,
                "Secure Boot capable firmware was found.",
                "Install Secure Boot capable UEFI firmware for this distribution.",
            ),
            Self::capability(
                "forwarding_privilege",
                probe.forwarding_privilege,
                "The native forwarding helper is installed.",
                "Install a supported native KVM Manager package to use port forwarding.",
            ),
        ];
        let required_ready = capabilities
            .iter()
            .take(4)
            .all(|capability| capability.state == CapabilityState::Available);

        HostReadinessReport {
            checked_at: Utc::now().to_rfc3339(),
            connection_uri: "qemu:///system".to_string(),
            distribution: profile,
            overall_state: if required_ready {
                "ready".to_string()
            } else {
                "degraded".to_string()
            },
            capabilities,
        }
    }

    pub fn local_report() -> HostReadinessReport {
        Self::report(
            DistributionProfileService::detect_local(),
            Self::probe_local(),
        )
    }

    fn capability(
        kind: &str,
        available: bool,
        available_summary: &str,
        remediation: &str,
    ) -> CapabilityResult {
        CapabilityResult {
            kind: kind.to_string(),
            state: if available {
                CapabilityState::Available
            } else {
                CapabilityState::Unavailable
            },
            summary: if available {
                available_summary.to_string()
            } else {
                format!("{} is unavailable.", kind.replace('_', " "))
            },
            remediation: (!available).then(|| remediation.to_string()),
            details: None,
            repair_action: None,
        }
    }

    fn attach_repair_actions(
        capabilities: &mut [CapabilityResult],
        context: &OperationContext,
        profile: &DistributionProfile,
    ) {
        for capability in capabilities
            .iter_mut()
            .filter(|item| item.state != CapabilityState::Available)
        {
            let (id, mode, title, effect, privileged, steps) = match capability.kind.as_str() {
                "uefi"
                    if context.connection_scope == ConnectionScope::LocalSystem
                        && profile.supported =>
                {
                    let package = profile
                        .packages
                        .iter()
                        .find(|package| package.contains("ovmf"))
                        .cloned()
                        .unwrap_or_else(|| "the verified UEFI firmware package".to_string());
                    (
                        "install_firmware",
                        ReadinessRepairMode::Automated,
                        "Install virtualization firmware",
                        format!("Install package {} for {}.", package, profile.display_name),
                        true,
                        vec![
                            "Authorize the package installation when prompted.".to_string(),
                            "Refresh libvirt capabilities after installation.".to_string(),
                        ],
                    )
                }
                "tpm"
                    if context.connection_scope == ConnectionScope::LocalSystem
                        && profile.supported =>
                {
                    let package = profile
                        .packages
                        .iter()
                        .find(|package| package.starts_with("swtpm"))
                        .cloned()
                        .unwrap_or_else(|| "the verified TPM emulator package".to_string());
                    (
                        "install_tpm",
                        ReadinessRepairMode::Automated,
                        "Install TPM emulation support",
                        format!("Install package {} for {}.", package, profile.display_name),
                        true,
                        vec![
                            "Authorize the package installation when prompted.".to_string(),
                            "Refresh readiness after installation.".to_string(),
                        ],
                    )
                }
                "storage" => (
                    "open_storage",
                    ReadinessRepairMode::Navigate,
                    "Configure storage",
                    "Open storage management for the selected connection.".to_string(),
                    false,
                    vec![
                        "Create or activate a storage pool, then return and refresh readiness."
                            .to_string(),
                    ],
                ),
                "virtualization" => (
                    "firmware_virtualization",
                    ReadinessRepairMode::Manual,
                    "Enable hardware virtualization",
                    "Review firmware virtualization and KVM access for this host.".to_string(),
                    false,
                    vec![
                        "Enable Intel VT-x or AMD-V in system firmware.".to_string(),
                        "Restart the host and verify KVM access.".to_string(),
                    ],
                ),
                "secure_boot" => (
                    "secure_boot_guidance",
                    ReadinessRepairMode::Manual,
                    "Configure Secure Boot firmware",
                    if profile.family == crate::models::host::DistributionFamily::ArchCachyos {
                        "CachyOS has Secure Boot-capable OVMF installed, but Arch-family edk2-ovmf does not provide a variable-store template with Microsoft keys already enrolled.".to_string()
                    } else {
                        "Secure Boot firmware and an enrolled-key variable-store template must both be advertised by the selected libvirt connection.".to_string()
                    },
                    false,
                    if profile.family == crate::models::host::DistributionFamily::ArchCachyos {
                        let mut steps = vec![
                            "The edk2-ovmf package is already detected; reinstalling it will not add enrolled keys.".to_string(),
                            "Option 1 — continue without Secure Boot enforcement: Return to the VM wizard and select UEFI. TPM 2.0 can remain enabled. This does not satisfy the Windows 11 Secure Boot requirement.".to_string(),
                            "Option 2 — prepare for Windows 11 Secure Boot: An administrator must provide an enrolled-key firmware template to libvirt, or create and import the VM with its own enrolled NVRAM.".to_string(),
                        ];
                        if Path::new("/usr/bin/virt-fw-vars").is_file() {
                            steps.push("The virt-firmware key-enrollment utility is already installed on this host.".to_string());
                        } else {
                            steps.push("Run: sudo pacman -S --needed virt-firmware".to_string());
                        }
                        steps.extend([
                            "Do not overwrite the system OVMF_VARS template. KVM Manager does not currently automate the remaining Arch/CachyOS key-template setup because changing a shared template could affect other VMs.".to_string(),
                            "Reference: https://wiki.archlinux.org/title/KVM#Enabling_Secure_Boot".to_string(),
                            "Use Recheck readiness only after libvirt advertises an enrolled-key firmware template. A VM-specific NVRAM change will not change this host-wide check.".to_string(),
                        ]);
                        steps
                    } else {
                        vec![
                            format!("Verify the Secure Boot/enrolled-key firmware package for {} is installed.", profile.display_name),
                            "Restart or refresh the libvirt service after changing firmware packages.".to_string(),
                            "Select Recheck readiness and confirm that both Secure Boot and enrolled keys are available.".to_string(),
                        ]
                    },
                ),
                _ => (
                    "manual_review",
                    ReadinessRepairMode::Manual,
                    "Review setup guidance",
                    capability.remediation.clone().unwrap_or_else(|| {
                        "Review this requirement with the host administrator.".to_string()
                    }),
                    false,
                    vec![capability
                        .remediation
                        .clone()
                        .unwrap_or_else(|| "Contact the selected host administrator.".to_string())],
                ),
            };
            capability.repair_action = Some(ReadinessRepairAction {
                id: id.to_string(),
                mode,
                title: title.to_string(),
                effect,
                requires_privilege: privileged,
                requires_confirmation: mode == ReadinessRepairMode::Automated,
                expected_connection_id: context.connection_id.clone(),
                steps,
            });
        }
    }

    fn command_available(command: &str) -> bool {
        std::process::Command::new(command)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn firmware_exists() -> bool {
        [
            "/usr/share/edk2/x64/OVMF_CODE.4m.fd",
            "/usr/share/OVMF/OVMF_CODE.fd",
            "/usr/share/edk2-ovmf/x64/OVMF_CODE.fd",
        ]
        .iter()
        .any(|path| Path::new(path).is_file())
    }

    fn native_forwarding_helper_exists() -> bool {
        Path::new("/usr/libexec/kvm-manager-network-helper").is_file()
    }
}
