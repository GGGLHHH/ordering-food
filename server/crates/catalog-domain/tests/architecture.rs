use std::{fs, path::Path};

#[test]
fn domain_manifest_does_not_depend_on_framework_or_infrastructure_crates() {
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();

    for forbidden in [
        "axum",
        "sqlx",
        "redis",
        "tracing",
        "config",
        "serde_json",
        "anyhow",
    ] {
        assert!(!manifest.contains(&format!("{forbidden}.workspace")));
        assert!(!manifest.contains(&format!("{forbidden} =")));
    }
}

#[test]
fn domain_sources_do_not_depend_on_organization_published_or_define_store_aggregate() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

    for entry in fs::read_dir(src_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }

        let source = fs::read_to_string(&path).unwrap();

        assert!(
            !source.contains("ordering_food_organization_published"),
            "{} must not depend on organization published contracts",
            path.display()
        );
        assert!(
            !source.contains("pub struct Store {"),
            "{} must not define a Store aggregate",
            path.display()
        );
        assert!(
            !source.contains("\nstruct Store {"),
            "{} must not define a private Store aggregate",
            path.display()
        );
    }
}
