use kvm_manager_app_lib::models::host::{CapabilityState, HostProbe, StorageReadinessState};
use kvm_manager_app_lib::models::storage::StoragePool;
use kvm_manager_app_lib::services::distribution_profile_service::DistributionProfileService;
use kvm_manager_app_lib::services::host_readiness_service::HostReadinessService;
use kvm_manager_app_lib::services::storage_service::StorageService;

fn cachyos_profile() -> kvm_manager_app_lib::models::host::DistributionProfile {
    DistributionProfileService::classify_os_release(include_str!("fixtures/os-release/cachyos"))
}

#[test]
fn pool_readiness_requires_an_explicit_eligible_uuid() {
    let pools: Vec<StoragePool> = serde_json::from_str(include_str!(
        "fixtures/first-run-onboarding/pools-multiple.json"
    ))
    .unwrap();
    let unselected =
        StorageService::assess_pools("fixture-a", &pools, Some(20 * 1024_u64.pow(3)), None);
    assert_eq!(unselected.state, StorageReadinessState::SelectionRequired);
    assert!(!serde_json::to_string(&unselected)
        .unwrap()
        .contains("/fixture/"));

    let selected = StorageService::assess_pools(
        "fixture-a",
        &pools,
        Some(20 * 1024_u64.pow(3)),
        Some("22222222-2222-2222-2222-222222222222"),
    );
    assert_eq!(selected.state, StorageReadinessState::Ready);
}

#[test]
fn pool_readiness_explains_zero_inactive_and_insufficient_capacity() {
    let zero: Vec<StoragePool> = serde_json::from_str(include_str!(
        "fixtures/first-run-onboarding/pools-zero.json"
    ))
    .unwrap();
    assert_eq!(
        StorageService::assess_pools("fixture", &zero, Some(1), None).state,
        StorageReadinessState::Unavailable
    );
    for fixture in ["pools-inactive.json", "pools-insufficient.json"] {
        let pools: Vec<StoragePool> = serde_json::from_str(match fixture {
            "pools-inactive.json" => {
                include_str!("fixtures/first-run-onboarding/pools-inactive.json")
            }
            _ => include_str!("fixtures/first-run-onboarding/pools-insufficient.json"),
        })
        .unwrap();
        let result =
            StorageService::assess_pools("fixture", &pools, Some(2 * 1024_u64.pow(3)), None);
        assert!(matches!(
            result.state,
            StorageReadinessState::Unavailable | StorageReadinessState::InsufficientCapacity
        ));
        assert!(!result.pools[0].eligible);
    }
}

#[test]
fn domain_capability_fixtures_distinguish_windows_requirements() {
    let full = HostReadinessService::domain_capabilities(include_str!(
        "fixtures/first-run-onboarding/domain-capabilities-full.xml"
    ));
    assert!(full
        .iter()
        .all(|item| item.state == CapabilityState::Available));

    let limited = HostReadinessService::domain_capabilities(include_str!(
        "fixtures/first-run-onboarding/domain-capabilities-bios-only.xml"
    ));
    assert!(limited
        .iter()
        .all(|item| item.state == CapabilityState::Unavailable));
}

#[test]
fn secure_boot_firmware_without_enrolled_keys_reports_the_actual_blocker() {
    let capabilities = HostReadinessService::domain_capabilities(
        "<domainCapabilities><os supported='yes'><enum name='firmware'><value>efi</value></enum><firmwareFeatures supported='yes'><enum name='secureBoot'><value>yes</value></enum><enum name='enrolledKeys'><value>no</value></enum></firmwareFeatures></os></domainCapabilities>",
    );
    let secure_boot = capabilities
        .iter()
        .find(|capability| capability.kind == "secure_boot")
        .unwrap();

    assert_eq!(secure_boot.state, CapabilityState::Unavailable);
    assert!(secure_boot
        .remediation
        .as_deref()
        .unwrap()
        .contains("variable-store template"));
    assert!(!secure_boot
        .remediation
        .as_deref()
        .unwrap()
        .contains("Install Secure Boot firmware"));
}

#[test]
fn non_local_distribution_guidance_does_not_claim_desktop_setup() {
    let profile = DistributionProfileService::scoped_unknown("Remote connection");
    assert!(!profile.supported);
    assert!(profile.packages.is_empty());
    assert!(profile
        .service
        .contains("local setup commands are not applicable"));
}

#[test]
fn every_non_local_scope_fixture_forbids_local_guidance() {
    let fixtures: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/first-run-onboarding/connection-scopes.json"
    ))
    .unwrap();
    for fixture in fixtures.as_array().unwrap() {
        if fixture["scope"] != "local_system" {
            assert_eq!(fixture["localGuidance"], false);
        }
    }
}

#[test]
fn network_fixture_marks_only_active_connection_networks_available() {
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/first-run-onboarding/networks.json")).unwrap();
    for fixture in fixtures.as_array().unwrap() {
        assert_eq!(
            fixture["requiredResult"] == "available",
            fixture["active"] == true
        );
    }
}

#[test]
fn maps_each_missing_capability_without_hiding_other_results() {
    let report = HostReadinessService::report(
        cachyos_profile(),
        HostProbe {
            libvirt_access: true,
            qemu_emulator: false,
            kvm: true,
            uefi: false,
            secure_boot: false,
            forwarding_privilege: false,
        },
    );

    assert_eq!(report.overall_state, "degraded");
    assert_eq!(report.capabilities.len(), 6);
    assert_eq!(report.capabilities[0].state, CapabilityState::Available);
    assert_eq!(report.capabilities[1].state, CapabilityState::Unavailable);
    assert!(report.capabilities[1].remediation.is_some());
    assert_eq!(report.capabilities[2].state, CapabilityState::Available);
}

#[test]
fn reports_ready_only_when_required_host_capabilities_are_available() {
    let report = HostReadinessService::report(
        cachyos_profile(),
        HostProbe {
            libvirt_access: true,
            qemu_emulator: true,
            kvm: true,
            uefi: true,
            secure_boot: false,
            forwarding_privilege: false,
        },
    );

    assert_eq!(report.overall_state, "ready");
    assert_eq!(report.capabilities[4].state, CapabilityState::Unavailable);
}
