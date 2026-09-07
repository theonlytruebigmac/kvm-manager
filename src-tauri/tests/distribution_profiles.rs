use kvm_manager_app_lib::models::host::DistributionFamily;
use kvm_manager_app_lib::services::distribution_profile_service::DistributionProfileService;

#[test]
fn parses_quoted_and_unquoted_os_release_values() {
    let values = DistributionProfileService::parse_os_release(
        "# comment\nID=cachyos\nPRETTY_NAME=\"CachyOS Linux\"\n",
    );

    assert_eq!(values.get("ID"), Some(&"cachyos".to_string()));
    assert_eq!(
        values.get("PRETTY_NAME"),
        Some(&"CachyOS Linux".to_string())
    );
}

#[test]
fn classifies_every_supported_distribution_family() {
    let cases = [
        (
            include_str!("fixtures/os-release/cachyos"),
            DistributionFamily::ArchCachyos,
            "pacman",
        ),
        (
            include_str!("fixtures/os-release/ubuntu-24.04"),
            DistributionFamily::DebianUbuntu,
            "apt",
        ),
        (
            include_str!("fixtures/os-release/fedora-42"),
            DistributionFamily::FedoraRhel,
            "dnf",
        ),
        (
            include_str!("fixtures/os-release/opensuse-tumbleweed"),
            DistributionFamily::Opensuse,
            "zypper",
        ),
    ];

    for (fixture, family, package_manager) in cases {
        let profile = DistributionProfileService::classify_os_release(fixture);
        assert!(profile.supported);
        assert_eq!(profile.family, family);
        assert_eq!(profile.package_manager, package_manager);
        assert!(!profile.packages.is_empty());
    }
}

#[test]
fn unsupported_distribution_gets_no_foreign_package_commands() {
    let profile = DistributionProfileService::classify_os_release(include_str!(
        "fixtures/os-release/unsupported"
    ));

    assert_eq!(profile.family, DistributionFamily::BestEffort);
    assert!(!profile.supported);
    assert_eq!(profile.package_manager, "unknown");
    assert!(profile.packages.is_empty());
}
