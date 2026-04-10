use std::{fs, path::Path};

fn read_source(relative_path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)).unwrap()
}

#[test]
fn composition_root_no_longer_assembles_access_service() {
    let composition_root = read_source("src/composition/mod.rs");

    let forbidden_imports = [
        "AppShellAccessSubjectProvider",
        "AppShellAccessStoreScopeProvider",
        "UserQueryService",
        "StoreQueryService",
        "SqlxUserReadRepository",
        "SqlxStoreReadRepository",
        "build_access_service",
    ];

    for forbidden_import in forbidden_imports {
        assert!(
            !composition_root.contains(forbidden_import),
            "composition root must not assemble access dependency {forbidden_import}"
        );
    }
}

#[test]
fn access_context_bootstraps_access_service_instead_of_app_shell() {
    let access_context = read_source("src/composition/contexts/access.rs");

    assert!(access_context.contains("ordering_food_access_integration"));
    assert!(access_context.contains("ordering_food_identity_published"));
    assert!(access_context.contains("ordering_food_organization_published"));
    assert!(access_context.contains("build_access_context_runtime"));
    assert!(access_context.contains(".capabilities"));
    assert!(access_context.contains(".publish(ACCESS_ORDER_MANAGEMENT_GATEWAY"));
    assert!(access_context.contains("IDENTITY_SUBJECT_LOOKUP_GATEWAY"));
    assert!(access_context.contains("ORGANIZATION_STORE_SCOPE_GATEWAY"));
    assert!(access_context.contains(".resolve::<Arc<dyn SubjectLookupGateway>>"));
    assert!(access_context.contains(".resolve::<Arc<dyn StoreScopeGateway>>"));
    assert!(access_context.contains("OrderManagementAccessGateway"));
    assert!(access_context.contains("retain_private"));
    assert!(!access_context.contains("build_subject_lookup_gateway"));
    assert!(!access_context.contains("build_store_scope_gateway"));
}

#[test]
fn fulfillment_route_does_not_depend_on_access_published_contract() {
    let route_source = read_source("src/routes/fulfillment.rs");

    assert!(!route_source.contains("ordering_food_access_published"));
    assert!(!route_source.contains("OrderManagementAccessGateway"));
    assert!(!route_source.contains("ordering_food_access_application::AccessService"));
}

#[test]
fn fulfillment_context_consumes_access_published_capability() {
    let context_source = read_source("src/composition/contexts/fulfillment.rs");

    assert!(context_source.contains(".capabilities"));
    assert!(context_source.contains(".resolve::<Arc<dyn OrderManagementAccessGateway>>"));
    assert!(context_source.contains("ACCESS_ORDER_MANAGEMENT_GATEWAY"));
    assert!(context_source.contains("OrderManagementAccessGateway"));
    assert!(!context_source.contains("build_access_service("));
    assert!(!context_source.contains("build_access_decision_gateway"));
}

#[test]
fn fulfillment_application_does_not_own_projector_transport_contracts() {
    let application_source = read_source("../../crates/fulfillment-application/src/ports.rs");

    for forbidden in ["OutboxMessage", "ProjectionCheckpointStore", "serde_json::Value"] {
        assert!(
            !application_source.contains(forbidden),
            "fulfillment application must not expose projector transport type {forbidden}"
        );
    }
}
