use kvm_manager_app_lib::models::network::NetworkConfig;
use kvm_manager_app_lib::models::nwfilter::{
    NwFilterConfig, NwFilterRule, RuleAction, RuleDirection,
};
use kvm_manager_app_lib::models::storage::{StoragePoolConfig, VolumeConfig};
use kvm_manager_app_lib::services::network_service::NetworkService;
use kvm_manager_app_lib::services::nwfilter_service::NwFilterService;
use kvm_manager_app_lib::services::storage_service::StorageService;
use kvm_manager_app_lib::utils::xml::validate_document_root;

fn baseline_config() -> NetworkConfig {
    NetworkConfig {
        name: "isolated-lab".to_string(),
        bridge_name: "virbr42".to_string(),
        forward_mode: "nat".to_string(),
        ip_address: "192.0.2.1".to_string(),
        netmask: "255.255.255.0".to_string(),
        dhcp_start: "192.0.2.10".to_string(),
        dhcp_end: "192.0.2.200".to_string(),
        ipv6_enabled: false,
        ipv6_address: None,
        ipv6_prefix: None,
        ipv6_dhcp_start: None,
        ipv6_dhcp_end: None,
        autostart: false,
    }
}

#[test]
fn network_configuration_corpus_rejects_unsafe_values_before_mutation() {
    let malicious_values = (0..100)
        .map(|index| match index % 10 {
            0 => format!("network</name><bridge name='owned-{index}'/>"),
            1 => format!("quote'\"-{index}"),
            2 => format!("unicode-雪-{index}"),
            3 => format!("control\u{0007}-{index}"),
            4 => format!("../path-{index}"),
            5 => format!("space value-{index}"),
            6 => format!("ampersand&-{index}"),
            7 => format!("comment<!---{index}"),
            8 => "a".repeat(129),
            _ => format!("newline\n-{index}"),
        })
        .collect::<Vec<_>>();

    for value in malicious_values {
        let mut config = baseline_config();
        config.name = value;
        let result = NetworkService::network_definition(&config);

        // Building is the last pre-mutation step. An error proves that no libvirt definition or
        // start operation was attempted for that corpus member.
        assert!(
            result.is_err(),
            "unsafe value unexpectedly produced a definition"
        );
    }
}

#[test]
fn safe_network_definition_has_one_network_root_and_no_extra_structure() {
    let definition = NetworkService::network_definition(&baseline_config()).unwrap();
    validate_document_root(&definition, "network").unwrap();
    assert_eq!(definition.matches("<bridge").count(), 1);
    assert_eq!(definition.matches("<forward").count(), 1);
    assert_eq!(definition.matches("<range").count(), 1);
}

#[test]
fn filter_configuration_rejects_untrusted_structure_before_mutation() {
    let safe_rule = NwFilterRule {
        direction: RuleDirection::In,
        action: RuleAction::Accept,
        priority: Some(10),
        protocol: Some("tcp".to_string()),
        src_ip: Some("192.0.2.10/24".to_string()),
        src_mac: Some("52:54:00:12:34:56".to_string()),
        dest_ip: None,
        dest_mac: None,
        src_port: Some("80-90".to_string()),
        dest_port: Some("443".to_string()),
        comment: Some("safe & documented".to_string()),
    };
    let config = NwFilterConfig {
        name: "isolated-filter".to_string(),
        chain: Some("ipv4".to_string()),
        priority: Some(10),
        rules: vec![safe_rule],
        filter_refs: vec!["clean-traffic".to_string()],
    };
    let definition = NwFilterService::config_to_xml(&config).unwrap();
    validate_document_root(&definition, "filter").unwrap();
    assert!(definition.contains("srcportstart='80' srcportend='90'"));
    assert!(definition.contains("comment='safe &amp; documented'"));

    let invalid_configs = [
        NwFilterConfig {
            name: "filter'><rule action='accept'/>".to_string(),
            ..config.clone()
        },
        NwFilterConfig {
            filter_refs: vec!["ref'/><rule action='accept'/>".to_string()],
            ..config.clone()
        },
        NwFilterConfig {
            rules: vec![NwFilterRule {
                protocol: Some("tcp/><rule action='accept'".to_string()),
                ..config.rules[0].clone()
            }],
            ..config.clone()
        },
        NwFilterConfig {
            rules: vec![NwFilterRule {
                src_ip: Some("192.0.2.1'/><rule action='accept'".to_string()),
                ..config.rules[0].clone()
            }],
            ..config.clone()
        },
        NwFilterConfig {
            rules: vec![NwFilterRule {
                comment: Some("comment -- injection".to_string()),
                protocol: Some("all".to_string()),
                src_ip: None,
                src_mac: None,
                src_port: None,
                dest_ip: None,
                dest_mac: None,
                dest_port: None,
                ..config.rules[0].clone()
            }],
            ..config
        },
    ];

    for invalid in invalid_configs {
        assert!(
            NwFilterService::config_to_xml(&invalid).is_err(),
            "unsafe filter input unexpectedly produced a definition"
        );
    }
}

#[test]
fn storage_configuration_emits_only_data_and_rejects_injected_structure() {
    let pool = StoragePoolConfig {
        name: "vm-images".to_string(),
        pool_type: "dir".to_string(),
        target_path: "/var/lib/libvirt/images & archive".to_string(),
        autostart: false,
        source_devices: Vec::new(),
        source_host: None,
        source_path: None,
        iscsi_target: None,
        initiator_iqn: None,
        gluster_volume: None,
        rbd_pool: None,
        ceph_monitors: Vec::new(),
        ceph_auth_user: None,
        ceph_auth_secret: None,
    };
    let pool_definition = StorageService::pool_definition(&pool).unwrap();
    validate_document_root(&pool_definition, "pool").unwrap();
    assert!(pool_definition.contains("&amp; archive"));

    let volume = VolumeConfig {
        name: "disk-01.qcow2".to_string(),
        capacity_gb: 20,
        format: "qcow2".to_string(),
        encrypted: false,
        passphrase: None,
    };
    let volume_definition = StorageService::volume_definition(&volume, 1024, None).unwrap();
    validate_document_root(&volume_definition, "volume").unwrap();

    let invalid_pool = StoragePoolConfig {
        name: "images</name><source/>".to_string(),
        ..pool
    };
    let invalid_volume = VolumeConfig {
        name: "disk</name><target/>".to_string(),
        ..volume
    };
    assert!(StorageService::pool_definition(&invalid_pool).is_err());
    assert!(StorageService::volume_definition(&invalid_volume, 1024, None).is_err());
}
