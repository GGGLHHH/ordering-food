use std::{fs, path::Path};

#[test]
fn ordering_infrastructure_uses_catalog_item_language() {
    let source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/order_repository.rs"))
            .unwrap();

    assert!(
        !source.contains("menu item id"),
        "order_repository.rs unexpectedly keeps legacy menu item wording"
    );
    assert!(
        !source.contains("menu_item_id"),
        "order_repository.rs unexpectedly keeps legacy menu item field naming"
    );
}
