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

const FORBIDDEN_INNER_TECH_TOKENS: &[&str] = &[
    "axum::",
    "sqlx::",
    "redis::",
    "tower_http::",
    "utoipa::",
    "serde_json::",
    "serde_urlencoded::",
];

#[test]
fn domain_crates_do_not_depend_on_web_storage_or_outer_layers() {
    for crate_dir in crate_dirs("-domain") {
        for source_file in rust_files_in(&crate_dir.join("src")) {
            let source = read_source(&source_file);

            for forbidden in FORBIDDEN_INNER_TECH_TOKENS {
                assert!(
                    !source.contains(forbidden),
                    "{} must not depend on outer technology `{}`",
                    source_file.display(),
                    forbidden
                );
            }

            for layer in ["application", "infrastructure", "integration", "published"] {
                for context in CONTEXTS {
                    let forbidden = format!("ordering_food_{context}_{layer}");
                    assert!(
                        !source.contains(&forbidden),
                        "{} must not depend on `{}`",
                        source_file.display(),
                        forbidden
                    );
                }
            }
        }
    }
}

#[test]
fn application_crates_do_not_depend_on_transport_storage_or_foreign_context_layers() {
    for crate_dir in crate_dirs("-application") {
        let owner = context_from_crate_dir(&crate_dir, "-application");

        for source_file in rust_files_in(&crate_dir.join("src")) {
            let source = read_source(&source_file);

            for forbidden in FORBIDDEN_INNER_TECH_TOKENS {
                assert!(
                    !source.contains(forbidden),
                    "{} must not depend on outer technology `{}`",
                    source_file.display(),
                    forbidden
                );
            }

            for context in CONTEXTS {
                if *context == owner {
                    continue;
                }

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
                        "{} must not depend on foreign context internals `{}`",
                        source_file.display(),
                        forbidden
                    );
                }
            }
        }
    }
}

#[test]
fn integration_crates_only_collaborate_with_foreign_published_contracts() {
    for crate_dir in crate_dirs("-integration") {
        let owner = context_from_crate_dir(&crate_dir, "-integration");

        for source_file in rust_files_in(&crate_dir.join("src")) {
            let source = read_source(&source_file);

            for context in CONTEXTS {
                if *context == owner {
                    continue;
                }

                for layer in ["domain", "application", "infrastructure", "integration"] {
                    let forbidden = format!("ordering_food_{context}_{layer}");
                    assert!(
                        !source.contains(&forbidden),
                        "{} must not depend on foreign non-published crate `{}`",
                        source_file.display(),
                        forbidden
                    );
                }
            }
        }
    }
}

fn crate_dirs(suffix: &str) -> Vec<PathBuf> {
    let crates_dir = workspace_root().join("crates");
    let mut dirs = fs::read_dir(&crates_dir)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if !path.is_dir() {
                return None;
            }

            let file_name = path.file_name()?.to_str()?;
            file_name.ends_with(suffix).then_some(path)
        })
        .collect::<Vec<_>>();
    dirs.sort();
    dirs
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

fn context_from_crate_dir(crate_dir: &Path, suffix: &str) -> String {
    crate_dir
        .file_name()
        .and_then(|value| value.to_str())
        .and_then(|value| value.strip_suffix(suffix))
        .unwrap()
        .to_string()
}

fn read_source(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
