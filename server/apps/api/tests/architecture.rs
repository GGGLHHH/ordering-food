use std::{fs, path::Path};

#[test]
fn route_modules_do_not_reference_identity_sqlx_infrastructure() {
    for relative_path in [
        "src/routes/api.rs",
        "src/routes/health.rs",
        "src/routes/identity.rs",
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
        "src/routes/health.rs",
        "src/routes/identity.rs",
        "src/routes/mod.rs",
        "src/runtime.rs",
    ] {
        let source =
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)).unwrap();
        assert!(!source.contains("ordering_food_identity_infrastructure_sqlx"));
    }
}

#[test]
fn app_state_does_not_store_context_modules() {
    let source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/app.rs")).unwrap();
    assert!(!source.contains("IdentityModule"));
    assert!(!source.contains("ordering_module"));
}
