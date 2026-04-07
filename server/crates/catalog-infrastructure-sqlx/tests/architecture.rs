use std::{fs, path::Path};

#[test]
fn catalog_runtime_sources_only_target_catalog_schema() {
    for relative_path in [
        "src/lib.rs",
        "src/module.rs",
        "src/transaction.rs",
        "src/brand_catalog_repository.rs",
        "src/store_catalog_repository.rs",
        "src/category_repository.rs",
        "src/item_repository.rs",
        "src/store_item_listing_repository.rs",
        "src/brand_catalog_read_repository.rs",
        "src/store_catalog_read_repository.rs",
        "src/category_read_repository.rs",
        "src/item_read_repository.rs",
    ] {
        let source =
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)).unwrap();
        let forbidden_schemas = [format!("{}.", "menu"), format!("{}.", "organization")];

        for forbidden in forbidden_schemas {
            assert!(
                !source.contains(&forbidden),
                "{relative_path} unexpectedly references foreign schema `{forbidden}`"
            );
        }
    }
}

#[test]
fn catalog_backfill_migration_is_the_only_place_that_references_menu_schema() {
    let source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../database-infrastructure-sqlx/migrations/202604050301_catalog_context.up.sql"),
    )
    .unwrap();
    let expected_references = [
        format!("FROM {}.categories", "menu"),
        format!("FROM {}.items", "menu"),
        format!("FROM {}.brands", "organization"),
        format!("FROM {}.stores", "organization"),
    ];

    for expected in expected_references {
        assert!(
            source.contains(&expected),
            "catalog migration must contain backfill reference `{expected}`"
        );
    }
}
