use crate::models::host::{
    DistributionFamily, DistributionProfile, ReadinessRepairOutcome, ReadinessRepairResult,
};
use crate::models::operation::ConnectionScope;
use crate::utils::error::AppError;
use std::process::{Command, Stdio};

pub struct ReadinessRepairService;

struct RepairPlan {
    program: &'static str,
    args: Vec<&'static str>,
}

impl ReadinessRepairService {
    pub fn is_known_action(action_id: &str) -> bool {
        matches!(action_id, "install_firmware" | "install_tpm")
    }

    fn plan(action_id: &str, profile: &DistributionProfile) -> Option<RepairPlan> {
        use DistributionFamily::*;
        let plan = match (action_id, profile.family) {
            ("install_firmware", ArchCachyos) => RepairPlan {
                program: "/usr/bin/pacman",
                args: vec!["-S", "--needed", "--noconfirm", "edk2-ovmf"],
            },
            ("install_firmware", DebianUbuntu) => RepairPlan {
                program: "/usr/bin/apt-get",
                args: vec!["install", "-y", "ovmf"],
            },
            ("install_firmware", FedoraRhel) => RepairPlan {
                program: "/usr/bin/dnf",
                args: vec!["install", "-y", "edk2-ovmf"],
            },
            ("install_firmware", Opensuse) => RepairPlan {
                program: "/usr/bin/zypper",
                args: vec!["--non-interactive", "install", "qemu-ovmf-x86_64"],
            },
            ("install_tpm", ArchCachyos) => RepairPlan {
                program: "/usr/bin/pacman",
                args: vec!["-S", "--needed", "--noconfirm", "swtpm"],
            },
            ("install_tpm", DebianUbuntu) => RepairPlan {
                program: "/usr/bin/apt-get",
                args: vec!["install", "-y", "swtpm-tools"],
            },
            ("install_tpm", FedoraRhel) => RepairPlan {
                program: "/usr/bin/dnf",
                args: vec!["install", "-y", "swtpm"],
            },
            ("install_tpm", Opensuse) => RepairPlan {
                program: "/usr/bin/zypper",
                args: vec!["--non-interactive", "install", "swtpm"],
            },
            _ => return None,
        };
        profile.supported.then_some(plan)
    }

    pub fn is_automated_action(action_id: &str, profile: &DistributionProfile) -> bool {
        Self::plan(action_id, profile).is_some()
    }

    pub fn is_allowed(
        scope: &ConnectionScope,
        action_id: &str,
        profile: &DistributionProfile,
    ) -> bool {
        *scope == ConnectionScope::LocalSystem && Self::is_automated_action(action_id, profile)
    }

    pub fn execute(
        action_id: &str,
        connection_id: &str,
        profile: &DistributionProfile,
    ) -> Result<ReadinessRepairResult, AppError> {
        let plan = Self::plan(action_id, profile).ok_or_else(|| {
            AppError::InvalidConfig(
                "This readiness repair is not available for the detected distribution".to_string(),
            )
        })?;
        let status = match Command::new("/usr/bin/pkexec")
            .arg(plan.program)
            .args(plan.args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            Ok(status) => status,
            Err(_) => {
                return Ok(ReadinessRepairResult {
                    action_id: action_id.to_string(),
                    connection_id: connection_id.to_string(),
                    outcome: ReadinessRepairOutcome::InspectionRequired,
                    summary: "Desktop privilege authorization is unavailable. Follow the guided manual steps.".to_string(),
                });
            }
        };
        let (outcome, summary) = if status.success() {
            (
                ReadinessRepairOutcome::Applied,
                "The repair completed. Readiness will now be checked again.",
            )
        } else if status.code() == Some(126) {
            (
                ReadinessRepairOutcome::Cancelled,
                "Authorization was cancelled. No successful repair was recorded.",
            )
        } else if status.code() == Some(127) {
            (
                ReadinessRepairOutcome::Rejected,
                "Authorization was not granted. Use the guided manual steps or retry.",
            )
        } else {
            (
                ReadinessRepairOutcome::Failed,
                "The repair did not complete. Review the guided steps and retry.",
            )
        };
        Ok(ReadinessRepairResult {
            action_id: action_id.to_string(),
            connection_id: connection_id.to_string(),
            outcome,
            summary: summary.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ReadinessRepairService;
    use crate::services::distribution_profile_service::DistributionProfileService;

    #[test]
    fn allowlist_rejects_unknown_and_best_effort_actions() {
        let arch = DistributionProfileService::classify_os_release("ID=arch\nNAME=Arch");
        let unknown = DistributionProfileService::classify_os_release("ID=unknown\nNAME=Other");
        assert!(ReadinessRepairService::is_automated_action(
            "install_firmware",
            &arch
        ));
        assert!(!ReadinessRepairService::is_automated_action(
            "run_anything",
            &arch
        ));
        assert!(!ReadinessRepairService::is_known_action("../../bin/sh"));
        assert!(!ReadinessRepairService::is_automated_action(
            "install_firmware",
            &unknown
        ));
    }

    #[test]
    fn every_executable_and_argument_is_backend_owned() {
        for release in [
            "ID=arch",
            "ID=ubuntu",
            "ID=fedora",
            "ID=opensuse-tumbleweed",
        ] {
            let profile = DistributionProfileService::classify_os_release(release);
            for action in ["install_firmware", "install_tpm"] {
                let plan = ReadinessRepairService::plan(action, &profile).unwrap();
                assert!(plan.program.starts_with("/usr/bin/"));
                assert!(plan
                    .args
                    .iter()
                    .all(|arg| !arg.contains(';') && !arg.contains("$(")));
            }
        }
    }
}
