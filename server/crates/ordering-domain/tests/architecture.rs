use std::{fs, path::Path};

#[test]
fn ordering_domain_uses_catalog_language_for_item_identity() {
    for relative_path in [
        "src/lib.rs",
        "src/order.rs",
        "src/order_item.rs",
        "src/catalog_item_id.rs",
    ] {
        let source =
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)).unwrap();

        assert!(
            !source.contains("menu_item_id"),
            "{relative_path} unexpectedly keeps legacy menu item language"
        );
        assert!(
            !source.contains("MenuItemId"),
            "{relative_path} unexpectedly keeps legacy menu item type name"
        );
    }
}
