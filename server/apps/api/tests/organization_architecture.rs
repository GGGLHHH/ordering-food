use std::{fs, path::Path};

#[test]
fn api_registers_organization_before_catalog() {
    let contexts_source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/composition/contexts/mod.rs"),
    )
    .unwrap();
    let organization_source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/composition/contexts/organization.rs"),
    )
    .unwrap();

    let organization_registration = contexts_source
        .find("organization::register_organization()")
        .expect("organization context should be registered");
    let catalog_registration = contexts_source
        .find("catalog::register_catalog()")
        .expect("catalog context should be registered");

    assert!(contexts_source.contains("mod organization;"));
    assert!(contexts_source.contains("mod catalog;"));
    assert!(
        organization_registration < catalog_registration,
        "organization must be registered before catalog"
    );
    assert!(
        !organization_source.contains("add_route_mount("),
        "organization context must not mount routes"
    );
}

#[test]
fn organization_context_makes_default_seed_explicit_after_build() {
    let organization_source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/composition/contexts/organization.rs"),
    )
    .unwrap();

    assert!(organization_source.contains("ordering_food_organization_integration"));
    assert!(organization_source.contains("build_organization_context_runtime"));
    assert!(organization_source.contains("seed_default_organization"));
    assert!(!organization_source.contains("build_bootstrapped_organization_module"));
    assert!(!organization_source.contains("ordering_food_organization_infrastructure_sqlx"));
    assert!(!organization_source.contains("ordering_food_organization_application"));
    assert!(!organization_source.contains("seed_organization_if_empty"));
    assert!(!organization_source.contains("CreateBrandInput"));
    assert!(!organization_source.contains("CreateStoreInput"));
    assert!(!organization_source.contains("DEFAULT_BRAND_ID"));
    assert!(!organization_source.contains("OrganizationClockAdapter"));
    assert!(!organization_source.contains("UuidV4OrganizationIdGenerator"));
    assert!(!organization_source.contains("#[cfg(test)]"));
    assert!(organization_source.contains("ORGANIZATION_STORE_SCOPE_GATEWAY"));
    assert!(organization_source.contains("ORGANIZATION_BRAND_LOOKUP_GATEWAY"));
    assert!(organization_source.contains(".publish("));
}

#[test]
fn organization_published_gateways_use_provider_query_facades() {
    let organization_source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/composition/contexts/organization.rs"),
    )
    .unwrap();
    let integration_source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/organization-integration/src/lib.rs"),
    )
    .unwrap();
    let application_source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/organization-application/src/ports.rs"),
    )
    .unwrap();

    assert!(application_source.contains("BrandReadRepository"));
    assert!(application_source.contains("BrandQueryService"));
    assert!(integration_source.contains("BrandQueryService"));
    assert!(!integration_source.contains("sqlx::query("));
    assert!(!integration_source.contains("fetch_optional(&self.pg_pool)"));
    assert!(organization_source.contains("runtime.brand_lookup_gateway"));
    assert!(!organization_source.contains("build_brand_lookup_gateway(pg_pool)"));
}
