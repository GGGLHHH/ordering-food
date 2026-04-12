use std::{fs, path::Path};

fn read_source(relative_path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)).unwrap()
}

#[test]
fn ordering_published_exposes_event_language_instead_of_temporary_sync_gateways() {
    let source = read_source("../../crates/ordering-published/src/lib.rs");

    assert!(source.contains("pub const COMMERCIAL_ORDER_PLACED_EVENT_TYPE"));
    assert!(source.contains("pub const COMMERCIAL_ORDER_STATUS_CHANGED_EVENT_TYPE"));
    assert!(source.contains("pub const COMMERCIAL_ORDER_CANCELLED_BY_CUSTOMER_EVENT_TYPE"));
    assert!(source.contains("pub struct CommercialOrderLineSnapshotV1"));
    assert!(source.contains("pub struct CommercialOrderPlacedV1"));
    assert!(source.contains("pub struct CommercialOrderStatusChangedV1"));
    assert!(source.contains("pub struct CommercialOrderCancelledByCustomerV1"));
    assert!(!source.contains("pub struct OrderPlaced"));
    assert!(!source.contains("pub struct OrderCommercialStateChanged"));
    assert!(!source.contains("pub struct OrderCancelledByCustomer"));
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
    let source = read_source("../../ARCHITECTURE.md");

    assert!(source.contains("长期目标态整改"));
    assert!(source.contains("integration runner 拉取 outbox 消息"));
    assert!(source.contains("application handler 在事务中完成本地投影更新"));
    assert!(source.contains("projector"));
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
fn ordering_application_no_longer_reexports_published_events() {
    let application_lib = read_source("../../crates/ordering-application/src/lib.rs");
    let application_ports = read_source("../../crates/ordering-application/src/ports.rs");
    let integration_lib = read_source("../../crates/ordering-integration/src/lib.rs");

    assert!(!application_lib.contains("ordering_food_ordering_published"));
    assert!(!application_lib.contains("CommercialOrderPlacedV1"));
    assert!(!application_lib.contains("CommercialOrderStatusChangedV1"));
    assert!(!application_lib.contains("CommercialOrderCancelledByCustomerV1"));
    assert!(!application_lib.contains("COMMERCIAL_ORDER_PLACED_EVENT_TYPE"));
    assert!(!application_lib.contains("COMMERCIAL_ORDER_STATUS_CHANGED_EVENT_TYPE"));
    assert!(!application_lib.contains("COMMERCIAL_ORDER_CANCELLED_BY_CUSTOMER_EVENT_TYPE"));
    assert!(!application_ports.contains("ordering_food_ordering_published"));
    assert!(!application_ports.contains("CommercialOrderPlacedV1"));
    assert!(!application_ports.contains("CommercialOrderStatusChangedV1"));
    assert!(!application_ports.contains("CommercialOrderCancelledByCustomerV1"));
    assert!(!application_ports.contains("COMMERCIAL_ORDER_PLACED_EVENT_TYPE"));
    assert!(!application_ports.contains("COMMERCIAL_ORDER_STATUS_CHANGED_EVENT_TYPE"));
    assert!(!application_ports.contains("COMMERCIAL_ORDER_CANCELLED_BY_CUSTOMER_EVENT_TYPE"));
    assert!(application_lib.contains("LocalCommercialOrderPlaced"));
    assert!(application_lib.contains("LocalCommercialOrderStatusChanged"));
    assert!(application_lib.contains("LocalCommercialOrderCancelledByCustomer"));
    assert!(application_lib.contains("OrderingEvent"));
    assert!(integration_lib.contains("published_event_adapter"));
    assert!(integration_lib.contains("AdapterBackedOrderingEventRecorder"));
}

#[test]
fn ordering_integration_no_longer_exports_sync_read_gateway_builder() {
    let source = read_source("../../crates/ordering-integration/src/lib.rs");

    assert!(!source.contains("build_commercial_order_read_gateway"));
    assert!(!source.contains("CommercialOrderReadGateway"));
}
