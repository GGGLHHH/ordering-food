use std::{
    fs,
    path::{Path, PathBuf},
};

const CONTEXTS: &[&str] = &[
    "access",
    "catalog",
    "fulfillment",
    "identity",
    "ordering",
    "organization",
];

#[test]
fn shared_and_platform_kernels_do_not_depend_on_business_context_crates() {
    for crate_name in ["shared-kernel", "platform-kernel"] {
        let crate_dir = workspace_root().join("crates").join(crate_name).join("src");

        for source_file in rust_files_in(&crate_dir) {
            let source = read_source(&source_file);

            for context in CONTEXTS {
                for layer in [
                    "domain",
                    "application",
                    "infrastructure",
                    "integration",
                    "published",
                ] {
                    let forbidden = format!("ordering_food_{context}_{layer}");
                    assert!(
                        !source.contains(&forbidden),
                        "{} must not depend on business context crate `{}`",
                        source_file.display(),
                        forbidden
                    );
                }
            }
        }
    }
}

#[test]
fn only_api_composition_modules_may_depend_on_integration_crates() {
    let api_src = workspace_root().join("apps/api/src");

    for source_file in rust_files_in(&api_src) {
        let source = read_source(&source_file);
        let relative = source_file.strip_prefix(&api_src).unwrap();
        let is_composition = relative.starts_with("composition");

        if is_composition {
            continue;
        }

        for context in CONTEXTS {
            let forbidden = format!("ordering_food_{context}_integration");
            assert!(
                !source.contains(&forbidden),
                "{} must not depend on integration crate `{}` outside composition root",
                source_file.display(),
                forbidden
            );
        }
    }
}

#[test]
fn business_context_seeds_move_to_explicit_bootstrap_entrypoint() {
    let organization_source =
        read_source(&workspace_root().join("apps/api/src/composition/contexts/organization.rs"));
    let catalog_source =
        read_source(&workspace_root().join("apps/api/src/composition/contexts/catalog.rs"));
    let bootstrap_source = read_source(&workspace_root().join("apps/bootstrap/src/lib.rs"));
    let bootstrap_bin_source = read_source(&workspace_root().join("apps/bootstrap/src/main.rs"));
    let api_manifest = read_source(&workspace_root().join("apps/api/Cargo.toml"));
    let bootstrap_manifest = read_source(&workspace_root().join("apps/bootstrap/Cargo.toml"));
    let api_composition_mod =
        read_source(&workspace_root().join("apps/api/src/composition/mod.rs"));
    let architecture_doc = read_source(&workspace_root().join("ARCHITECTURE.md"));

    assert!(!organization_source.contains("seed_default_organization"));
    assert!(!catalog_source.contains("seed_default_catalog"));
    assert!(bootstrap_source.contains("seed_default_organization"));
    assert!(bootstrap_source.contains("seed_default_catalog"));
    assert!(bootstrap_source.contains("build_organization_context_runtime"));
    assert!(bootstrap_source.contains("build_catalog_context_runtime"));
    assert!(bootstrap_bin_source.contains("run_default_data_bootstrap"));
    assert!(!api_manifest.contains("bootstrap-default-data"));
    assert!(bootstrap_manifest.contains("name = \"ordering-food-bootstrap\""));
    assert!(!bootstrap_manifest.contains("ordering-food-api"));
    assert!(!api_composition_mod.contains("run_default_data_bootstrap"));
    assert!(architecture_doc.contains("Default API startup must not auto-seed business contexts."));
    assert!(architecture_doc.contains("explicit bootstrap command"));
}

fn rust_files_in(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    visit_rust_files(dir, &mut files);
    files.sort();
    files
}

fn visit_rust_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            visit_rust_files(&path, files);
            continue;
        }

        if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn read_source(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
