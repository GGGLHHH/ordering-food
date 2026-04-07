use std::{fs, path::Path};

#[test]
fn catalog_application_manifest_only_depends_on_catalog_domain_platform_kernel_and_published_scope_facts()
 {
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();

    assert!(manifest.contains("ordering-food-catalog-domain"));
    assert!(manifest.contains("ordering-food-platform-kernel"));
    assert!(manifest.contains("ordering-food-organization-published"));

    let forbidden = [
        "ordering-food-identity-application".to_string(),
        "ordering-food-identity-domain".to_string(),
        format!("ordering-food-{}-application", "menu"),
        format!("ordering-food-{}-domain", "menu"),
        "ordering-food-organization-application".to_string(),
        "ordering-food-organization-domain".to_string(),
        "ordering-food-organization-infrastructure".to_string(),
        "ordering-food-access-application".to_string(),
        "ordering-food-access-domain".to_string(),
        "ordering-food-fulfillment-application".to_string(),
        "ordering-food-fulfillment-domain".to_string(),
        "ordering-food-ordering-application".to_string(),
        "ordering-food-ordering-domain".to_string(),
    ];

    for forbidden in forbidden {
        assert!(
            !manifest.contains(&forbidden),
            "unexpected dependency in catalog application manifest: {forbidden}"
        );
    }
}

#[test]
fn catalog_application_sources_do_not_import_foreign_internal_layers() {
    for relative_path in [
        "src/error.rs",
        "src/lib.rs",
        "src/module.rs",
        "src/ports.rs",
        "src/use_cases/bootstrap_brand_catalog.rs",
        "src/use_cases/attach_store_catalog.rs",
        "src/use_cases/create_category.rs",
        "src/use_cases/create_item.rs",
        "src/use_cases/upsert_store_item_listing.rs",
    ] {
        let source =
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)).unwrap();

        let forbidden = [
            "ordering_food_organization_application".to_string(),
            "ordering_food_organization_domain".to_string(),
            "ordering_food_identity_application".to_string(),
            format!("ordering_food_{}_application", "menu"),
            "ordering_food_ordering_application".to_string(),
        ];

        for forbidden in forbidden {
            assert!(
                !source.contains(&forbidden),
                "{relative_path} unexpectedly imports foreign internal layer {forbidden}"
            );
        }
    }
}
