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
