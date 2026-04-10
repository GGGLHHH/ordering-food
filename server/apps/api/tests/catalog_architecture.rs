use std::{fs, path::Path};

#[test]
fn catalog_routes_use_catalog_prefix() {
    let source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/routes/catalog.rs"))
            .unwrap();
    let legacy_route_prefix = format!("/api/{}", "menu");

    assert!(source.contains("pub(crate) const CATALOG_ROUTE_PREFIX: &str = \"/api/catalog\";"));
    assert!(!source.contains(&legacy_route_prefix));
}

#[test]
fn catalog_api_consumes_organization_gateway_instead_of_internal_queries() {
    let route_source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/routes/catalog.rs"))
            .unwrap();
    let context_source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/composition/contexts/catalog.rs"),
    )
    .unwrap();

    for forbidden in [
        "StoreQueryService",
        "SqlxStoreReadRepository",
        "ordering_food_organization_application",
        "ordering_food_organization_infrastructure_sqlx",
    ] {
        assert!(
            !route_source.contains(forbidden),
            "catalog route layer must not depend on organization internal query type {forbidden}"
        );
        assert!(
            !context_source.contains(forbidden),
            "catalog context wiring must not depend on organization internal query type {forbidden}"
        );
    }

    assert!(context_source.contains("ordering_food_organization_published"));
    assert!(context_source.contains("ORGANIZATION_BRAND_LOOKUP_GATEWAY"));
    assert!(context_source.contains("ORGANIZATION_STORE_SCOPE_GATEWAY"));
    assert!(context_source.contains(".resolve::<Arc<dyn BrandLookupGateway>>"));
    assert!(context_source.contains(".resolve::<Arc<dyn StoreScopeGateway>>"));
    assert!(!context_source.contains("build_brand_lookup_gateway"));
    assert!(!context_source.contains("build_store_scope_gateway"));
    assert!(!route_source.contains("StoreScopeGateway"));
}

#[test]
fn catalog_context_makes_seed_explicit_after_runtime_build() {
    let context_source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/composition/contexts/catalog.rs"),
    )
    .unwrap();

    assert!(context_source.contains("ordering_food_catalog_integration"));
    assert!(context_source.contains("build_catalog_context_runtime"));
    assert!(context_source.contains("seed_default_catalog"));
    assert!(!context_source.contains("build_seeded_catalog_sqlx_module"));
    assert!(!context_source.contains("ordering_food_catalog_infrastructure_sqlx"));

    for forbidden in [
        "seed_catalog_if_empty",
        "ensure_brand_catalog",
        "ensure_store_catalog",
        "ensure_category",
        "ensure_item",
        "ApiOrganizationScopeReader",
    ] {
        assert!(
            !context_source.contains(forbidden),
            "catalog context must not retain bootstrap orchestration detail {forbidden}"
        );
    }
}

#[test]
fn catalog_route_does_not_own_active_catalog_navigation() {
    let route_source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/routes/catalog.rs"))
            .unwrap();
    let production_source = route_source.split("#[cfg(test)]").next().unwrap();

    assert!(!production_source.contains("load_active_catalog_context("));
    assert!(!production_source.contains("get_active()"));
}

#[test]
fn catalog_application_owns_active_catalog_query_facade() {
    let application_source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/catalog-application/src/ports.rs"),
    )
    .unwrap();

    assert!(application_source.contains("ActiveCatalogQueryService"));
    assert!(application_source.contains("get_active_store"));
}

#[test]
fn catalog_application_ports_do_not_expose_organization_published_models() {
    let application_source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/catalog-application/src/ports.rs"),
    )
    .unwrap();

    assert!(!application_source.contains("ordering_food_organization_published"));
    assert!(!application_source.contains("StoreSummary"));
    assert!(!application_source.contains("BrandRef"));
}

#[test]
fn catalog_schema_does_not_hold_cross_context_foreign_keys() {
    let migration_source = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../crates/database-infrastructure-sqlx/migrations/202604050301_catalog_context.up.sql",
    ))
    .unwrap();

    assert!(!migration_source.contains("REFERENCES organization."));
}

#[test]
fn catalog_integration_owns_organization_scope_acl_translation() {
    let integration_lib_source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/catalog-integration/src/lib.rs"),
    )
    .unwrap();
    let integration_acl_source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/catalog-integration/src/organization_scope_acl.rs"),
    )
    .unwrap();

    assert!(integration_lib_source.contains("mod organization_scope_acl;"));
    assert!(integration_lib_source.contains("ordering_food_organization_published"));
    assert!(integration_lib_source.contains("CatalogOrganizationScopeAclAdapter"));
    assert!(!integration_lib_source.contains("pub mod organization_scope_adapter"));
    assert!(!integration_lib_source.contains("CatalogOrganizationScopeReader"));
    assert!(integration_acl_source.contains("CatalogBrandScope"));
    assert!(integration_acl_source.contains("CatalogStoreScope"));
    assert!(integration_acl_source.contains("impl OrganizationScopeReader"));
    assert!(integration_acl_source.contains("BrandLookupGateway"));
    assert!(integration_acl_source.contains("StoreScopeGateway"));
    assert!(integration_acl_source.contains("BrandRef"));
    assert!(integration_acl_source.contains("StoreSummary"));
}
