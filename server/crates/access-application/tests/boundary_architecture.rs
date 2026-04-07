use std::{fs, path::Path};

fn read_source(relative_path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)).unwrap()
}

#[test]
fn access_application_does_not_depend_on_external_published_fact_types() {
    let cargo_toml = read_source("Cargo.toml");
    let ports = read_source("src/ports.rs");
    let service = read_source("src/service.rs");

    assert!(
        !cargo_toml.contains("ordering-food-identity-published"),
        "access application must not depend on identity published contracts directly"
    );
    assert!(
        !cargo_toml.contains("ordering-food-organization-published"),
        "access application must not depend on organization published contracts directly"
    );

    for forbidden in [
        "ordering_food_identity_published",
        "ordering_food_organization_published",
        "SubjectRef",
        "StoreRef",
    ] {
        assert!(
            !ports.contains(forbidden),
            "access application port language must not reference external contract {forbidden}"
        );
        assert!(
            !service.contains(forbidden),
            "access service must not reference external contract {forbidden}"
        );
    }
}
