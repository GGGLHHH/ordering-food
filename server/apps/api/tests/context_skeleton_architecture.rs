use std::{fs, path::Path};

#[test]
fn workspace_members_include_target_published_crates() {
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml")).unwrap();

    for member in [
        "crates/identity-published",
        "crates/catalog-published",
        "crates/ordering-published",
        "crates/access-published",
        "crates/organization-published",
        "crates/fulfillment-published",
        "crates/identity-integration",
        "crates/catalog-integration",
        "crates/ordering-integration",
        "crates/access-integration",
        "crates/organization-integration",
        "crates/fulfillment-integration",
    ] {
        assert!(
            manifest.contains(member),
            "missing workspace member: {member}"
        );
    }
}

#[test]
fn workspace_members_include_catalog_business_crates() {
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml")).unwrap();

    for member in [
        "crates/catalog-domain",
        "crates/catalog-application",
        "crates/catalog-infrastructure-sqlx",
    ] {
        assert!(
            manifest.contains(member),
            "missing workspace member: {member}"
        );
    }
}

#[test]
fn workspace_members_no_longer_include_menu_business_crates() {
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml")).unwrap();

    for member in [
        "crates/menu-domain",
        "crates/menu-application",
        "crates/menu-infrastructure-sqlx",
    ] {
        assert!(
            !manifest.contains(member),
            "unexpected workspace member: {member}"
        );
    }
}

#[test]
fn workspace_members_include_access_infrastructure_and_exclude_authz_members() {
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml")).unwrap();

    assert!(manifest.contains("crates/access-infrastructure-sqlx"));

    for member in [
        "crates/authz-domain",
        "crates/authz-application",
        "crates/authz-infrastructure-sqlx",
    ] {
        assert!(
            !manifest.contains(member),
            "unexpected legacy authz workspace member: {member}"
        );
    }
}

#[test]
fn historical_authz_migrations_remain_in_place() {
    let migrations_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/database-infrastructure-sqlx/migrations");

    assert!(migrations_dir.join("202603150002_authz.up.sql").exists());
    assert!(migrations_dir.join("202603150002_authz.down.sql").exists());
}
