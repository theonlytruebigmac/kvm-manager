use kvm_manager_app_lib::models::operation::ConnectionScope;
use kvm_manager_app_lib::services::distribution_profile_service::DistributionProfileService;
use kvm_manager_app_lib::services::readiness_repair_service::ReadinessRepairService;

#[test]
fn verified_distribution_matrix_exposes_only_closed_repair_ids() {
    for os_release in [
        "ID=cachyos\nNAME=CachyOS",
        "ID=ubuntu\nNAME=Ubuntu",
        "ID=fedora\nNAME=Fedora",
        "ID=opensuse-tumbleweed\nNAME=openSUSE",
    ] {
        let profile = DistributionProfileService::classify_os_release(os_release);
        assert!(ReadinessRepairService::is_automated_action(
            "install_firmware",
            &profile
        ));
        assert!(ReadinessRepairService::is_automated_action(
            "install_tpm",
            &profile
        ));
        for rejected in ["", "manual_review", "open_storage", "sudo", "../../bin/sh"] {
            assert!(!ReadinessRepairService::is_automated_action(
                rejected, &profile
            ));
        }
    }
}

#[test]
fn best_effort_profiles_never_expose_automated_repairs() {
    let profile = DistributionProfileService::classify_os_release("ID=other\nNAME=Other");
    assert!(!ReadinessRepairService::is_automated_action(
        "install_firmware",
        &profile
    ));
    assert!(!ReadinessRepairService::is_automated_action(
        "install_tpm",
        &profile
    ));
}

#[test]
fn automated_repairs_are_rejected_for_every_non_local_system_scope() {
    let profile = DistributionProfileService::classify_os_release("ID=arch\nNAME=Arch");
    for scope in [
        ConnectionScope::LocalSession,
        ConnectionScope::Remote,
        ConnectionScope::Test,
    ] {
        assert!(!ReadinessRepairService::is_allowed(
            &scope,
            "install_tpm",
            &profile
        ));
    }
    assert!(ReadinessRepairService::is_allowed(
        &ConnectionScope::LocalSystem,
        "install_tpm",
        &profile,
    ));
}
