use std::{fs, path::Path};

fn read_source(relative_path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)).unwrap()
}

#[test]
fn ordering_published_exposes_event_language_instead_of_temporary_sync_gateways() {
    let source = read_source("../../crates/ordering-published/src/lib.rs");

    assert!(source.contains("OrderPlaced"));
    assert!(source.contains("OrderCommercialStateChanged"));
    assert!(source.contains("OrderCancelledByCustomer"));
    assert!(!source.contains("CommercialOrderReadGateway"));
    assert!(!source.contains("CommercialOrderSnapshot"));
    assert!(!source.contains("CommercialOrderItemSnapshot"));
    assert!(!source.contains("OrderRef"));
    assert!(!source.contains("SyncCollaborationError"));
    assert!(!source.contains("TemporarySyncFulfillmentWorkflowGateway"));
    assert!(!source.contains("TemporarySyncFulfillmentBootstrapGateway"));
}

#[test]
fn target_architecture_doc_requires_phase_3_event_and_projection_path() {
    let source = read_source(
        "../../../docs/superpowers/specs/2026-04-05-backend-ddd-target-architecture-design.md",
    );

    assert!(source.contains("Phase 3"));
    assert!(source.contains("事件协作替换"));
    assert!(source.contains("Postgres outbox"));
    assert!(source.contains("dispatcher / projector"));
    assert!(source.contains("本地投影"));
}

#[test]
fn ordering_http_layer_does_not_pull_fulfillment_read_side_directly() {
    let ordering_routes = read_source("src/routes/ordering.rs");
    let ordering_context = read_source("src/composition/contexts/ordering.rs");

    assert!(!ordering_routes.contains("WorkflowOrderQueryService"));
    assert!(!ordering_routes.contains("WorkflowOrderReadModel"));
    assert!(!ordering_routes.contains("map_workflow_query_error"));
    assert!(!ordering_context.contains("SqlxWorkflowOrderReadRepository"));
}

#[test]
fn ordering_context_no_longer_wires_fulfillment_sync_bootstrap() {
    let ordering_context = read_source("src/composition/contexts/ordering.rs");

    assert!(!ordering_context.contains("ordering_food_fulfillment_integration"));
    assert!(!ordering_context.contains("build_temporary_sync_fulfillment_bootstrap_gateway"));
}

#[test]
fn ordering_application_records_published_events_inside_local_transaction() {
    let ordering_application =
        read_source("../../crates/ordering-application/src/use_cases/place_order_from_cart.rs");
    let ordering_cancellation =
        read_source("../../crates/ordering-application/src/use_cases/cancel_order_by_customer.rs");

    assert!(ordering_application.contains("record_order_placed"));
    assert!(ordering_cancellation.contains("record_order_commercial_state_changed"));
    assert!(ordering_cancellation.contains("record_order_cancelled_by_customer"));
    assert!(!ordering_application.contains("bootstrap_order_placed"));
    assert!(!ordering_cancellation.contains("bootstrap_order_cancelled_by_customer"));
}

#[test]
fn ordering_integration_no_longer_exports_sync_read_gateway_builder() {
    let source = read_source("../../crates/ordering-integration/src/lib.rs");

    assert!(!source.contains("build_commercial_order_read_gateway"));
    assert!(!source.contains("CommercialOrderReadGateway"));
}
