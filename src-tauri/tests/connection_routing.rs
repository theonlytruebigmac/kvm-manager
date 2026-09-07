use kvm_manager_app_lib::models::operation::{OperationKind, TargetIdentity};
use kvm_manager_app_lib::services::connection_service::{
    ConnectionService, ConnectionType, SavedConnection,
};
use kvm_manager_app_lib::services::vm_service::VmService;
use std::path::PathBuf;
use virt::connect::Connect;
use virt::domain::Domain;

fn fixture_uri(fixture: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/hardening/connections")
        .join(fixture);
    format!("test://{}", path.display())
}

fn fixture_connection(id: &str, fixture: &str) -> SavedConnection {
    SavedConnection {
        id: id.to_string(),
        name: format!("Fixture {id}"),
        connection_type: ConnectionType::Local,
        hypervisor: "test".to_string(),
        host: None,
        username: None,
        ssh_port: None,
        tls_port: None,
        auto_connect: false,
        path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/hardening/connections")
            .join(fixture)
            .display()
            .to_string(),
    }
}

fn same_name_target() -> TargetIdentity {
    TargetIdentity {
        resource_kind: "vm".to_string(),
        stable_id: "same-name".to_string(),
        display_name: Some("same-name".to_string()),
    }
}

#[test]
fn same_named_resources_resolve_to_the_connection_captured_at_entry() {
    let connection_a = Connect::open(Some(&fixture_uri("fixture-a.xml"))).unwrap();
    let connection_b = Connect::open(Some(&fixture_uri("fixture-b.xml"))).unwrap();

    let domain_a = Domain::lookup_by_name(&connection_a, "same-name").unwrap();
    let domain_b = Domain::lookup_by_name(&connection_b, "same-name").unwrap();

    assert_eq!(
        domain_a.get_uuid_string().unwrap(),
        "00000000-0000-0000-0000-0000000000a1"
    );
    assert_eq!(
        domain_b.get_uuid_string().unwrap(),
        "00000000-0000-0000-0000-0000000000b2"
    );
}

#[test]
fn lifecycle_changes_stay_with_the_test_driver_connection() {
    let connection_a = Connect::open(Some(&fixture_uri("fixture-a.xml"))).unwrap();
    let connection_b = Connect::open(Some(&fixture_uri("fixture-b.xml"))).unwrap();
    let domain_a = Domain::lookup_by_name(&connection_a, "same-name").unwrap();
    let domain_b = Domain::lookup_by_name(&connection_b, "same-name").unwrap();

    assert!(domain_a.is_active().unwrap());
    assert!(domain_b.is_active().unwrap());

    domain_a.destroy().unwrap();

    assert!(!domain_a.is_active().unwrap());
    assert!(domain_b.is_active().unwrap());
}

#[test]
fn service_queries_mutates_and_refreshes_only_the_selected_fixture() {
    let service = ConnectionService::new();
    service
        .add_connection(fixture_connection("fixture-a", "fixture-a.xml"))
        .unwrap();
    service
        .add_connection(fixture_connection("fixture-b", "fixture-b.xml"))
        .unwrap();

    service.connect("fixture-a").unwrap();
    let query = service
        .resolve_operation(OperationKind::Query, Some(same_name_target()))
        .unwrap();
    assert_eq!(query.context.connection_id, "fixture-a");
    assert_eq!(
        VmService::list_vms(&query.connection)
            .unwrap()
            .into_iter()
            .find(|vm| vm.name == "same-name")
            .unwrap()
            .id,
        "00000000-0000-0000-0000-0000000000a1"
    );
    assert_eq!(
        Domain::lookup_by_name(&query.connection, "same-name")
            .unwrap()
            .get_uuid_string()
            .unwrap(),
        "00000000-0000-0000-0000-0000000000a1"
    );

    service.connect("fixture-b").unwrap();
    let mutation = service
        .resolve_operation(OperationKind::Mutation, Some(same_name_target()))
        .unwrap();
    assert_eq!(mutation.context.connection_id, "fixture-b");
    let domain = Domain::lookup_by_name(&mutation.connection, "same-name").unwrap();
    domain.destroy().unwrap();

    let refresh = service
        .resolve_operation(OperationKind::Query, Some(same_name_target()))
        .unwrap();
    assert_eq!(refresh.context.connection_id, "fixture-b");
    assert!(!Domain::lookup_by_name(&refresh.connection, "same-name")
        .unwrap()
        .is_active()
        .unwrap());
    assert!(Domain::lookup_by_name(&query.connection, "same-name")
        .unwrap()
        .is_active()
        .unwrap());
}

#[test]
fn captured_operation_survives_selection_change_and_disconnect_does_not_fall_back_to_local() {
    let service = ConnectionService::new();
    service
        .add_connection(fixture_connection("fixture-a", "fixture-a.xml"))
        .unwrap();
    service
        .add_connection(fixture_connection("fixture-b", "fixture-b.xml"))
        .unwrap();
    service.connect("fixture-a").unwrap();
    let captured = service
        .resolve_operation(OperationKind::Mutation, Some(same_name_target()))
        .unwrap();

    service.connect("fixture-b").unwrap();
    assert_eq!(captured.context.connection_id, "fixture-a");
    assert_eq!(
        Domain::lookup_by_name(&captured.connection, "same-name")
            .unwrap()
            .get_uuid_string()
            .unwrap(),
        "00000000-0000-0000-0000-0000000000a1"
    );

    service.disconnect("fixture-b").unwrap();
    assert!(service
        .resolve_operation(OperationKind::Query, Some(same_name_target()))
        .is_err());
}

#[test]
fn duplicate_connection_ids_are_rejected_before_they_change_selection() {
    let service = ConnectionService::new();
    service
        .add_connection(fixture_connection("fixture-a", "fixture-a.xml"))
        .unwrap();
    assert!(service
        .add_connection(fixture_connection("fixture-a", "fixture-b.xml"))
        .is_err());
}
