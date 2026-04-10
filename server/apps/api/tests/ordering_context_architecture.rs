use std::{fs, path::Path};

#[test]
fn workspace_members_include_ordering_context_crates() {
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml")).unwrap();

    for member in [
        "crates/ordering-domain",
        "crates/ordering-application",
        "crates/ordering-infrastructure-sqlx",
    ] {
        assert!(
            manifest.contains(member),
            "missing workspace member: {member}"
        );
    }
}

#[test]
fn route_modules_separate_ordering_and_fulfillment_http_contracts() {
    let mod_source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/routes/mod.rs"))
            .unwrap();

    assert!(mod_source.contains("pub mod ordering;"));
    assert!(mod_source.contains("pub mod fulfillment;"));
    assert!(!mod_source.contains("pub mod orders;"));
}

#[test]
fn route_modules_do_not_reference_legacy_order_sqlx_infrastructure() {
    for relative_path in [
        "src/routes/api.rs",
        "src/routes/auth.rs",
        "src/routes/catalog.rs",
        "src/routes/fulfillment.rs",
        "src/routes/health.rs",
        "src/routes/identity.rs",
        "src/routes/ordering.rs",
        "src/routes/mod.rs",
        "src/http.rs",
    ] {
        let source =
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)).unwrap();
        assert!(!source.contains("ordering_food_order_infrastructure_sqlx"));
    }
}

#[test]
fn fulfillment_route_module_owns_its_http_contract() {
    let source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/routes/fulfillment.rs"))
            .unwrap();

    assert!(source.contains("pub struct FulfillmentOrderResponse"));
    assert!(!source.contains("use super::ordering::{"));
    assert!(!source.contains("load_order_response"));
    assert!(!source.contains("map_ordering_error"));
}

#[test]
fn workspace_members_no_longer_include_legacy_order_crates() {
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml")).unwrap();

    for member in [
        "crates/order-domain",
        "crates/order-application",
        "crates/order-infrastructure-sqlx",
    ] {
        assert!(
            !manifest.contains(member),
            "legacy member still present: {member}"
        );
    }
}

#[test]
fn ordering_context_bootstraps_runtime_through_integration_boundary() {
    let source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/composition/contexts/ordering.rs"),
    )
    .unwrap();

    assert!(source.contains("ordering_food_ordering_integration"));
    assert!(source.contains("build_ordering_context_runtime"));
    assert!(!source.contains("ordering_food_ordering_infrastructure_sqlx"));
    assert!(!source.contains("build_ordering_module"));
    assert!(!source.contains("UuidV4OrderIdGenerator"));
}

#[test]
fn ordering_route_does_not_hold_customer_visibility_logic() {
    let source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/routes/ordering.rs"))
            .unwrap();
    let production_source = source.split("#[cfg(test)]").next().unwrap_or(&source);

    assert!(production_source.contains("get_by_id_for_customer"));
    assert!(!production_source.contains("customer_id.is_some_and"));
    assert!(!production_source.contains("customer_id != order.customer_id"));
    assert!(!production_source.contains(".get_by_id(order_id)"));
}
