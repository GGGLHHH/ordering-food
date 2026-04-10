#[test]
fn application_dto_module_no_longer_reexports_published_models() {
    let source = std::fs::read_to_string("src/dto.rs").expect("dto source should exist");
    assert!(
        !source.contains("ordering_food_organization_published"),
        "application dto must not depend on published contracts"
    );
}

#[test]
fn application_boundary_no_longer_depends_on_published_read_models() {
    let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("manifest should exist");
    let ports = std::fs::read_to_string("src/ports.rs").expect("ports source should exist");

    assert!(
        !cargo_toml.contains("ordering-food-organization-published"),
        "organization application manifest must not depend on published contracts"
    );

    for forbidden in [
        "ordering_food_organization_published",
        "BrandRef",
        "StoreSummary",
    ] {
        assert!(
            !ports.contains(forbidden),
            "organization application ports must not expose published read model {forbidden}"
        );
    }
}
