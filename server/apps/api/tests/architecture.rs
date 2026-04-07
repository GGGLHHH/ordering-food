use std::{fs, path::Path};

#[test]
fn route_modules_do_not_reference_identity_sqlx_infrastructure() {
    for relative_path in [
        "src/routes/api.rs",
        "src/routes/auth.rs",
        "src/routes/catalog.rs",
        "src/routes/fulfillment.rs",
        "src/routes/health.rs",
        "src/routes/identity.rs",
        "src/routes/ordering.rs",
        "src/routes/mod.rs",
        "src/http.rs",
    ] {
        let source =
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)).unwrap();
        assert!(!source.contains("ordering_food_identity_infrastructure_sqlx"));
    }
}

#[test]
fn only_composition_modules_reference_identity_sqlx_infrastructure() {
    let composition_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/composition");
    for entry in fs::read_dir(&composition_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            for nested in fs::read_dir(path).unwrap() {
                let nested = nested.unwrap();
                let nested_path = nested.path();
                if nested_path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                    let _ = fs::read_to_string(nested_path).unwrap();
                }
            }
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            let _ = fs::read_to_string(path).unwrap();
        }
    }

    for relative_path in [
        "src/app.rs",
        "src/config.rs",
        "src/error.rs",
        "src/http.rs",
        "src/lib.rs",
        "src/main.rs",
        "src/observability.rs",
        "src/readiness.rs",
        "src/routes/api.rs",
        "src/routes/auth.rs",
        "src/routes/catalog.rs",
        "src/routes/fulfillment.rs",
        "src/routes/health.rs",
        "src/routes/identity.rs",
        "src/routes/ordering.rs",
        "src/routes/mod.rs",
        "src/runtime.rs",
    ] {
        let source =
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)).unwrap();
        assert!(!source.contains("ordering_food_identity_infrastructure_sqlx"));
    }
}

#[test]
fn app_shell_no_longer_references_authz_crates() {
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();

    for dependency in [
        "ordering-food-authz-application",
        "ordering-food-authz-domain",
        "ordering-food-authz-infrastructure-sqlx",
    ] {
        assert!(
            !manifest.contains(dependency),
            "unexpected legacy authz dependency: {dependency}"
        );
    }
}

#[test]
fn context_registry_only_registers_access_not_authz() {
    let source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/composition/contexts/mod.rs"),
    )
    .unwrap();

    assert!(source.contains("mod access;"));
    assert!(source.contains("access::register_access()"));
    assert!(!source.contains("mod authz;"));
    assert!(!source.contains("authz::register_authz()"));
}

#[test]
fn app_state_does_not_store_context_modules() {
    let source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/app.rs")).unwrap();
    assert!(!source.contains("IdentityModule"));
    assert!(!source.contains("ordering_module"));
}
