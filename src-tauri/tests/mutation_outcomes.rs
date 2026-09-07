use kvm_manager_app_lib::models::operation::{
    ConnectionScope, MutationOutcome, MutationResult, OperationContext, OperationKind,
    TargetIdentity,
};
use kvm_manager_app_lib::models::vm::VmConfig;
use kvm_manager_app_lib::services::vm_service::VmService;
use virt::connect::Connect;
use virt::storage_pool::StoragePool;

fn mutation_context() -> OperationContext {
    OperationContext {
        operation_id: "op-multi-step".to_string(),
        operation_kind: OperationKind::Mutation,
        connection_id: "fixture-a".to_string(),
        connection_label: "Fixture A".to_string(),
        connection_scope: ConnectionScope::Test,
        capabilities: Vec::new(),
        target: None,
        captured_at: "2026-09-06T00:00:00Z".to_string(),
    }
}

fn mutation_target() -> TargetIdentity {
    TargetIdentity {
        resource_kind: "volume".to_string(),
        stable_id: "volume-fixture".to_string(),
        display_name: Some("Fixture volume".to_string()),
    }
}

#[test]
fn multi_step_mutations_report_every_terminal_outcome_explicitly() {
    let cases = [
        (MutationOutcome::Applied, "applied"),
        (MutationOutcome::Rejected, "rejected"),
        (MutationOutcome::RolledBack, "rolled_back"),
        (MutationOutcome::Partial, "partial"),
        (MutationOutcome::Unknown, "unknown"),
    ];

    for (outcome, expected_wire_value) in cases {
        let result = MutationResult::from_context(&mutation_context(), mutation_target(), outcome);
        let serialized = serde_json::to_value(&result).unwrap();

        assert_eq!(serialized["operationId"], "op-multi-step");
        assert_eq!(serialized["connectionId"], "fixture-a");
        assert_eq!(serialized["target"]["stableId"], "volume-fixture");
        assert_eq!(serialized["outcome"], expected_wire_value);
    }
}

#[test]
fn private_home_iso_contract_preserves_source_and_forbids_implicit_overwrite() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/first-run-onboarding/private-home-iso.json"
    ))
    .unwrap();
    assert_eq!(fixture["parentMode"], "0700");
    assert_eq!(fixture["transfer"], "libvirt_stream");
    assert_eq!(fixture["sourcePreserved"], true);
    assert_eq!(fixture["overwrite"], false);
}

#[test]
fn missing_pool_and_windows_capabilities_reject_before_domain_mutation() {
    let connection = Connect::open(Some("test:///default")).expect("libvirt test driver");
    let before = connection.list_all_domains(0).unwrap().len();
    let mut context = mutation_context();
    context.target = Some(TargetIdentity {
        resource_kind: "vm".to_string(),
        stable_id: "preflight-rejected".to_string(),
        display_name: Some("preflight-rejected".to_string()),
    });
    let config: VmConfig = serde_json::from_value(serde_json::json!({
        "name": "preflight-rejected",
        "cpuCount": 2,
        "memoryMb": 4096,
        "diskSizeGb": 64,
        "osType": "windows",
        "firmware": "uefi-secure",
        "tpmEnabled": true,
        "chipset": "q35",
        "cpuSockets": 1,
        "cpuCores": 2,
        "cpuThreads": 1,
        "network": "default",
        "installationType": "iso"
    }))
    .unwrap();

    assert!(VmService::create_vm(&connection, &context, config).is_err());
    assert_eq!(connection.list_all_domains(0).unwrap().len(), before);
}

fn basic_vm_config(name: &str, pool_id: Option<&str>, disk_size_gb: u64) -> VmConfig {
    serde_json::from_value(serde_json::json!({
        "name": name,
        "cpuCount": 1,
        "memoryMb": 1024,
        "diskSizeGb": disk_size_gb,
        "storagePoolId": pool_id,
        "firmware": "bios",
        "cpuSockets": 1,
        "cpuCores": 1,
        "cpuThreads": 1,
        "network": "default",
        "installationType": "manual"
    }))
    .unwrap()
}

#[test]
fn missing_stale_inactive_and_undersized_pools_leave_no_vm_or_volume() {
    let connection = Connect::open(Some("test:///default")).expect("libvirt test driver");
    let pool = StoragePool::lookup_by_name(&connection, "default-pool").unwrap();
    let pool_id = pool.get_uuid_string().unwrap();
    let before_domains = connection.list_all_domains(0).unwrap().len();
    let before_volumes = pool.list_volumes().unwrap().len();

    for (name, selected, size) in [
        ("missing-selection", None, 1),
        (
            "stale-selection",
            Some("00000000-0000-0000-0000-000000000000"),
            1,
        ),
        ("undersized-selection", Some(pool_id.as_str()), 101),
    ] {
        assert!(VmService::create_vm(
            &connection,
            &mutation_context(),
            basic_vm_config(name, selected, size),
        )
        .is_err());
        assert_eq!(
            connection.list_all_domains(0).unwrap().len(),
            before_domains
        );
        assert_eq!(pool.list_volumes().unwrap().len(), before_volumes);
    }

    pool.destroy().unwrap();
    let inactive_result = VmService::create_vm(
        &connection,
        &mutation_context(),
        basic_vm_config("inactive-selection", Some(&pool_id), 1),
    );
    pool.create(0).unwrap();
    assert!(inactive_result.is_err());
    assert_eq!(
        connection.list_all_domains(0).unwrap().len(),
        before_domains
    );
    assert_eq!(pool.list_volumes().unwrap().len(), before_volumes);
}
