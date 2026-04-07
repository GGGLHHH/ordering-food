use std::{fs, path::Path};

fn read_source(relative_path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)).unwrap()
}

#[test]
fn access_integration_consumes_context_gateways_instead_of_internal_layers() {
    let cargo_toml = read_source("Cargo.toml");
    let lib_rs = read_source("src/lib.rs");

    for forbidden in [
        "ordering-food-identity-application",
        "ordering-food-identity-domain",
        "ordering-food-identity-infrastructure-sqlx",
        "ordering-food-organization-application",
        "ordering-food-organization-domain",
        "ordering-food-organization-infrastructure-sqlx",
    ] {
        assert!(
            !cargo_toml.contains(forbidden),
            "access integration must not depend on internal layer {forbidden}"
        );
    }

    for forbidden in [
        "ordering-food-identity-integration",
        "ordering-food-organization-integration",
    ] {
        assert!(
            !cargo_toml.contains(forbidden),
            "access integration must not depend on sibling integration crate {forbidden}"
        );
    }

    for required in [
        "ordering-food-identity-published",
        "ordering-food-organization-published",
    ] {
        assert!(
            cargo_toml.contains(required),
            "access integration must depend on published/integration contract {required}"
        );
    }

    for forbidden in [
        "UserQueryService",
        "StoreQueryService",
        "SqlxUserReadRepository",
        "SqlxStoreReadRepository",
        "build_subject_lookup_gateway",
        "build_store_scope_gateway",
    ] {
        assert!(
            !lib_rs.contains(forbidden),
            "access integration must not wire internal implementation {forbidden}"
        );
    }
}
