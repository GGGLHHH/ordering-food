use std::{fs, path::Path};

#[test]
fn application_ports_do_not_define_clock_trait_locally() {
    for relative_path in [
        "../../crates/identity-application/src/ports.rs",
        "../../crates/catalog-application/src/ports.rs",
        "../../crates/ordering-application/src/ports.rs",
    ] {
        let source =
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)).unwrap();
        assert!(!source.contains("pub trait Clock"));
    }
}

#[test]
fn api_platform_does_not_depend_on_identity_specific_id_generator() {
    let platform_source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/composition/platform.rs"),
    )
    .unwrap();
    let runtime_source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime.rs")).unwrap();

    assert!(!platform_source.contains("ordering_food_identity_application::IdGenerator"));
    assert!(!platform_source.contains("pub id_generator"));
    assert!(!runtime_source.contains("ordering_food_identity_application::IdGenerator"));
}
