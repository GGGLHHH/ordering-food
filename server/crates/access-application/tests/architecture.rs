use std::{fs, path::Path};

#[test]
fn access_application_manifest_only_depends_on_its_own_fact_language() {
    let manifest = fs::read_to_string("Cargo.toml").unwrap();

    assert!(!manifest.contains("ordering-food-identity-application"));
    assert!(!manifest.contains("ordering-food-organization-application"));
    assert!(!manifest.contains("ordering-food-identity-infrastructure"));
    assert!(!manifest.contains("ordering-food-organization-infrastructure"));
    assert!(!manifest.contains("ordering-food-identity-published"));
    assert!(!manifest.contains("ordering-food-organization-published"));
}

#[test]
fn access_application_sources_do_not_import_foreign_internal_layers() {
    for relative_path in [
        "src/error.rs",
        "src/facts.rs",
        "src/lib.rs",
        "src/ports.rs",
        "src/service.rs",
    ] {
        let source =
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)).unwrap();

        for forbidden in [
            "ordering_food_identity_application",
            "ordering_food_identity_domain",
            "ordering_food_identity_infrastructure",
            "ordering_food_organization_application",
            "ordering_food_organization_domain",
            "ordering_food_organization_infrastructure",
        ] {
            assert!(
                !source.contains(forbidden),
                "{relative_path} unexpectedly imports foreign internal layer {forbidden}"
            );
        }
    }
}
