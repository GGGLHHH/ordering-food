use std::{fs, path::Path};

#[test]
fn fulfillment_application_stays_store_workflow_only() {
    let source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/fulfillment-application/src/use_cases/mod.rs"),
    )
    .unwrap();

    assert!(!source.contains("place_order_from_cart"));
    assert!(!source.contains("cancel_order_by_customer"));
}

#[test]
fn ordering_context_no_longer_uses_fulfillment_sync_bootstrap() {
    let source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/composition/contexts/ordering.rs"),
    )
    .unwrap();

    assert!(!source.contains("build_temporary_sync_fulfillment_bootstrap_gateway"));
    assert!(!source.contains("ordering_food_fulfillment_integration"));
}

#[test]
fn fulfillment_context_uses_local_projection_and_event_projector() {
    let source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/composition/contexts/fulfillment.rs"),
    )
    .unwrap();

    assert!(source.contains("build_fulfillment_context_runtime"));
    assert!(source.contains("ordering_food_fulfillment_integration"));
    assert!(!source.contains("build_commercial_order_read_gateway"));
    assert!(!source.contains("ordering_food_ordering_integration"));
    assert!(!source.contains("ordering_food_fulfillment_infrastructure_sqlx"));
    assert!(!source.contains("build_fulfillment_module("));
    assert!(!source.contains("build_ordering_event_projector"));
}
